use indicatif::MultiProgress;
use std::{fs, path::PathBuf};
use tokio::task::JoinSet;

use crate::{
    db,
    maybe_progress::{MaybeBar, MaybeSpinner},
    scrape, Chapter, Manga, MgdlError,
};

type Result<T> = std::result::Result<T, MgdlError>;

const MAX_ATTEMPTS: usize = 20;

pub struct Downloader {
    db: db::Db,
    manga_dir: PathBuf,
    progress: Option<MultiProgress>,
}

impl Downloader {
    pub fn new(manga_dir: PathBuf, db_dir: PathBuf, plain: bool) -> Result<Self> {
        let db_path = db_dir.join("mgdl.db");
        let db = db::Db::new(db_path);
        db.init()?;

        let mut progress = None;
        if !plain {
            progress = Some(MultiProgress::new());
        }

        Ok(Self {
            db,
            manga_dir: manga_dir,
            progress,
        })
    }

    pub async fn add(&self, manga_url: &str) -> Result<(Manga, Vec<Chapter>)> {
        let spinner = MaybeSpinner::new(
            self.progress.as_ref(),
            Some("Scraping manga and chapters".to_owned()),
        )?;

        let (manga, chapters) = scrape::manga_from_url(manga_url, MAX_ATTEMPTS).await?;

        spinner.set_message(format!("Adding manga {}", &manga.name));
        let added_manga = self.db.add_manga(manga)?;

        spinner.finish_and_clear(self.progress.as_ref());
        Ok((added_manga, chapters))
    }

    pub async fn download_manga(&self, manga_url: &str) -> Result<Manga> {
        let (manga, chapters) = self.add(manga_url).await?;
        let manga_path = self.manga_dir.join(format!("{}", &manga.normalized_name));

        let spinner = MaybeSpinner::new(
            self.progress.as_ref(),
            Some(format!("Downloading {}", &manga.name)),
        )?;

        self.download_chapters(&manga_path, &manga.name, &chapters, None)
            .await?;

        spinner.finish_and_clear(self.progress.as_ref());
        Ok(manga)
    }

    pub async fn download_chapters(
        &self,
        manga_path: &PathBuf,
        manga_name: &str,
        chapters: &Vec<Chapter>,
        skip_chaps: Option<&[usize]>,
    ) -> Result<()> {
        fs::create_dir_all(&manga_path)?;

        let mut join_set = JoinSet::new();

        let progress_bar = MaybeBar::new(self.progress.as_ref(), chapters.len() as u64)?;
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
            progress_bar.println(format!(
                "Downloaded \"{}\" - Chapter {}",
                manga_name, &chapter.number
            ));
        }
        progress_bar.finish_and_clear(self.progress.as_ref());

        let progress_bar = MaybeBar::new(self.progress.as_ref(), join_set.len() as u64)?;
        progress_bar.set_prefix(format!("Downloading pages"));
        while let Some(res) = join_set.join_next().await {
            let _ = res?;
            progress_bar.inc(1);
        }
        progress_bar.finish_and_clear(self.progress.as_ref());

        Ok(())
    }

    pub async fn update(&self, manga_name: &str) -> Result<Manga> {
        let spinner = MaybeSpinner::new(
            self.progress.as_ref(),
            Some("Getting local manga data".to_owned()),
        )?;

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

        spinner.finish_and_clear(self.progress.as_ref());
        Ok(new_manga)
    }

    pub async fn update_all(&self) -> Result<()> {
        let spinner = MaybeSpinner::new(self.progress.as_ref(), None)?;
        for manga in self.db.get_ongoing_manga()? {
            spinner.set_message(format!("Trying to update {}", &manga.name));
            self.update(&manga.normalized_name).await?;
        }

        spinner.finish_and_clear(self.progress.as_ref());
        Ok(())
    }

    pub fn skip_chaps(&self, manga: &Manga) -> Result<Vec<usize>> {
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

    pub fn reset_db(&self) -> Result<()> {
        let spinner =
            MaybeSpinner::new(self.progress.as_ref(), Some("Dropping local DB".to_owned()))?;

        self.db.drop()?;

        spinner.finish_and_clear(self.progress.as_ref());
        Ok(())
    }
}
