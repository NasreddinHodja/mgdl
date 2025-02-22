use crate::MgdlError;
use crate::{db, db::Manga, scrape};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

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

    pub fn add(&self, manga_url: &str) -> Result<Manga> {
        println!("Getting manga data from {manga_url}...");
        let (manga, chapters) = scrape::manga_from_url(manga_url)?;

        println!("adding {} to db...", manga.name);
        let added_manga = self.db.add_manga(manga, &chapters)?;

        Ok(added_manga)
    }

    pub fn download(&self, manga_url: &str) -> Result<Manga> {
        let manga = self.add(manga_url)?;

        println!("Downloading {}...", &manga.name);
        let manga_dir = self.manga_dir.join(&manga.normalized_name);
        self.download_with_gallery_dl(&manga, None)?;

        self.organize(&manga_dir)?;

        Ok(manga)
    }

    pub fn update(&self, manga_name: &str) -> Result<Manga> {
        let manga = self.db.get_manga_by_normalized_name(manga_name)?;
        let skip_chaps = self.skip_chaps(&manga)?;

        self.download_with_gallery_dl(&manga, Some(skip_chaps))?;

        self.organize(&self.manga_dir.join(&manga.normalized_name))?;

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
            "Couldn't find manga's name".to_string(),
        ))?))
    }

    pub fn download_with_gallery_dl(&self, manga: &Manga, skip_chaps: Option<usize>) -> Result<()> {
        let download_path = self.manga_dir.join(&manga.normalized_name);
        let download_url = format!("https://weebcentral.com/series/{}/", &manga.hash);

        let mut cmd = Command::new("gallery-dl");
        cmd.arg("-D").arg(&download_path);
        if let Some(skip) = skip_chaps {
            cmd.arg("--chapter-filter")
                .arg(format!("{} < chapter", skip));
        }

        let mut child = cmd
            .arg(&download_url)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let status = child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(MgdlError::Downloader(format!(
                "Downloader failed with status = {}",
                status
            )))
        }
    }

    pub fn organize(&self, dir: &PathBuf) -> Result<()> {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let file_path = entry.path();
            let file_name = entry.file_name();
            let file_name = file_name.to_string_lossy();

            if !file_name.contains("_c") || !file_name.ends_with(".jpg") {
                continue;
            }

            let chapter_and_page = file_name.trim_end_matches(".jpg");
            let parts: Vec<&str> = chapter_and_page.split('_').collect();

            if parts.len() < 3 {
                continue;
            }

            let chapter_number = parts[1];
            let page_number = parts[2];

            let chapter_parts: Vec<&str> = chapter_number.split('.').collect();
            let num: i32 = chapter_parts[0][1..].parse().map_err(|err| {
                MgdlError::Downloader(format!("Cound't parse chapter directory name: {}", err))
            })?;
            let formatted_chapter = if chapter_parts.len() == 1 {
                format!("{:04}-01", num)
            } else {
                let sub_num = chapter_parts[1].parse::<u32>()?;
                format!("{:04}-{:02}", num, sub_num)
            };

            let formatted_chapter = format!("chapter_{}", formatted_chapter);

            let chapter_dir = dir.join(&formatted_chapter);
            fs::create_dir_all(&chapter_dir)?;

            let new_path = chapter_dir.join(format!("{}.jpg", page_number));
            fs::rename(&file_path, &new_path)?;
        }

        Ok(())
    }

    pub fn reset_db(&self) -> Result<()> {
        self.db.drop()?;
        Ok(())
    }
}
