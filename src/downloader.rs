use crate::MgdlError;
use crate::{db, scrape, Chapter, Manga};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::PathBuf;
use tokio::task::JoinSet;

type Result<T> = std::result::Result<T, MgdlError>;

const MAX_ATTEMPTS: usize = 20;

pub struct Downloader {
    db: db::Db,
    manga_dir: PathBuf,
}

impl Downloader {
    pub fn new(manga_dir: PathBuf, db_dir: PathBuf) -> Result<Self> {
        let db_path = db_dir.join("mgdl.db");
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

        println!("Downloading {}", &manga.name);
        let manga_path = self.manga_dir.join(format!("{}", &manga.normalized_name));

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

        let mut join_set = JoinSet::new();

        let bar_style_generator = || -> Result<ProgressStyle> {
            ProgressStyle::with_template("{prefix} {elapsed_precise} {wide_bar} {pos}/{len}")
                .map_err(|_| MgdlError::Scrape("Could not create progress bar style".to_string()))
        };
        let progress_bar = ProgressBar::new(chapters.len() as u64)
            .with_prefix(format!("Locating chapters and pages"));
        progress_bar.set_style(bar_style_generator()?);
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
                let page_number = page.number.clone();
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

        let progress_bar =
            ProgressBar::new(join_set.len() as u64).with_prefix(format!("Downloading pages"));
        progress_bar.set_style(bar_style_generator()?);
        while let Some(res) = join_set.join_next().await {
            let _ = res?;
            progress_bar.inc(1);
        }
        progress_bar.finish_and_clear();

        Ok(())
    }

    pub async fn update(&self, manga_name: &str) -> Result<Manga> {
        let manga = self.db.get_manga_by_normalized_name(manga_name)?;
        let manga_url = format!("https://weebcentral.com/series/{}", &manga.hash);
        let (new_manga, chapters) = scrape::manga_from_url(&manga_url).await?;
        let skip_chaps = self.skip_chaps(&new_manga)?;
        let manga_path = self
            .manga_dir
            .join(format!("{}", &new_manga.normalized_name));

        self.download_chapters(&manga_path, &chapters, Some(skip_chaps))
            .await?;

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
        self.db.drop()?;
        Ok(())
    }
}
