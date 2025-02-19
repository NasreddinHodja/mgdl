use reqwest::{blocking::get, StatusCode};
use scraper::{Html, Selector};

use crate::{
    db::{Chapter, Manga},
    MgdlError,
};

type Result<T> = std::result::Result<T, MgdlError>;

fn create_selector(selectors: &str) -> Result<Selector> {
    Selector::parse(selectors).map_err(|err| MgdlError::Scrape(err.to_string()))
}

fn get_manga_chapters(manga_hash: &str, manga_id: &str) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = vec![];
    let url = format!("https://weebcentral.com/series/{manga_hash}/full-chapter-list");

    let response = get(url)?;
    if response.status() != StatusCode::OK {
        return Err(MgdlError::Scrape(
            "Could not get manga full chapter list".to_string(),
        ));
    }
    let manga_html = Html::parse_document(&response.text()?);

    let chapter_selector = create_selector("div > a")?;
    let chapter_elements = manga_html.select(&chapter_selector);

    for chapter_element in chapter_elements {
        let url = chapter_element.attr("href").unwrap();
        let hash = url.split("/").last().unwrap();

        let number_selector = create_selector("span > span")?;
        let number = chapter_element
            .select(&number_selector)
            .next()
            .ok_or_else(|| MgdlError::Scrape("Manga name not found".to_string()))?
            .text()
            .collect::<String>()
            .trim()
            .split(" ")
            .last()
            .unwrap()
            .to_string()
            .replace(".", "-");

        let chapter = Chapter::new(&hash, &number, &manga_id);
        chapters.push(chapter)
    }

    Ok(chapters)
}

pub fn manga_from_url(manga_url: &str) -> Result<(Manga, Vec<Chapter>)> {
    println!("Getting manga data from {manga_url}...");

    let response = get(manga_url)?;
    if response.status() != StatusCode::OK {
        return Err(MgdlError::Scrape(format!(
            "Could not access manga url: {}",
            manga_url
        )));
    }
    let manga_html = Html::parse_document(&response.text()?);

    let name_selector = create_selector("main > div > section > section > h1")?;
    let name_element = manga_html
        .select(&name_selector)
        .next()
        .ok_or_else(|| MgdlError::Scrape("Manga name not found".to_string()))?;
    let name = name_element.text().collect::<String>().trim().to_string();
    let normalized_name = manga_url.split("/").last().unwrap().to_lowercase();

    let hash = manga_url
        .split("/")
        .collect::<Vec<&str>>()
        .into_iter()
        .rev()
        .nth(1)
        .unwrap()
        .to_string();

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
            .ok_or_else(|| MgdlError::Scrape("Manga name not found".to_string()))?
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
                    .ok_or_else(|| MgdlError::Scrape("Manga name not found".to_string()))?
                    .text()
                    .collect::<String>()
                    .trim()
                    .to_string();
            }
            _ => {}
        }
    }

    let manga = Manga::new(&hash, &name, &normalized_name, &authors, &status);
    let chapters = get_manga_chapters(&hash, &manga.hash)?;

    Ok((manga, chapters))
}
