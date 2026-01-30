use std::{fs, path::PathBuf};
use tokio::task::JoinSet;

use crate::{
    db,
    logger::{LogMode, Logger},
    scrape, Chapter, Manga, MgdlError, MgdlResult,
};

const MAX_ATTEMPTS: usize = 20;

pub struct Downloader {
    db: db::Db,
    manga_dir: PathBuf,
    logger: Logger,
}

impl Downloader {
    pub fn new(manga_dir: PathBuf, db_dir: PathBuf, log_mode: LogMode) -> MgdlResult<Self> {
        let db_path = db_dir.join("mgdl.db");
        let db = db::Db::new(db_path);

        db.init()?;

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
        let added_manga = self.db.add_manga(manga)?;

        self.logger.finish_spinner(spinner);
        Ok((added_manga, chapters))
    }

    pub async fn download_manga(&self, manga_url: &str) -> MgdlResult<Manga> {
        let (manga, chapters) = self.add(manga_url).await?;
        let manga_path = self.manga_dir.join(format!("{}", &manga.normalized_name));

        let spinner = self
            .logger
            .add_spinner(Some(format!("Downloading {}", &manga.name)))?;

        self.download_chapters(&manga_path, &manga.name, &chapters, None)
            .await?;

        self.logger.finish_spinner(spinner);
        Ok(manga)
    }

    pub async fn download_chapters(
        &self,
        manga_path: &PathBuf,
        manga_name: &str,
        chapters: &Vec<Chapter>,
        skip_chaps: Option<&[usize]>,
    ) -> MgdlResult<()> {
        fs::create_dir_all(&manga_path)?;

        let mut join_set = JoinSet::new();

        let progress_bar = self.logger.add_bar(chapters.len() as u64)?;
        progress_bar.set_prefix(format!("Locating chapters and pages"));
        for chapter in chapters {
            let chapter_number = chapter
                .number
                .split('-')
                .next()
                .ok_or(MgdlError::Downloader(
                    "Could not find manga's name".to_string(),
                ))?
                .parse::<usize>()?;

            if skip_chaps.map_or(false, |skips| skips.contains(&chapter_number)) {
                continue;
            }

            let pages = scrape::get_chapter_pages(&chapter.hash, MAX_ATTEMPTS).await?;
            let chapter_path = manga_path.join(format!("chapter_{}", &chapter.number));

            fs::create_dir_all(&chapter_path)?;

            for page in pages {
                let chapter_path = chapter_path.clone();
                let page_url = page.url.clone();
                let page_number = page.number;

                join_set.spawn(scrape::download_page(
                    page_url,
                    chapter_path,
                    page_number,
                    MAX_ATTEMPTS,
                ));
            }
            progress_bar.inc(1);
            progress_bar.success(format!(
                "Downloaded \"{}\" - Chapter {}",
                manga_name, &chapter.number
            ));
        }
        self.logger.finish_bar(progress_bar);

        let progress_bar = self.logger.add_bar(join_set.len() as u64)?;
        progress_bar.set_prefix(format!("Downloading pages"));
        while let Some(res) = join_set.join_next().await {
            let _ = res?;
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
        let skip_chaps = self.skip_chaps(&new_manga)?;
        let manga_path = self
            .manga_dir
            .join(format!("{}", &new_manga.normalized_name));

        spinner.set_message(format!("Downloading {}", &manga.name));
        self.download_chapters(&manga_path, &manga.name, &chapters, Some(&skip_chaps))
            .await?;

        self.logger.finish_spinner(spinner);
        Ok(new_manga)
    }

    pub async fn update_all(&self) -> MgdlResult<()> {
        let spinner = self.logger.add_spinner(None)?;

        spinner.set_message("Cleaning up missing manga directories".to_owned());
        self.cleanup_missing_manga_dirs()?;

        for manga in self.db.get_ongoing_manga()? {
            spinner.set_message(format!("Trying to update {}", &manga.name));
            self.update(&manga.normalized_name).await?;
        }

        self.logger.finish_spinner(spinner);
        Ok(())
    }

    pub fn cleanup_missing_manga_dirs(&self) -> MgdlResult<()> {
        let ongoing_manga = self.db.get_ongoing_manga()?;
        let mut deleted_count = 0;

        for manga in ongoing_manga {
            let manga_path = self.manga_dir.join(&manga.normalized_name);
            if !manga_path.exists() {
                self.db
                    .delete_manga_by_normalized_name(&manga.normalized_name)?;
                deleted_count += 1;
            }
        }

        if deleted_count > 0 {
            eprintln!(
                "Cleaned up {} manga entries with missing directories",
                deleted_count
            );
        }

        Ok(())
    }

    pub fn skip_chaps(&self, manga: &Manga) -> MgdlResult<Vec<usize>> {
        let manga_path = self.manga_dir.join(&manga.normalized_name);
        let existing_chaps: Vec<usize> = fs::read_dir(&manga_path)?
            .filter_map(std::result::Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|file_name| file_name.contains("chapter"))
            .filter_map(|chapter_name| chapter_name.split('_').nth(1).map(|s| s.to_string()))
            .filter_map(|chapter_number| chapter_number.split("-").next()?.parse::<usize>().ok())
            .collect();

        Ok(existing_chaps)
    }

    pub fn reset_db(&self) -> MgdlResult<()> {
        let spinner = self
            .logger
            .add_spinner(Some("Dropping local DB".to_owned()))?;

        self.db.drop()?;

        self.logger.finish_spinner(spinner);
        Ok(())
    }
}
