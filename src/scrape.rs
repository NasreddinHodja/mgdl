use csv::Writer;
use reqwest::get;
use scraper::{Html, Selector};
use std::{path::PathBuf, time::Duration};
use tokio::{fs, io::AsyncWriteExt, time::sleep};
use uuid::Uuid;

use crate::{
    error::{MgdlError, MgdlResult},
    models::{Chapter, Manga, Page},
    utils::{extract_hash, normalize},
};

const INITIAL_DELAY: u64 = 300;

fn create_selector(selectors: &str) -> MgdlResult<Selector> {
    Selector::parse(selectors).map_err(|err| MgdlError::Scrape(err.to_string()))
}

pub async fn get_chapter_pages(base_url: &str, chapter_hash: &str, max_attempts: usize) -> MgdlResult<Vec<Page>> {
    let url = format!(
        "{}/chapters/{}/images?is_prev=False&current_page=1&reading_style=long_strip",
        base_url, chapter_hash
    );

    let html = get_with_retry(&url, max_attempts).await?;
    let selector = create_selector("img")?;

    let pages: Vec<Page> = html
        .select(&selector)
        .map(|el| {
            let url = el
                .attr("src")
                .ok_or(MgdlError::Scrape("Could not find page url".to_string()))?
                .to_string();
            let number = el
                .attr("alt")
                .ok_or(MgdlError::Scrape("Could not find page number".to_string()))?
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

async fn get_manga_chapters(base_url: &str, manga_hash: &str, max_attempts: usize) -> MgdlResult<Vec<Chapter>> {
    let url = format!("{base_url}/series/{manga_hash}/full-chapter-list");
    let html = get_with_retry(&url, max_attempts).await?;

    let link_selector = create_selector("div > a")?;
    let number_selector = create_selector("span > span")?;

    let mut chapters = Vec::new();

    for el in html.select(&link_selector) {
        let hash = el
            .attr("href")
            .ok_or(MgdlError::Scrape("Could not find chapter URL".to_string()))?
            .split('/')
            .next_back()
            .ok_or(MgdlError::Scrape("Could not find chapter hash".to_string()))?;

        let numbers_str = el
            .select(&number_selector)
            .next()
            .ok_or(MgdlError::Scrape("Chapter number not found".to_string()))?
            .text()
            .collect::<String>();

        let raw_number = numbers_str
            .trim()
            .split(' ')
            .next_back()
            .ok_or(MgdlError::Scrape("Chapter number not found".to_string()))?;

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

pub async fn manga_from_url(
    base_url: &str,
    manga_url: &str,
    max_attempts: usize,
) -> MgdlResult<(Manga, Vec<Chapter>)> {
    let html = get_with_retry(manga_url, max_attempts).await?;

    let name_selector = create_selector("main > div > section > section > h1")?;
    let name = html
        .select(&name_selector)
        .next()
        .ok_or(MgdlError::Scrape("Manga name not found".to_string()))?
        .text()
        .collect::<String>()
        .trim()
        .to_string();
    let normalized_name = normalize(&name);

    let hash = extract_hash(manga_url).ok_or(MgdlError::Scrape(format!(
        "Could not parse manga hash from {}",
        manga_url
    )))?;

    let mut authors = String::new();
    let mut status = String::new();

    let infos_selector =
        create_selector("main > div > section > section > section > ul.flex.flex-col.gap-4 > li")?;
    let strong_selector = create_selector("strong")?;

    for info_el in html.select(&infos_selector) {
        let label = info_el
            .select(&strong_selector)
            .next()
            .ok_or(MgdlError::Scrape("Info label not found".to_string()))?
            .text()
            .collect::<String>()
            .trim()
            .replace(':', "")
            .replace("(s)", "");

        match label.as_str() {
            "Author" => {
                let sel = create_selector("span > a")?;
                authors = info_el
                    .select(&sel)
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
            }
            "Status" => {
                let sel = create_selector("a")?;
                status = info_el
                    .select(&sel)
                    .next()
                    .ok_or(MgdlError::Scrape("Status value not found".to_string()))?
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_string();
            }
            _ => {}
        }
    }

    let manga = Manga::new(&hash, &name, &normalized_name, &authors, &status);
    let chapters = get_manga_chapters(base_url, &hash, max_attempts).await?;

    Ok((manga, chapters))
}

pub async fn download_page(
    page_url: String,
    chapter_path: PathBuf,
    page_number: usize,
    max_attempts: usize,
) -> MgdlResult<()> {
    let response = retry(
        || async { Ok(get(&page_url).await?) },
        max_attempts,
        INITIAL_DELAY,
    )
    .await?;

    let bytes = response.bytes().await?;

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

    Ok(())
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

async fn get_with_retry(url: &str, max_attempts: usize) -> MgdlResult<Html> {
    retry(
        || async {
            let response = get(url).await?;
            let text = response.text().await?;
            let html = Html::parse_document(&text);

            if html.html().contains("error code: 1015") {
                return Err(MgdlError::Scrape(format!(
                    "Rate limited while accessing {}",
                    url
                )));
            }

            Ok(html)
        },
        max_attempts,
        INITIAL_DELAY,
    )
    .await
}

pub async fn scrape_to_csv(base_url: &str, manga_url: &str, max_attempts: Option<usize>) -> MgdlResult<()> {
    let max_attempts = max_attempts.unwrap_or(10);
    let (manga, chapters) = manga_from_url(base_url, manga_url, max_attempts).await?;

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
        let pages = get_chapter_pages(base_url, &chapter.hash, max_attempts).await?;

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
