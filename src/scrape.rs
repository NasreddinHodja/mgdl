use reqwest::get;
use scraper::{Html, Selector};
use std::{fs, io::Write, path::PathBuf, time::Duration};
use tokio::time::sleep;

use crate::{
    models::{Chapter, Manga, Page},
    utils::{extract_hash, normalize},
    MgdlError,
};

const INITIAL_DELAY: u64 = 300;

type Result<T> = std::result::Result<T, MgdlError>;

fn create_selector(selectors: &str) -> Result<Selector> {
    Selector::parse(selectors).map_err(|err| MgdlError::Scrape(err.to_string()))
}

pub async fn get_chapter_pages(chapter_hash: &str, max_attempts: usize) -> Result<Vec<Page>> {
    let url = format!("https://weebcentral.com/chapters/{}/images?is_prev=False&current_page=1&reading_style=long_strip", chapter_hash);

    let response = retry(
        || async {
            get(&url)
                .await
                .map_err(|err| MgdlError::Scrape(err.to_string()))
        },
        max_attempts,
        Duration::from_millis(INITIAL_DELAY),
    )
    .await?;

    let mut page_urls: Vec<Page> = vec![];

    let html = Html::parse_document(&response.text().await?);

    let pages_selector = create_selector("img")?;

    let chapter_elements = html.select(&pages_selector);
    for chapter_element in chapter_elements {
        let url = chapter_element
            .attr("src")
            .ok_or(MgdlError::Scrape("Could not find page url.".to_string()))?
            .to_string();
        let number = chapter_element
            .attr("alt")
            .ok_or(MgdlError::Scrape("Could not find page name.".to_string()))?
            .to_string()
            .split(' ')
            .last()
            .ok_or(MgdlError::Scrape("Could not find page name.".to_string()))?
            .parse::<usize>()?;

        page_urls.push(Page { url, number });
    }

    Ok(page_urls)
}

async fn get_manga_chapters(
    manga_hash: &str,
    manga_id: &str,
    max_attempts: usize,
) -> Result<Vec<Chapter>> {
    let url = format!("https://weebcentral.com/series/{manga_hash}/full-chapter-list");

    let response = retry(
        || async {
            get(&url)
                .await
                .map_err(|err| MgdlError::Scrape(err.to_string()))
        },
        max_attempts,
        Duration::from_millis(INITIAL_DELAY),
    )
    .await?;

    let mut chapters: Vec<Chapter> = vec![];

    let manga_html = Html::parse_document(&response.text().await?);

    let chapter_selector = create_selector("div > a")?;
    let chapter_elements = manga_html.select(&chapter_selector);

    for chapter_element in chapter_elements {
        let url = chapter_element
            .attr("href")
            .ok_or(MgdlError::Scrape("Could not find chapter URL".to_string()))?;
        let hash = url
            .split('/')
            .last()
            .ok_or(MgdlError::Scrape("Could not find chapter hash".to_string()))?;

        let number_selector = create_selector("span > span")?;
        let numbers_str = chapter_element
            .select(&number_selector)
            .next()
            .ok_or(MgdlError::Scrape("Chapter number not found".to_string()))?
            .text()
            .collect::<String>()
            .trim()
            .split(' ')
            .last()
            .ok_or(MgdlError::Scrape("Chapter number not found".to_string()))?
            .to_string();
        let numbers = numbers_str
            .split('.')
            .map(|number: &str| number.parse::<usize>())
            .collect::<std::result::Result<Vec<usize>, _>>()?;

        if numbers.len() == 1 || numbers.len() == 2 {
            let num = format!("{:04}", numbers[0]);
            let subnum = if numbers.len() == 1 {
                "01".to_string()
            } else {
                format!("{:02}", numbers[1])
            };

            let number = format!("{}-{}", num, subnum);
            let chapter = Chapter::new(&hash, &number, &manga_id);
            chapters.push(chapter);
        } else {
            return Err(MgdlError::Scrape(
                "Could not find chapter number.".to_string(),
            ));
        }
    }

    Ok(chapters)
}

pub async fn manga_from_url(manga_url: &str, max_attempts: usize) -> Result<(Manga, Vec<Chapter>)> {
    let response = retry(
        || async {
            get(manga_url)
                .await
                .map_err(|err| MgdlError::Scrape(err.to_string()))
        },
        max_attempts,
        Duration::from_millis(INITIAL_DELAY),
    )
    .await?;

    let manga_html = Html::parse_document(&response.text().await?);

    let name_selector = create_selector("main > div > section > section > h1")?;
    let name_element = manga_html
        .select(&name_selector)
        .next()
        .ok_or(MgdlError::Scrape("Manga name not found".to_string()))?;
    let name = name_element.text().collect::<String>().trim().to_string();
    let normalized_name = normalize(&name);

    let hash = extract_hash(&manga_url).ok_or(MgdlError::Scrape(format!(
        "Could not parse manga hash from {}",
        &manga_url
    )))?;

    let mut authors = "".to_string();
    let mut status = "".to_string();

    let infos_selector =
        create_selector("main > div > section > section > section > ul.flex.flex-col.gap-4 > li")?;
    let info_elements = manga_html.select(&infos_selector);

    for info_element in info_elements {
        let strong_selector = create_selector("strong")?;
        let info_label = info_element
            .select(&strong_selector)
            .next()
            .ok_or(MgdlError::Scrape("Manga name not found".to_string()))?
            .text()
            .collect::<String>()
            .trim()
            .to_string()
            .replace(":", "")
            .replace("(s)", "");

        match info_label.as_str() {
            "Author" => {
                let authors_selector = create_selector("span > a")?;
                authors = info_element
                    .select(&authors_selector)
                    .map(|element| element.text().collect::<String>().trim().to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
            }
            "Status" => {
                let status_selector =
                    Selector::parse("a").map_err(|err| MgdlError::Scrape(err.to_string()))?;
                status = info_element
                    .select(&status_selector)
                    .next()
                    .ok_or(MgdlError::Scrape("Manga name not found".to_string()))?
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_string();
            }
            _ => {}
        }
    }

    let manga = Manga::new(&hash, &name, &normalized_name, &authors, &status);
    let chapters = get_manga_chapters(&hash, &manga.hash, 20).await?;

    Ok((manga, chapters))
}

pub async fn download_page(
    page_url: String,
    chapter_path: PathBuf,
    page_number: usize,
    max_attempts: usize,
) -> Result<()> {
    let response = retry(
        || async {
            get(&page_url)
                .await
                .map_err(|err| MgdlError::Scrape(err.to_string()))
        },
        max_attempts,
        Duration::from_millis(INITIAL_DELAY),
    )
    .await?;

    let bytes = response.bytes().await?;

    let file_ext = page_url.split('.').last().ok_or(MgdlError::Scrape(
        "Could not find file extension".to_string(),
    ))?;
    let file_name = format!("{:03}.{}", page_number, file_ext);
    let file_path = chapter_path.join(&file_name);

    let mut file = fs::File::create(&file_path)?;
    file.write_all(&bytes)?;

    Ok(())
}

async fn retry<F, Fut, T>(
    mut operation: F,
    max_attempts: usize,
    initial_delay: Duration,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempts = 0;
    let mut delay = initial_delay;

    while attempts < max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(_) => {
                attempts += 1;
                sleep(delay.clone()).await;
                delay *= 2;
            }
        }
    }

    Err(MgdlError::Scrape(
        "Operation failed after max attempts\n{:?}".to_string(),
    ))
}
