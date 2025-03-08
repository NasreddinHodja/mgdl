use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::{Manga, MgdlError};

type Result<T> = std::result::Result<T, MgdlError>;

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

        transaction.execute("drop table if exists mangas", ())?;

        transaction.commit()?;

        Ok(())
    }

    pub fn init(&self) -> Result<()> {
        self.create()?;

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

    pub fn add_manga(&self, manga: Manga) -> Result<Manga> {
        let upserted_manga = self.upsert_manga(manga)?;

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
