use crate::MgdlError;
use crate::{db, db::Manga, scrape};
use std::fs;
use std::path::PathBuf;
use std::{fmt::format, process::Command};

pub struct Downloader {
    db: db::Db,
    manga_path: PathBuf,
}

type Result<T> = std::result::Result<T, MgdlError>;

impl Downloader {
    pub fn new(manga_dir: PathBuf, db_dir: PathBuf) -> Self {
        let db_path = db_dir.join(PathBuf::from("mgdl.db"));
        let db = db::Db::new(db_path);
        db.init();
        Self {
            db,
            manga_path: manga_dir,
        }
    }

    pub fn add(&self, manga_url: &str) -> Manga {
        let (manga, chapters) = scrape::manga_from_url(manga_url).unwrap();
        println!("Adding {} to DB...", manga.name);
        let added_manga = self.db.add_manga(manga, &chapters);
        added_manga
    }

    pub fn download(&self, manga_url: &str) {
        let manga = self.add(manga_url);
        let manga_path = self.manga_path.join(&manga.normalized_name);
        self.download_with_gallery_dl(manga);
        self.organize(manga_path);
    }

    pub fn download_with_gallery_dl(&self, manga: Manga) {
        let download_path = self.manga_path.join(&manga.normalized_name);
        let download_url = format!("https://weebcentral.com/series/{}/", &manga.hash);

        let output = Command::new("gallery-dl")
            .arg("-D")
            .arg(&download_path)
            .arg(&download_url)
            .output()
            .unwrap();

        if !output.status.success() {
            panic!(
                "gallery-dl Error: {}",
                String::from_utf8(output.stderr).unwrap()
            );
        }
    }

    pub fn organize(&self, dir: PathBuf) {
        for entry in fs::read_dir(&dir).unwrap() {
            let entry = entry.unwrap();
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
            let num: i32 = chapter_parts[0][1..].parse().unwrap();
            let formatted_chapter = if chapter_parts.len() == 1 {
                format!("{:04}_01", num)
            } else {
                let sub_num = chapter_parts[1].parse::<u32>().unwrap();
                format!("{:04}_{:02}", num, sub_num)
            };

            let formatted_chapter = format!("chapter_{}", formatted_chapter);

            let chapter_dir = dir.join(&formatted_chapter);
            fs::create_dir_all(&chapter_dir).unwrap();

            let new_path = chapter_dir.join(format!("{}.jpg", page_number));
            fs::rename(&file_path, &new_path).unwrap();
        }
    }
}
