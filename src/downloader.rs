use crate::MgdlError;
use crate::{
    db,
    db::{Chapter, Manga},
    scrape,
};
use std::fs;
use std::path::PathBuf;

pub struct Downloader {
    db: db::Db,
    manga_dir: PathBuf,
}

type Result<T> = std::result::Result<T, MgdlError>;

impl Downloader {
    pub fn new(manga_dir: PathBuf, db_dir: PathBuf) -> Result<Self> {
        let db_path = db_dir.join(PathBuf::from("mgdl.db"));
        let db = db::Db::new(db_path);
        db.init()?;

        Ok(Self {
            db,
            manga_dir: manga_dir,
        })
    }

    pub fn add(&self, manga_url: &str) -> Result<(Manga, Vec<Chapter>)> {
        let (manga, chapters) = scrape::manga_from_url(manga_url)?;

        let added_manga = self.db.add_manga(manga, &chapters)?;

        Ok((added_manga, chapters))
    }

    pub fn download_manga(&self, manga_url: &str) -> Result<()> {
        let (manga, chapters) = self.add(manga_url)?;
        println!("Downloading manga {}", &manga.name);
        let manga_path = self
            .manga_dir
            .join(PathBuf::from(format!("{}", &manga.normalized_name)));
        self.download_chapters(&manga_path, &chapters, None)?;
        Ok(())
    }

    pub fn download_chapters(&self, manga_path: &PathBuf, chapters: &Vec<Chapter>, skip_chaps: Option<usize>) -> Result<()> {
        fs::create_dir_all(&manga_path)?;
        for chapter in chapters {
            let chapter_number = chapter
                .number
                .split('-')
                .next()
                .ok_or(MgdlError::Downloader(
                    "Could not find manga's name".to_string(),
                ))?
                .parse::<usize>()?;
            if let Some(skip) = skip_chaps {
                if chapter_number <= skip {
                    continue;
                }
            }
            let pages = scrape::get_chapter_pages(&chapter.hash)?;
            let chapter_path =
                manga_path.join(PathBuf::from(format!("chapter_{}", &chapter.number)));

            fs::create_dir_all(&chapter_path)?;

            for page in pages {
                scrape::download_page(&page.url, &chapter_path, page.number, 3)?;
            }
        }

        Ok(())
    }

    pub fn update(&self, manga_name: &str) -> Result<Manga> {
        let manga = self.db.get_manga_by_normalized_name(manga_name)?;
        let chapters = self.db.get_manga_chapters(&manga)?;
        let skip_chaps = self.skip_chaps(&manga)?;
        let manga_path = self
            .manga_dir
            .join(PathBuf::from(format!("{}", &manga.normalized_name)));

        self.download_chapters(&manga_path, &chapters, Some(skip_chaps))?;

        Ok(manga)
    }

    pub fn update_all(&self) -> Result<()> {
        for manga in self.db.get_ongoing_manga()? {
            println!("Trying to update {}", &manga.name);
            self.update(&manga.normalized_name)?;
        }

        Ok(())
    }

    pub fn skip_chaps(&self, manga: &Manga) -> Result<usize> {
        let manga_path = self.manga_dir.join(&manga.normalized_name);
        let mut chaps: Vec<usize> = fs::read_dir(&manga_path)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|name| name.contains("chapter"))
            .filter_map(|name| {
                name.split('_')
                    .nth(1)
                    .and_then(|s| s.split('-').next())
                    .and_then(|num| num.parse::<usize>().ok())
            })
            .collect();

        if chaps.is_empty() {
            chaps.push(0);
        }

        Ok(*(chaps.iter().max().ok_or(MgdlError::Downloader(
            "Could not find manga name".to_string(),
        ))?))
    }

    pub fn reset_db(&self) -> Result<()> {
        self.db.drop()?;
        Ok(())
    }
}
