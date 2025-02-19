use rusqlite::{Connection};
use std::fmt;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug)]
pub struct Chapter {
    pub id: String,
    pub hash: String,
    pub number: String,
    pub manga: String,
}

impl Chapter {
    pub fn new(hash: &str, number: &str, manga: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            hash: hash.to_string(),
            number: number.to_string(),
            manga: manga.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Manga {
    pub id: String,
    pub hash: String,
    pub name: String,
    pub normalized_name: String,
    pub authors: String,
    pub status: String,
}

impl Manga {
    pub fn new(hash: &str, name: &str, normalized_name: &str, authors: &str, status: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            hash: hash.to_string(),
            name: name.to_string(),
            normalized_name: normalized_name.to_string(),
            authors: authors.to_string(),
            status: status.to_string(),
        }
    }
}

impl fmt::Display for Manga {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:\n\tid: {}\n\thash: {}\n\tnormalize_name: {}\n\tauthors: {}\n\tstatus: {}",
            self.name, self.id, self.hash, self.normalized_name, self.authors, self.status
        )
    }
}

pub struct Db {
    path: PathBuf,
}

impl Db {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn create(&self) {
        let mut conn = Connection::open(&self.path).unwrap();
        let transaction = conn.transaction().unwrap();

        transaction
            .execute(
                "
                create table if not exists chapters (
                    id text primary key,
                    hash text not null,
                    number text not null,
                    manga text,
                    foreign key (manga) references mangas(id),
                    unique(number, manga)
                )",
                (),
            )
            .unwrap();
        transaction
            .execute(
                "
                create table if not exists mangas (
                    id text primary key,
                    hash text not null,
                    name text not null,
                    normalized_name text not null,
                    authors text not null,
                    status text not null,
                    unique(name, authors)
                );",
                [],
            )
            .unwrap();

        transaction.commit().unwrap();
    }

    pub fn init(&self) {
        self.create();
    }

    pub fn upsert_chapters(&self, chapters: &[Chapter], manga_id: &str) {
        let mut conn = Connection::open(&self.path).unwrap();
        let transaction = conn.transaction().unwrap();

        for chapter in chapters {
            transaction
                .execute(
                    "
                    insert into chapters (id, hash, number, manga)
                    values (?, ?, ?, ?)
                    on conflict(number, manga)
                    do update set hash = excluded.hash
                    ",
                    (&chapter.id, &chapter.hash, &chapter.number, &manga_id),
                )
                .unwrap();
        }

        transaction.commit().unwrap();
    }

    pub fn upsert_manga(&self, manga: Manga) -> Manga {
        let conn = Connection::open(&self.path).unwrap();
        let existing_manga = conn.query_row(
            "SELECT id, hash, name, normalized_name, authors, status
            FROM mangas
            WHERE name = ? AND authors = ?",
            (&manga.name, &manga.authors),
            |row| {
                Ok(Manga {
                    id: row.get(0)?,
                    hash: row.get(1)?,
                    name: row.get(2)?,
                    normalized_name: row.get(3)?,
                    authors: row.get(4)?,
                    status: row.get(5)?,
                })
            },
        );

        if let Ok(found_manga) = existing_manga {
            return found_manga;
        }

        conn.execute(
            "
            insert into mangas (id, hash, name, normalized_name, authors, status)
            values (?, ?, ?, ?, ?, ?)
            on conflict(name, authors)
            do update set hash = excluded.hash,
                normalized_name = excluded.normalized_name,
                status = excluded.status
            ",
            (
                manga.id.to_string(),
                manga.hash.to_string(),
                manga.name.to_string(),
                manga.normalized_name.to_string(),
                manga.authors.to_string(),
                manga.status.to_string(),
            ),
        )
        .unwrap();

        manga
    }

    pub fn add_manga(&self, manga: Manga, chapters: &[Chapter]) -> Manga {
        let upserted_manga = self.upsert_manga(manga);
        self.upsert_chapters(chapters, &upserted_manga.id);
        upserted_manga
    }

    // pub fn print_mangas(&self) {
    //     let conn = Connection::open(&self.path).unwrap();
    //     let mut stmt = conn.prepare("select * from mangas").unwrap();
    //     let rows = stmt
    //         .query_map([], |row| {
    //             Ok(Manga {
    //                 id: row.get(0).unwrap(),
    //                 hash: row.get(1).unwrap(),
    //                 name: row.get(2).unwrap(),
    //                 normalized_name: row.get(3).unwrap(),
    //                 authors: row.get(4).unwrap(),
    //                 status: row.get(5).unwrap(),
    //             })
    //         })
    //         .unwrap()
    //         .collect::<Vec<_>>();
    //     println!("rows = {rows:#?}");
    // }
}
