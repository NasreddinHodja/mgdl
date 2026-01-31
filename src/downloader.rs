use std::sync::Arc;
use std::{fs, path::Path, path::PathBuf};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::{
    db,
    error::{MgdlError, MgdlResult},
    logger::{LogMode, Logger},
    models::{Chapter, Manga},
    scrape,
};

const MAX_ATTEMPTS: usize = 20;

pub struct Downloader {
    db: db::Db,
    manga_dir: PathBuf,
    logger: Logger,
}

impl Downloader {
    pub fn new(manga_dir: PathBuf, db_dir: PathBuf, log_mode: LogMode) -> MgdlResult<Self> {
        let db = db::Db::new(db_dir.join("mgdl.db"))?;
        let logger = Logger::new(log_mode);

        Ok(Self {
            db,
            manga_dir,
            logger,
        })
    }

    pub async fn add(&self, manga_url: &str) -> MgdlResult<(Manga, Vec<Chapter>)> {
        let spinner = self
            .logger
            .add_spinner(Some("Scraping manga and chapters".to_owned()))?;

        let (manga, chapters) = scrape::manga_from_url(manga_url, MAX_ATTEMPTS).await?;

        spinner.set_message(format!("Adding manga {}", &manga.name));
        let added_manga = self.db.upsert_manga(manga)?;

        self.logger.finish_spinner(spinner);
        Ok((added_manga, chapters))
    }

    pub async fn download_manga(&self, manga_url: &str) -> MgdlResult<Manga> {
        let (manga, chapters) = self.add(manga_url).await?;
        let manga_path = self.manga_dir.join(&manga.normalized_name);

        let spinner = self
            .logger
            .add_spinner(Some(format!("Downloading {}", &manga.name)))?;

        self.download_chapters(&manga_path, &manga.name, &chapters, None)
            .await?;

        self.logger.finish_spinner(spinner);
        Ok(manga)
    }

    async fn download_chapters(
        &self,
        manga_path: &Path,
        manga_name: &str,
        chapters: &[Chapter],
        skip_chaps: Option<&[usize]>,
    ) -> MgdlResult<()> {
        fs::create_dir_all(manga_path)?;

        let mut join_set = JoinSet::new();
        let semaphore = Arc::new(Semaphore::new(16));

        let progress_bar = self.logger.add_bar(chapters.len() as u64)?;
        progress_bar.set_prefix("Locating chapters and pages".to_string());
        for chapter in chapters {
            let chapter_number = chapter
                .number
                .split('-')
                .next()
                .ok_or(MgdlError::Downloader(
                    "Could not parse chapter number".to_string(),
                ))?
                .parse::<usize>()?;

            if skip_chaps.is_some_and(|skips| skips.contains(&chapter_number)) {
                continue;
            }

            let pages = scrape::get_chapter_pages(&chapter.hash, MAX_ATTEMPTS).await?;
            let chapter_path = manga_path.join(format!("chapter_{}", &chapter.number));

            fs::create_dir_all(&chapter_path)?;

            for page in pages {
                let chapter_path = chapter_path.clone();
                let permit = Arc::clone(&semaphore);
                join_set.spawn(async move {
                    let _permit = permit.acquire().await.unwrap();
                    scrape::download_page(page.url, chapter_path, page.number, MAX_ATTEMPTS).await
                });
            }
            progress_bar.inc(1);
            progress_bar.success(format!(
                "Queued \"{}\" - Chapter {}",
                manga_name, &chapter.number
            ));
        }
        self.logger.finish_bar(progress_bar);

        let progress_bar = self.logger.add_bar(join_set.len() as u64)?;
        progress_bar.set_prefix("Downloading pages".to_string());
        while let Some(res) = join_set.join_next().await {
            res??;
            progress_bar.inc(1);
        }
        self.logger.finish_bar(progress_bar);

        Ok(())
    }

    pub async fn update(&self, manga_name: &str) -> MgdlResult<Manga> {
        let spinner = self
            .logger
            .add_spinner(Some("Getting local manga data".to_owned()))?;

        let manga = self.db.get_manga_by_normalized_name(manga_name)?;
        let manga_url = format!("https://weebcentral.com/series/{}", &manga.hash);

        spinner.set_message("Scraping manga and chapters".to_owned());
        let (new_manga, chapters) = scrape::manga_from_url(&manga_url, MAX_ATTEMPTS).await?;
        let skip_chaps = self.existing_chapter_numbers(&new_manga)?;
        let manga_path = self.manga_dir.join(&new_manga.normalized_name);

        spinner.set_message(format!("Downloading {}", &manga.name));
        self.download_chapters(&manga_path, &manga.name, &chapters, Some(&skip_chaps))
            .await?;

        self.logger.finish_spinner(spinner);
        Ok(new_manga)
    }

    pub async fn update_all(&self) -> MgdlResult<()> {
        let spinner = self.logger.add_spinner(None)?;

        spinner.set_message("Cleaning up missing manga directories".to_owned());
        let ongoing_manga = self.cleanup_missing_manga_dirs()?;

        for manga in ongoing_manga {
            spinner.set_message(format!("Trying to update {}", &manga.name));
            self.update(&manga.normalized_name).await?;
        }

        self.logger.finish_spinner(spinner);
        Ok(())
    }

    fn cleanup_missing_manga_dirs(&self) -> MgdlResult<Vec<Manga>> {
        let ongoing_manga = self.db.get_ongoing_manga()?;
        let mut deleted_count = 0;
        let mut remaining = Vec::new();

        for manga in ongoing_manga {
            let manga_path = self.manga_dir.join(&manga.normalized_name);
            if !manga_path.exists() {
                self.db
                    .delete_manga_by_normalized_name(&manga.normalized_name)?;
                deleted_count += 1;
            } else {
                remaining.push(manga);
            }
        }

        if deleted_count > 0 {
            eprintln!(
                "Cleaned up {} manga entries with missing directories",
                deleted_count
            );
        }

        Ok(remaining)
    }

    fn existing_chapter_numbers(&self, manga: &Manga) -> MgdlResult<Vec<usize>> {
        let manga_path = self.manga_dir.join(&manga.normalized_name);
        if !manga_path.exists() {
            return Ok(vec![]);
        }
        let mut chapters = Vec::new();
        for entry in fs::read_dir(&manga_path)? {
            let entry = entry?;
            let Some(name) = entry.file_name().into_string().ok() else {
                continue;
            };
            let Some(num) = name
                .strip_prefix("chapter_")
                .and_then(|s| s.split('-').next())
                .and_then(|s| s.parse().ok())
            else {
                continue;
            };
            chapters.push(num);
        }

        Ok(chapters)
    }

    pub fn reset_db(&self) -> MgdlResult<()> {
        let spinner = self
            .logger
            .add_spinner(Some("Dropping local DB".to_owned()))?;

        self.db.drop_table()?;

        self.logger.finish_spinner(spinner);
        Ok(())
    }
}
