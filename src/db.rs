use rusqlite::{params, Connection};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Chapter {
    pub hash: String,
    pub number: String,
    pub manga: String,
}

impl Chapter {
    pub fn new(hash: &str, number: &str, manga: &str) -> Self {
        Self {
            hash: hash.to_string(),
            number: number.to_string(),
            manga: manga.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Manga {
    pub hash: String,
    pub name: String,
    pub normalized_name: String,
    pub authors: String,
    pub status: String,
}

impl Manga {
    pub fn new(hash: &str, name: &str, normalized_name: &str, authors: &str, status: &str) -> Self {
        Self {
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
            "{}:\n\thash: {}\n\tnormalize_name: {}\n\tauthors: {}\n\tstatus: {}",
            self.name, self.hash, self.normalized_name, self.authors, self.status
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
                    hash text not null,
                    number text not null,
                    manga text,
                    foreign key (manga) references mangas(hash),
                    unique(number, manga)
                )",
                (),
            )
            .unwrap();
        transaction
            .execute(
                "
                create table if not exists mangas (
                    hash text not null primary key,
                    name text not null unique,
                    normalized_name text unique,
                    authors text not null,
                    status text not null
                );",
                [],
            )
            .unwrap();

        transaction.commit().unwrap();
    }

    pub fn drop(&self) {
        let mut conn = Connection::open(&self.path).unwrap();
        let transaction = conn.transaction().unwrap();

        transaction
            .execute("drop table if exists chapters", ())
            .unwrap();
        transaction
            .execute(" drop table if exists mangas", ())
            .unwrap();

        transaction.commit().unwrap();
    }

    pub fn init(&self) {
        self.create();
    }

    pub fn upsert_chapters(&self, chapters: &[Chapter], manga_hash: &str) {
        let mut conn = Connection::open(&self.path).unwrap();
        let transaction = conn.transaction().unwrap();

        for chapter in chapters {
            transaction
                .execute(
                    "
                    insert into chapters (hash, number, manga)
                    values (?, ?, ?)
                    on conflict(number, manga)
                    do update set hash = excluded.hash
                    ",
                    (&chapter.hash, &chapter.number, manga_hash),
                )
                .unwrap();
        }

        transaction.commit().unwrap();
    }

    pub fn upsert_manga(&self, manga: Manga) -> Manga {
        let conn = Connection::open(&self.path).unwrap();
        let existing_manga = conn.query_row(
            "SELECT hash, name, normalized_name, authors, status
            FROM mangas
            WHERE name = ? AND authors = ?",
            (&manga.name, &manga.authors),
            |row| {
                Ok(Manga {
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
            insert into mangas (hash, name, normalized_name, authors, status)
            values (?, ?, ?, ?, ?)
            on conflict(name)
            do update set hash = excluded.hash,
                normalized_name = excluded.normalized_name,
                status = excluded.status
            ",
            (
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
        self.upsert_chapters(chapters, &upserted_manga.hash);
        upserted_manga
    }

    pub fn get_manga_by_normalized_name(&self, normalized_name: &str) -> Option<Manga> {
        let conn = Connection::open(&self.path).unwrap();
        let mut stmt = conn
            .prepare("select * from mangas where normalized_name = ?")
            .unwrap();
        let rows = stmt
            .query_map(params![normalized_name], |row| {
                Ok(Manga {
                    hash: row.get(0).unwrap(),
                    name: row.get(1).unwrap(),
                    normalized_name: row.get(2).unwrap(),
                    authors: row.get(3).unwrap(),
                    status: row.get(4).unwrap(),
                })
            })
            .unwrap()
            .collect::<Vec<_>>();

        for manga in rows {
            return Some(manga.unwrap());
        }

        None
    }

    pub fn query_mangas(&self, query: &str) {
        let conn = Connection::open(&self.path).unwrap();
        let mut stmt = conn.prepare(query).unwrap();
        let rows = stmt
            .query_map([], |row| {
                Ok(Manga {
                    hash: row.get(1).unwrap(),
                    name: row.get(2).unwrap(),
                    normalized_name: row.get(3).unwrap(),
                    authors: row.get(4).unwrap(),
                    status: row.get(5).unwrap(),
                })
            })
            .unwrap()
            .collect::<Vec<_>>();
        println!("result = {rows:#?}");
    }
}
