use std::{fs, path::PathBuf};
use indicatif::MultiProgress;
use tokio::task::JoinSet;

use crate::{
    db, scrape,
    utils::{gen_progress_bar, gen_progress_spinner},
    Chapter, Manga, MgdlError,
};

type Result<T> = std::result::Result<T, MgdlError>;

const MAX_ATTEMPTS: usize = 20;

pub struct Downloader {
    db: db::Db,
    manga_dir: PathBuf,
    progress: MultiProgress,
}

impl Downloader {
    pub fn new(manga_dir: PathBuf, db_dir: PathBuf) -> Result<Self> {
        let db_path = db_dir.join("mgdl.db");
        let db = db::Db::new(db_path);
        db.init()?;

        let progress = MultiProgress::new();

        Ok(Self {
            db,
            manga_dir: manga_dir,
            progress,
        })
    }

    pub async fn add(&self, manga_url: &str) -> Result<(Manga, Vec<Chapter>)> {
        let spinner = self.progress.add(gen_progress_spinner()?);

        spinner.set_message("Scraping manga and chapters");
        let (manga, chapters) = scrape::manga_from_url(manga_url, MAX_ATTEMPTS).await?;

        spinner.set_message(format!("Adding manga {}", &manga.name));
        let added_manga = self.db.add_manga(manga)?;

        spinner.finish_and_clear();
        self.progress.remove(&spinner);
        Ok((added_manga, chapters))
    }

    pub async fn download_manga(&self, manga_url: &str) -> Result<Manga> {
        let (manga, chapters) = self.add(manga_url).await?;
        let manga_path = self.manga_dir.join(format!("{}", &manga.normalized_name));

        let spinner = self.progress.add(gen_progress_spinner()?);
        spinner.set_message(format!("Downloading {}", &manga.name));

        self.download_chapters(&manga_path, &chapters, None).await?;

        spinner.finish_and_clear();
        self.progress.remove(&spinner);
        Ok(manga)
    }

    pub async fn download_chapters(
        &self,
        manga_path: &PathBuf,
        chapters: &Vec<Chapter>,
        skip_chaps: Option<usize>,
    ) -> Result<()> {
        fs::create_dir_all(&manga_path)?;

        let mut join_set = JoinSet::new();

        let progress_bar = self.progress.add(gen_progress_bar(chapters.len() as u64)?);
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

            if skip_chaps.map_or(false, |skip| chapter_number <= skip) {
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
        }
        progress_bar.finish_and_clear();
        self.progress.remove(&progress_bar);

        let progress_bar = self.progress.add(gen_progress_bar(join_set.len() as u64)?);
        progress_bar.set_prefix(format!("Downloading pages"));
        while let Some(res) = join_set.join_next().await {
            let _ = res?;
            progress_bar.inc(1);
        }
        progress_bar.finish_and_clear();
        self.progress.remove(&progress_bar);

        Ok(())
    }

    pub async fn update(&self, manga_name: &str) -> Result<Manga> {
        let spinner = self.progress.add(gen_progress_spinner()?);
        spinner.set_message("Getting local manga data");

        let manga = self.db.get_manga_by_normalized_name(manga_name)?;
        let manga_url = format!("https://weebcentral.com/series/{}", &manga.hash);

        spinner.set_message("Scraping manga and chapters");
        let (new_manga, chapters) = scrape::manga_from_url(&manga_url, MAX_ATTEMPTS).await?;
        let skip_chaps = self.skip_chaps(&new_manga)?;
        let manga_path = self
            .manga_dir
            .join(format!("{}", &new_manga.normalized_name));

        spinner.set_message(format!("Downloading {}", &manga.name));
        self.download_chapters(&manga_path, &chapters, Some(skip_chaps))
            .await?;

        spinner.finish_and_clear();
        self.progress.remove(&spinner);
        Ok(new_manga)
    }

    pub async fn update_all(&self) -> Result<()> {
        let spinner = self.progress.add(gen_progress_spinner()?);
        for manga in self.db.get_ongoing_manga()? {
            spinner.set_message(format!("Trying to update {}", &manga.name));
            self.update(&manga.normalized_name).await?;
        }

        spinner.finish_and_clear();
        self.progress.remove(&spinner);
        Ok(())
    }

    pub fn skip_chaps(&self, manga: &Manga) -> Result<usize> {
        let manga_path = self.manga_dir.join(&manga.normalized_name);
        let max_chapter: usize = fs::read_dir(&manga_path)?
            .filter_map(std::result::Result::ok)
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter(|file_name| file_name.contains("chapter"))
            .filter_map(|chapter_name| chapter_name.split('_').nth(1).map(|s| s.to_string()))
            .filter_map(|chapter_number| chapter_number.split("-").next()?.parse::<usize>().ok())
            .max()
            .ok_or(MgdlError::Downloader(
                "Manga directory is empty".to_string(),
            ))?;

        Ok(max_chapter)
    }

    pub fn reset_db(&self) -> Result<()> {
        let spinner = self.progress.add(gen_progress_spinner()?);

        spinner.set_message("Dropping local DB");
        self.db.drop()?;

        spinner.finish_and_clear();
        self.progress.remove(&spinner);
        Ok(())
    }
}
