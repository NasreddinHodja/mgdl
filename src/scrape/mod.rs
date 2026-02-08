mod html;

use html::{extract_attr, extract_tag_content, find_all_tags, strip_tags};
use reqwest::Client;
use std::{path::PathBuf, time::Duration};
use tokio::{fs, io::AsyncWriteExt, time::sleep};

#[cfg(feature = "bench")]
use {csv::Writer, uuid::Uuid};

use crate::{
    error::{MgdlError, MgdlResult},
    models::{Chapter, Manga, Page},
    utils::{extract_hash, normalize},
};

const INITIAL_DELAY: u64 = 300;

/// Parse page image data from pre-fetched HTML (the chapter images page).
pub fn parse_pages_from_html(html: &str) -> MgdlResult<Vec<Page>> {
    let img_tags = find_all_tags(html, "img");

    let pages: Vec<Page> = img_tags
        .into_iter()
        .map(|tag| {
            let url = extract_attr(tag, "src")
                .ok_or(MgdlError::Scrape("Could not find page url".to_string()))?
                .to_string();
            let alt = extract_attr(tag, "alt")
                .ok_or(MgdlError::Scrape("Could not find page number".to_string()))?;
            let number = alt
                .split(' ')
                .next_back()
                .ok_or(MgdlError::Scrape("Could not find page number".to_string()))?
                .parse::<usize>()?;
            Ok(Page { url, number })
        })
        .collect::<MgdlResult<Vec<_>>>()?;

    if pages.is_empty() {
        return Err(MgdlError::Scrape("No pages found for chapter".to_string()));
    }

    Ok(pages)
}

pub async fn get_chapter_pages(
    client: &Client,
    base_url: &str,
    chapter_hash: &str,
    max_attempts: usize,
) -> MgdlResult<Vec<Page>> {
    let url = format!(
        "{}/chapters/{}/images?is_prev=False&current_page=1&reading_style=long_strip",
        base_url, chapter_hash
    );

    let html = get_with_retry(client, &url, max_attempts).await?;
    parse_pages_from_html(&html)
}

/// Parse chapter list from pre-fetched HTML (the full-chapter-list page).
pub fn parse_chapters_from_html(html: &str) -> MgdlResult<Vec<Chapter>> {
    let mut chapters = Vec::new();

    let divs = find_all_tags(html, "div");
    for div in divs {
        let links = find_all_tags(div, "a");
        let Some(link) = links.first() else {
            continue;
        };

        let Some(href) = extract_attr(link, "href") else {
            continue;
        };

        let hash = href
            .split('/')
            .next_back()
            .ok_or(MgdlError::Scrape("Could not find chapter hash".to_string()))?;

        // Extract chapter number from link text (e.g., "Chapter 18" or "Chapter 5.5")
        let link_text = strip_tags(link);
        let raw_number = link_text
            .split_whitespace()
            .skip_while(|w| *w != "Chapter")
            .nth(1)
            .ok_or(MgdlError::Scrape("Chapter number not found".to_string()))?
            .to_string();

        let parts: Vec<usize> = raw_number
            .split('.')
            .map(|s| s.parse::<usize>())
            .collect::<Result<Vec<_>, _>>()?;

        let number = match parts.as_slice() {
            [major] => format!("{:04}-01", major),
            [major, minor] => format!("{:04}-{:02}", major, minor),
            _ => {
                return Err(MgdlError::Scrape(
                    "Invalid chapter number format".to_string(),
                ))
            }
        };

        chapters.push(Chapter::new(hash, &number));
    }

    Ok(chapters)
}

/// Parse manga metadata from pre-fetched HTML (the manga series page).
/// `url` is the original manga URL, used to extract the hash.
pub fn parse_manga_from_html(html: &str, url: &str) -> MgdlResult<Manga> {
    let name = extract_tag_content(html, "h1")
        .ok_or(MgdlError::Scrape("Manga name not found".to_string()))?
        .trim()
        .to_string();
    let normalized_name = normalize(&name);

    let hash = extract_hash(url).ok_or(MgdlError::Scrape(format!(
        "Could not parse manga hash from {}",
        url
    )))?;

    let mut authors = String::new();
    let mut status = String::new();

    // Find the <ul class="flex flex-col gap-4"> list
    let uls = find_all_tags(html, "ul");
    for ul in uls {
        if !ul.contains("flex flex-col gap-4") {
            continue;
        }

        let lis = find_all_tags(ul, "li");
        for li in lis {
            let Some(strong_content) = extract_tag_content(li, "strong") else {
                continue;
            };
            let label = strong_content.trim().replace(':', "").replace("(s)", "");

            match label.as_str() {
                "Author" => {
                    let author_links = find_all_tags(li, "a");
                    authors = author_links
                        .iter()
                        .map(|a| strip_tags(a).trim().to_string())
                        .collect::<Vec<_>>()
                        .join(", ");
                }
                "Status" => {
                    let status_links = find_all_tags(li, "a");
                    if let Some(a) = status_links.first() {
                        status = strip_tags(a).trim().to_string();
                    }
                }
                _ => {}
            }
        }
        break; // only process the first matching <ul>
    }

    Ok(Manga::new(
        &hash,
        &name,
        &normalized_name,
        &authors,
        &status,
    ))
}

pub async fn manga_from_url(
    client: &Client,
    base_url: &str,
    manga_url: &str,
    max_attempts: usize,
) -> MgdlResult<(Manga, Vec<Chapter>)> {
    let html = get_with_retry(client, manga_url, max_attempts).await?;
    let manga = parse_manga_from_html(&html, manga_url)?;
    let chapters = get_manga_chapters(client, base_url, &manga.hash, max_attempts).await?;
    Ok((manga, chapters))
}

async fn get_manga_chapters(
    client: &Client,
    base_url: &str,
    manga_hash: &str,
    max_attempts: usize,
) -> MgdlResult<Vec<Chapter>> {
    let url = format!("{base_url}/series/{manga_hash}/full-chapter-list");
    let html = get_with_retry(client, &url, max_attempts).await?;
    parse_chapters_from_html(&html)
}

pub async fn download_page(
    client: &Client,
    page_url: String,
    chapter_path: PathBuf,
    page_number: usize,
    max_attempts: usize,
) -> MgdlResult<usize> {
    let response = retry(
        || async { Ok(client.get(&page_url).send().await?) },
        max_attempts,
        INITIAL_DELAY,
    )
    .await?;

    let bytes = response.bytes().await?;
    let byte_count = bytes.len();

    let url_without_query = page_url.split('?').next().unwrap_or(&page_url);
    let file_ext = url_without_query
        .split('.')
        .next_back()
        .ok_or(MgdlError::Scrape(
            "Could not find file extension".to_string(),
        ))?;

    let file_path = chapter_path.join(format!("{:03}.{}", page_number, file_ext));
    let mut file = fs::File::create(&file_path).await?;
    file.write_all(&bytes).await?;

    Ok(byte_count)
}

async fn retry<F, Fut, T>(
    mut operation: F,
    max_attempts: usize,
    initial_delay: u64,
) -> MgdlResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = MgdlResult<T>>,
{
    let mut delay = initial_delay;

    for attempt in 0..max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(_) if attempt + 1 < max_attempts => {
                sleep(Duration::from_millis(delay)).await;
                delay *= 2;
            }
            Err(e) => return Err(e),
        }
    }

    Err(MgdlError::Scrape("Max retry attempts exhausted".into()))
}

pub async fn get_with_retry(client: &Client, url: &str, max_attempts: usize) -> MgdlResult<String> {
    retry(
        || async {
            let response = client.get(url).send().await?;
            let text = response.text().await?;

            if text.contains("error code: 1015") {
                return Err(MgdlError::Scrape(format!(
                    "Rate limited while accessing {}",
                    url
                )));
            }

            Ok(text)
        },
        max_attempts,
        INITIAL_DELAY,
    )
    .await
}

#[cfg(feature = "bench")]
pub async fn scrape_to_csv(
    client: &Client,
    base_url: &str,
    manga_url: &str,
    max_attempts: Option<usize>,
) -> MgdlResult<()> {
    let max_attempts = max_attempts.unwrap_or(10);
    let (manga, chapters) = manga_from_url(client, base_url, manga_url, max_attempts).await?;

    let manga_id = Uuid::new_v4().to_string();

    let mut manga_w = Writer::from_path("manga.csv")?;
    let mut page_w = Writer::from_path("page.csv")?;

    manga_w.write_record(["id", "hash", "name", "normalized_name", "authors", "status"])?;
    manga_w.write_record([
        &manga_id,
        &manga.hash,
        &manga.name,
        &manga.normalized_name,
        &manga.authors,
        &manga.status,
    ])?;

    page_w.write_record(["id", "manga_id", "chapter_number", "number", "url"])?;

    for chapter in chapters {
        let pages = get_chapter_pages(client, base_url, &chapter.hash, max_attempts).await?;

        for page in pages {
            let page_id = Uuid::new_v4().to_string();
            let page_number = page.number.to_string();

            page_w.write_record([
                &page_id,
                &manga_id,
                &chapter.number,
                &page_number,
                &page.url,
            ])?;
        }
    }

    manga_w.flush()?;
    page_w.flush()?;

    Ok(())
}
