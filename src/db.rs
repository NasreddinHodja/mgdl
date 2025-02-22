use rusqlite::{params, Connection};
use std::fmt;
use std::path::PathBuf;

use crate::MgdlError;

#[derive(Debug)]
pub struct Chapter {
    pub hash: String,
    pub number: String,
    pub manga: String,
}

type Result<T> = std::result::Result<T, MgdlError>;

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

    pub fn create(&self) -> Result<()> {
        let mut conn = Connection::open(&self.path)?;
        let transaction = conn.transaction()?;

        transaction.execute(
            "
                create table if not exists chapters (
                    hash text not null,
                    number text not null,
                    manga text,
                    foreign key (manga) references mangas(hash),
                    unique(number, manga)
                )",
            (),
        )?;
        transaction.execute(
            "
                create table if not exists mangas (
                    hash text not null primary key,
                    name text not null unique,
                    normalized_name text unique,
                    authors text not null,
                    status text not null
                );",
            [],
        )?;

        transaction.commit()?;

        Ok(())
    }

    pub fn drop(&self) -> Result<()> {
        let mut conn = Connection::open(&self.path)?;
        let transaction = conn.transaction()?;

        transaction.execute("drop table if exists chapters", ())?;
        transaction.execute("drop table if exists mangas", ())?;

        transaction.commit()?;

        Ok(())
    }

    pub fn init(&self) -> Result<()> {
        self.create()?;

        Ok(())
    }

    pub fn upsert_chapters(&self, chapters: &[Chapter], manga_hash: &str) -> Result<()> {
        let mut conn = Connection::open(&self.path)?;
        let transaction = conn.transaction()?;

        for chapter in chapters {
            transaction.execute(
                "
                    insert into chapters (hash, number, manga)
                    values (?, ?, ?)
                    on conflict(number, manga)
                    do update set hash = excluded.hash
                    ",
                (&chapter.hash, &chapter.number, manga_hash),
            )?;
        }

        transaction.commit()?;

        Ok(())
    }

    pub fn upsert_manga(&self, manga: Manga) -> Result<Manga> {
        let conn = Connection::open(&self.path)?;
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
            return Ok(found_manga);
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
        )?;

        Ok(manga)
    }

    pub fn add_manga(&self, manga: Manga, chapters: &[Chapter]) -> Result<Manga> {
        let upserted_manga = self.upsert_manga(manga)?;

        self.upsert_chapters(chapters, &upserted_manga.hash)?;

        Ok(upserted_manga)
    }

    pub fn get_manga_by_normalized_name(&self, normalized_name: &str) -> Result<Manga> {
        let conn = Connection::open(&self.path)?;
        let mut stmt = conn.prepare("select * from mangas where normalized_name = ?")?;

        let rows = stmt
            .query_map(params![normalized_name], |row| {
                Ok(Manga {
                    hash: row.get(0)?,
                    name: row.get(1)?,
                    normalized_name: row.get(2)?,
                    authors: row.get(3)?,
                    status: row.get(4)?,
                })
            })?
            .collect::<Vec<_>>();

        for manga in rows {
            return Ok(manga?);
        }

        Err(MgdlError::Db(format!(
            "Cound't get manga by normalized_name = '{}'",
            normalized_name
        )))
    }

    pub fn get_manga_chapters(&self, manga: &Manga) -> Result<Vec<Chapter>> {
        let conn = Connection::open(&self.path)?;
        let mut stmt = conn.prepare("select * from chapters where manga = ?")?;

        let chapters = stmt
            .query_map(params![manga.hash], |row| {
                Ok(Chapter {
                    hash: row.get(0)?,
                    number: row.get(1)?,
                    manga: row.get(2)?,
                })
            })?
            .collect::<std::result::Result<Vec<Chapter>, _>>()?;

        Ok(chapters)
    }

    pub fn get_ongoing_manga(&self) -> Result<Vec<Manga>> {
        let conn = Connection::open(&self.path)?;
        let mut stmt = conn.prepare("select * from mangas where status = 'Ongoing'")?;

        let mangas: Vec<Manga> = stmt
            .query_map([], |row| {
                Ok(Manga {
                    hash: row.get(0)?,
                    name: row.get(1)?,
                    normalized_name: row.get(2)?,
                    authors: row.get(3)?,
                    status: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<Manga>, _>>()?;

        Ok(mangas)
    }
}
