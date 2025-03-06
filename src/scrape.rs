use regex::Regex;
use reqwest::{get, StatusCode};
use scraper::{Html, Selector};
use std::io::Write;
use std::time::Duration;
use std::{fs, path::PathBuf, thread::sleep};

use crate::{
    models::{Chapter, Manga},
    MgdlError,
};

type Result<T> = std::result::Result<T, MgdlError>;

#[derive(Debug)]
pub struct Page {
    pub url: String,
    pub number: usize,
}

fn create_selector(selectors: &str) -> Result<Selector> {
    Selector::parse(selectors).map_err(|err| MgdlError::Scrape(err.to_string()))
}

pub async fn get_chapter_pages(chapter_hash: &str) -> Result<Vec<Page>> {
    let url = format!("https://weebcentral.com/chapters/{}/images?is_prev=False&current_page=1&reading_style=long_strip", chapter_hash);
    let mut page_urls: Vec<Page> = vec![];

    let response = get(url).await?;
    if response.status() != StatusCode::OK {
        return Err(MgdlError::Scrape(format!(
            "Could not get chapter {} full page list",
            chapter_hash
        )));
    }

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

async fn get_manga_chapters(manga_hash: &str, manga_id: &str) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = vec![];
    let url = format!("https://weebcentral.com/series/{manga_hash}/full-chapter-list");

    let response = get(url).await?;
    if response.status() != StatusCode::OK {
        return Err(MgdlError::Scrape(
            "Could not get manga full chapter list".to_string(),
        ));
    }
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
            .ok_or(MgdlError::Scrape("Manga name not found".to_string()))?
            .text()
            .collect::<String>()
            .trim()
            .split(' ')
            .last()
            .ok_or(MgdlError::Scrape("Manga name not found".to_string()))?
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

pub async fn manga_from_url(manga_url: &str) -> Result<(Manga, Vec<Chapter>)> {
    let response = get(manga_url).await?;

    if response.status() != StatusCode::OK {
        return Err(MgdlError::Scrape(format!(
            "Could not access manga url: {}",
            manga_url
        )));
    }

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
    let chapters = get_manga_chapters(&hash, &manga.hash).await?;

    Ok((manga, chapters))
}

pub async fn download_page(
    page_url: &str,
    chapter_path: &PathBuf,
    page_number: usize,
    max_attempts: usize,
) -> Result<()> {
    let mut attempts = 0;
    let mut delay = Duration::from_micros(100);
    loop {
        if attempts == max_attempts {
            break;
        }

        let response = get(page_url).await?;

        if response.status() != StatusCode::OK {
            attempts += 1;
            sleep(delay);
            delay *= 2;
            continue;
        }

        let bytes = response.bytes().await?;

        let file_ext = page_url.split('.').last().ok_or(MgdlError::Scrape(
            "Could not find file extension".to_string(),
        ))?;
        let file_name = format!("{:03}.{}", page_number, file_ext);
        let file_path = chapter_path.join(PathBuf::from(file_name));

        let mut file = fs::File::create(&file_path)?;
        file.write_all(&bytes)?;

        return Ok(());
    }

    return Err(MgdlError::Scrape(format!(
        "Could not download page from {}",
        page_url
    )));
}

fn normalize(s: &str) -> String {
    let replacements = [
        (r"[áàâãä]", "a"),
        (r"[éèêë]", "e"),
        (r"[íìîï]", "i"),
        (r"[óòôõö]", "o"),
        (r"[úùûü]", "u"),
        (r"[ç]", "c"),
        (r"[ñ]", "n"),
        (r"[ýÿ]", "y"),
        (r"[ÁÀÂÃÄ]", "A"),
        (r"[ÉÈÊË]", "E"),
        (r"[ÍÌÎÏ]", "I"),
        (r"[ÓÒÔÕÖ]", "O"),
        (r"[ÚÙÛÜ]", "U"),
        (r"[Ç]", "C"),
        (r"[Ñ]", "N"),
        (r"[Ý]", "Y"),
    ];

    let mut s = s.to_string();

    for (pattern, replacement) in &replacements {
        let re = Regex::new(pattern).unwrap();
        s = re.replace_all(&s, *replacement).to_string();
    }

    let re = Regex::new(r"[^a-zA-Z0-9]+").unwrap();
    s = re.replace_all(&s, "_").to_string();

    s.trim_matches('_').to_lowercase()
}

fn extract_hash(url: &str) -> Option<String> {
    let re = Regex::new(r"/series/([^/]+)(?:/|$)").ok()?;
    re.captures(url)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}
