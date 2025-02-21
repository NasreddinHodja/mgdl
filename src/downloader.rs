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
    pub fn new(manga_dir: PathBuf, db_dir: PathBuf) -> Self {
        let db_path = db_dir.join(PathBuf::from("mgdl.db"));
        let db = db::Db::new(db_path);
        db.init();

        Self {
            db,
            manga_dir: manga_dir,
        }
    }

    pub fn add(&self, manga_url: &str) -> Manga {
        println!("Getting manga data from {manga_url}...");
        let (manga, chapters) = scrape::manga_from_url(manga_url).unwrap();

        println!("adding {} to db...", manga.name);
        let added_manga = self.db.add_manga(manga, &chapters);

        added_manga
    }

    pub fn download(&self, manga_url: &str) {
        let manga = self.add(manga_url);

        println!("Downloading {manga.name}...");
        let manga_dir = self.manga_dir.join(&manga.normalized_name);
        self.download_with_gallery_dl(&manga, None);

        self.organize(&manga_dir);
    }

    pub fn update(&self, manga_name: &str) -> Result<Manga> {
        if let Some(manga) = self.db.get_manga_by_normalized_name(manga_name) {
            let skip_chaps = self.skip_chaps(&manga);

            self.download_with_gallery_dl(&manga, Some(skip_chaps));

            self.organize(&self.manga_dir.join(&manga.normalized_name));

            Ok(manga)
        } else {
            Err(MgdlError::Db(format!("couldn't find manga: {}", manga_name)))
        }
    }

    pub fn skip_chaps(&self, manga: &Manga) -> usize {
        let manga_path = self.manga_dir.join(&manga.normalized_name);
        let mut chaps: Vec<usize> = fs::read_dir(&manga_path)
            .unwrap()
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

        *(chaps.iter().max().unwrap())
    }

    pub fn download_with_gallery_dl(&self, manga: &Manga, skip_chaps: Option<usize>) {
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
            .spawn()
            .unwrap();

        let status = child.wait().unwrap();
        if !status.success() {
            panic!("gallery-dl Error: {:?}", status.code());
        }
    }

    pub fn organize(&self, dir: &PathBuf) {
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
                format!("{:04}-01", num)
            } else {
                let sub_num = chapter_parts[1].parse::<u32>().unwrap();
                format!("{:04}-{:02}", num, sub_num)
            };

            let formatted_chapter = format!("chapter_{}", formatted_chapter);

            let chapter_dir = dir.join(&formatted_chapter);
            fs::create_dir_all(&chapter_dir).unwrap();

            let new_path = chapter_dir.join(format!("{}.jpg", page_number));
            fs::rename(&file_path, &new_path).unwrap();
        }
    }

    pub fn reset_db(&self) {
        self.db.drop();
    }
}
