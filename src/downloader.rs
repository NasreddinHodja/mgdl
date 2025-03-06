use crate::MgdlError;
use tokio;
use crate::{db, scrape, Chapter, Manga};
use std::fs;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, MgdlError>;

pub struct Downloader {
    db: db::Db,
    manga_dir: PathBuf,
}

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

    pub async fn add(&self, manga_url: &str) -> Result<(Manga, Vec<Chapter>)> {
        let (manga, chapters) = scrape::manga_from_url(manga_url).await?;

        let added_manga = self.db.add_manga(manga)?;

        Ok((added_manga, chapters))
    }

    pub async fn download_manga(&self, manga_url: &str) -> Result<Manga> {
        let (manga, chapters) = self.add(manga_url).await?;
        println!("Downloading manga {}", &manga.name);
        let manga_path = self
            .manga_dir
            .join(PathBuf::from(format!("{}", &manga.normalized_name)));

        self.download_chapters(&manga_path, &chapters, None).await?;

        Ok(manga)
    }

    pub async fn download_chapters(
        &self,
        manga_path: &PathBuf,
        chapters: &Vec<Chapter>,
        skip_chaps: Option<usize>,
    ) -> Result<()> {
        fs::create_dir_all(&manga_path)?;

        let mut tasks = vec![];

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

            let pages = scrape::get_chapter_pages(&chapter.hash).await?;
            let chapter_path =
                manga_path.join(PathBuf::from(format!("chapter_{}", &chapter.number)));

            fs::create_dir_all(&chapter_path)?;


            println!("+ Chapter {}", chapter.number);
            for page in pages {
                let chapter_path = chapter_path.clone();
                let page_url = page.url.clone();
                let page_number = page.number.clone();
                let handle = tokio::spawn(async move {
                    scrape::download_page(&page_url, &chapter_path, page_number, 3).await.ok();
                });
                tasks.push(handle);
            }
        }

        for task in tasks {
            task.await?;
        }
        Ok(())
    }

    pub async fn update(&self, manga_name: &str) -> Result<Manga> {
        let manga = self.db.get_manga_by_normalized_name(manga_name)?;
        let manga_url = format!("https://weebcentral.com/series/{}", &manga.hash);
        let (new_manga, chapters) = scrape::manga_from_url(&manga_url).await?;
        let skip_chaps = self.skip_chaps(&new_manga)?;
        let manga_path = self
            .manga_dir
            .join(PathBuf::from(format!("{}", &new_manga.normalized_name)));

        self.download_chapters(&manga_path, &chapters, Some(skip_chaps)).await?;

        Ok(new_manga)
    }

    pub async fn update_all(&self) -> Result<()> {
        for manga in self.db.get_ongoing_manga()? {
            println!("Trying to update {}", &manga.name);
            self.update(&manga.normalized_name).await?;
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
