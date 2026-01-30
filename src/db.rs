use rusqlite::{params, Connection};
use std::path::PathBuf;

use crate::{
    error::{MgdlError, MgdlResult},
    models::Manga,
};

pub struct Db {
    conn: Connection,
}

fn manga_from_row(row: &rusqlite::Row) -> rusqlite::Result<Manga> {
    Ok(Manga {
        hash: row.get(0)?,
        name: row.get(1)?,
        normalized_name: row.get(2)?,
        authors: row.get(3)?,
        status: row.get(4)?,
    })
}

impl Db {
    pub fn new(path: PathBuf) -> MgdlResult<Self> {
        let conn = Connection::open(&path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mangas (
                hash TEXT NOT NULL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                normalized_name TEXT UNIQUE,
                authors TEXT NOT NULL,
                status TEXT NOT NULL
            )",
            [],
        )?;
        Ok(Self { conn })
    }

    pub fn drop_table(&self) -> MgdlResult<()> {
        self.conn.execute("DROP TABLE IF EXISTS mangas", [])?;
        Ok(())
    }

    pub fn upsert_manga(&self, manga: Manga) -> MgdlResult<Manga> {
        let existing = self.conn.query_row(
            "SELECT hash, name, normalized_name, authors, status
             FROM mangas WHERE name = ?",
            params![manga.name],
            manga_from_row,
        );

        if let Ok(found) = existing {
            return Ok(found);
        }

        self.conn.execute(
            "INSERT INTO mangas (hash, name, normalized_name, authors, status)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(name) DO UPDATE SET
                hash = excluded.hash,
                normalized_name = excluded.normalized_name,
                status = excluded.status",
            params![
                manga.hash,
                manga.name,
                manga.normalized_name,
                manga.authors,
                manga.status,
            ],
        )?;

        Ok(manga)
    }

    pub fn get_manga_by_normalized_name(&self, normalized_name: &str) -> MgdlResult<Manga> {
        self.conn
            .query_row(
                "SELECT hash, name, normalized_name, authors, status
                 FROM mangas WHERE normalized_name = ?",
                params![normalized_name],
                manga_from_row,
            )
            .map_err(|_| {
                MgdlError::Db(format!(
                    "Couldn't get manga by normalized_name = '{}'",
                    normalized_name
                ))
            })
    }

    pub fn get_ongoing_manga(&self) -> MgdlResult<Vec<Manga>> {
        let mut stmt = self
            .conn
            .prepare("SELECT hash, name, normalized_name, authors, status FROM mangas WHERE status = 'Ongoing'")?;

        let mangas = stmt
            .query_map([], manga_from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(mangas)
    }

    pub fn delete_manga_by_normalized_name(&self, normalized_name: &str) -> MgdlResult<()> {
        self.conn.execute(
            "DELETE FROM mangas WHERE normalized_name = ?",
            params![normalized_name],
        )?;
        Ok(())
    }
}
