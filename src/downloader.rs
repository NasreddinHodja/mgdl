use std::collections::HashSet;
use std::sync::Arc;
use std::{fs, path::Path, path::PathBuf};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use std::time::Instant;

use crate::{
    db,
    error::MgdlResult,
    logger::{LogMode, Logger},
    models::{Chapter, ChapterRange, Manga},
    scrape,
};

#[cfg(feature = "bench")]
use crate::bench::BenchCollector;

const MAX_ATTEMPTS: usize = 20;

pub struct Downloader {
    db: db::Db,
    client: reqwest::Client,
    manga_dir: PathBuf,
    base_url: String,
    logger: Logger,
    #[cfg(feature = "bench")]
    bench: Option<BenchCollector>,
}

impl Downloader {
    pub fn new(
        manga_dir: PathBuf,
        db_dir: PathBuf,
        base_url: String,
        log_mode: LogMode,
        verbose: bool,
        client: reqwest::Client,
        #[cfg(feature = "bench")] bench: Option<BenchCollector>,
    ) -> MgdlResult<Self> {
        let db = db::Db::new(db_dir.join("mgdl.db"))?;
        let logger = Logger::new(log_mode, verbose);

        Ok(Self {
            db,
            client,
            manga_dir,
            base_url,
            logger,
            #[cfg(feature = "bench")]
            bench,
        })
    }

    pub async fn add(&self, manga_url: &str) -> MgdlResult<(Manga, Vec<Chapter>)> {
        let spinner = self
            .logger
            .add_spinner(Some("Scraping manga and chapters".to_owned()))?;

        let scrape_start = Instant::now();
        let (manga, chapters) =
            scrape::manga_from_url(&self.client, &self.base_url, manga_url, MAX_ATTEMPTS).await?;
        #[cfg(feature = "bench")]
        if let Some(bench) = &self.bench {
            bench.record_scrape(scrape_start.elapsed());
        }
        let _ = scrape_start; // suppress unused warning when bench is off

        spinner.set_message(format!("Adding manga {}", &manga.name));
        let added_manga = self.db.upsert_manga(manga)?;

        self.logger.finish_spinner(spinner);
        Ok((added_manga, chapters))
    }

    pub async fn download_manga(
        &self,
        manga_url: &str,
        chapter_range: Option<&ChapterRange>,
        force: bool,
    ) -> MgdlResult<Manga> {
        let (manga, chapters) = self.add(manga_url).await?;
        let chapters = Self::filter_by_range(chapters, chapter_range);
        let manga_path = self.manga_dir.join(&manga.normalized_name);

        let spinner = self
            .logger
            .add_spinner(Some(format!("Downloading {}", &manga.name)))?;

        self.download_chapters(&manga.name, &manga_path, &chapters, force)
            .await?;

        self.logger.finish_spinner(spinner);
        Ok(manga)
    }

    /// Download all pages for given chapters. If force=false, skip pages that already exist.
    async fn download_chapters(
        &self,
        manga_name: &str,
        manga_path: &Path,
        chapters: &[Chapter],
        force: bool,
    ) -> MgdlResult<()> {
        fs::create_dir_all(manga_path)?;

        let mut join_set: JoinSet<MgdlResult<()>> = JoinSet::new();
        let semaphore = Arc::new(Semaphore::new(16));

        let progress_bar = self.logger.add_bar(chapters.len() as u64)?;
        progress_bar.set_prefix("Locating chapters and pages".to_string());
        for chapter in chapters {
            let ch_start = Instant::now();
            let pages = scrape::get_chapter_pages(
                &self.client,
                &self.base_url,
                &chapter.hash,
                MAX_ATTEMPTS,
            )
            .await?;
            #[cfg(feature = "bench")]
            if let Some(bench) = &self.bench {
                bench.record_chapter_discovered(ch_start.elapsed());
            }
            let _ = ch_start;

            let chapter_path = manga_path.join(format!("chapter_{}", &chapter.number));

            let existing = existing_page_numbers(&chapter_path);
            let skipped_count = if force { 0 } else { existing.len() };
            let new_pages: Vec<_> = if force {
                pages
            } else {
                pages
                    .into_iter()
                    .filter(|p| !existing.contains(&p.number))
                    .collect()
            };

            #[cfg(feature = "bench")]
            if let Some(bench) = &self.bench {
                for _ in 0..skipped_count {
                    bench.record_page_skipped();
                }
            }
            let _ = skipped_count;

            if new_pages.is_empty() {
                #[cfg(feature = "bench")]
                if let Some(bench) = &self.bench {
                    bench.record_chapter_skipped();
                }
                continue;
            }

            fs::create_dir_all(&chapter_path)?;

            let page_count = new_pages.len();
            for page in new_pages {
                let chapter_path = chapter_path.clone();
                let permit = Arc::clone(&semaphore);
                let client = self.client.clone();
                #[cfg(feature = "bench")]
                let bench = self.bench.clone();
                join_set.spawn(async move {
                    let _permit = permit.acquire().await.unwrap();
                    let page_start = Instant::now();
                    let bytes = scrape::download_page(
                        &client,
                        page.url,
                        chapter_path,
                        page.number,
                        MAX_ATTEMPTS,
                    )
                    .await?;
                    #[cfg(feature = "bench")]
                    if let Some(bench) = &bench {
                        bench.record_page_downloaded(page_start.elapsed(), bytes);
                    }
                    let _ = page_start;
                    let _ = bytes;
                    Ok(())
                });
            }
            progress_bar.inc(1);
            progress_bar.success(format!(
                "Queued {} ch.{} ({} pages)",
                manga_name, &chapter.number, page_count
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

    /// Update: only download chapters that don't have a local directory yet.
    async fn update_manga(&self, manga: &Manga) -> MgdlResult<usize> {
        let manga_url = format!("{}/series/{}", &self.base_url, &manga.hash);
        let (_, chapters) =
            scrape::manga_from_url(&self.client, &self.base_url, &manga_url, MAX_ATTEMPTS).await?;
        let manga_path = self.manga_dir.join(&manga.normalized_name);

        // Filter to only chapters without a local directory
        let new_chapters: Vec<_> = chapters
            .into_iter()
            .filter(|ch| !manga_path.join(format!("chapter_{}", &ch.number)).exists())
            .collect();

        let count = new_chapters.len();
        if !new_chapters.is_empty() {
            self.download_chapters(&manga.name, &manga_path, &new_chapters, false)
                .await?;
        }

        Ok(count)
    }

    pub async fn update(&self, manga_name: &str) -> MgdlResult<()> {
        let manga = self.db.get_manga_by_normalized_name(manga_name)?;
        let spinner = self
            .logger
            .add_spinner(Some(format!("Updating {}", &manga.name)))?;
        self.update_manga(&manga).await?;
        self.logger.finish_spinner(spinner);
        Ok(())
    }

    pub async fn update_all(&self) -> MgdlResult<()> {
        let ongoing_manga = self.cleanup_missing_manga_dirs()?;

        for manga in ongoing_manga {
            let spinner = self
                .logger
                .add_spinner(Some(format!("Updating {}", &manga.name)))?;
            self.update_manga(&manga).await?;
            self.logger.finish_spinner(spinner);
        }

        Ok(())
    }

    /// Consolidate: check all chapters for missing pages and download them.
    async fn consolidate_manga(&self, manga: &Manga) -> MgdlResult<()> {
        let manga_url = format!("{}/series/{}", &self.base_url, &manga.hash);
        let (_, chapters) =
            scrape::manga_from_url(&self.client, &self.base_url, &manga_url, MAX_ATTEMPTS).await?;
        let manga_path = self.manga_dir.join(&manga.normalized_name);
        self.download_chapters(&manga.name, &manga_path, &chapters, false)
            .await?;
        Ok(())
    }

    pub async fn consolidate_all(&self) -> MgdlResult<()> {
        let ongoing_manga = self.db.get_ongoing_manga()?;

        for manga in ongoing_manga {
            let spinner = self
                .logger
                .add_spinner(Some(format!("Consolidating {}", &manga.name)))?;
            self.consolidate_manga(&manga).await?;
            self.logger.finish_spinner(spinner);
        }

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

    fn filter_by_range(chapters: Vec<Chapter>, range: Option<&ChapterRange>) -> Vec<Chapter> {
        let Some(range) = range else {
            return chapters;
        };
        chapters
            .into_iter()
            .filter(|ch| ch.major_number().is_some_and(|n| range.contains(n)))
            .collect()
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

fn existing_page_numbers(chapter_path: &Path) -> HashSet<usize> {
    let Ok(entries) = fs::read_dir(chapter_path) else {
        return HashSet::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.file_name()
                .to_str()?
                .split('.')
                .next()?
                .parse::<usize>()
                .ok()
        })
        .collect()
}
