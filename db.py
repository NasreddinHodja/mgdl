#!/usr/bin/env python3

import os
import sqlite3
from dataclasses import dataclass
import uuid
import os

DB_PATH = os.path.expanduser("~/.config/mgdl/mgdl.db")

@dataclass
class Chapter:
    id: str
    hash: str
    number: str
    manga: str

    @classmethod
    def create(cls, hash: str, number: str, manga: str) -> "Chapter":
        return cls(id=str(uuid.uuid4()), hash=hash, number=number, manga=manga)

    def to_tuple(self) -> tuple:
        return (self.id, self.hash, self.number, self.manga)

@dataclass
class Manga:
    id: str
    hash: str
    name: str
    normalized_name: str
    authors: str
    status: str

    @classmethod
    def create(
            cls,
            hash: str,
            name: str,
            normalized_name: str,
            authors: str,
            status: str
    ) -> "Manga":
        return cls(
            id=str(uuid.uuid4()),
            hash=hash,
            name=name,
            normalized_name=normalized_name,
            authors=authors,
            status=status
        )

    def to_tuple(self) -> tuple:
        return (
            self.id,
            self.hash,
            self.name,
            self.normalized_name,
            self.authors,
            self.status
        )

    def __repr__(self) -> str:
        return f"{self.name}:\n\tid: {self.id}\n\thash: {self.hash}\n\tnormalize_name: {self.normalized_name}\n\tauthors: {self.authors}\n\tstatus: {self.status}"

class Db:

    @staticmethod
    def init():
        Db.create()

    @staticmethod
    def create():
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.executescript("""
        create table if not exists chapters (
            id text primary key,
            hash text not null,
            number text not null,
            manga text,
            foreign key (manga) references manga(id),
            unique(number, manga)
        );
        create table if not exists mangas (
            id text primary key,
            hash text not null,
            name text not null,
            normalized_name text not null,
            authors text not null,
            status text not null,
            unique(name, authors)
        );
        """)

        conn.commit()
        conn.close()

    @staticmethod
    def drop():
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("""
        drop table if exists mangas
        drop table if exists chapters
        """)

        conn.commit()
        conn.close()

    @staticmethod
    def get_chapters() -> list[Chapter]:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("select * from chapters")
        chapters = [
            Chapter.create(*chapter)
            for chapter in cursor.fetchall()
        ]

        conn.close()
        return chapters

    @staticmethod
    def upsert_chapters(chapters: list[Chapter]) -> bool:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        chapter_tuples = [chapter.to_tuple() for chapter in chapters]
        cursor.executemany("""
            INSERT INTO chapters (id, hash, number, manga)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(number, manga)
            DO UPDATE SET hash = excluded.hash
            """,
            chapter_tuples
        )

        conn.commit()
        conn.close()
        return True

    @staticmethod
    def upsert_manga(manga: Manga) -> bool:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("""
            INSERT INTO mangas (id, hash, name, normalized_name, authors, status)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(name, authors)
            DO UPDATE SET hash = excluded.hash,
                normalized_name = excluded.normalized_name,
                status = excluded.status
            """,
            manga.to_tuple()
        )

        conn.commit()
        conn.close()
        return True


    @staticmethod
    def add_manga(manga: Manga, chapters: list[Chapter]):
            Db.upsert_manga(manga)
            Db.upsert_chapters(chapters)

    @staticmethod
    def get_chapter(id: str):
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("select * from chapters where id = ?", (id,))
        chapter = cursor.fetchone()
        chapter = Chapter(*chapter)

        conn.close()
        return chapter

    @staticmethod
    def get_manga(hash: str) -> Manga:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("select * from mangas where hash = ?", (hash,))
        manga = cursor.fetchone()
        manga = Manga(*manga)
        return manga

    @staticmethod
    def get_ongoing_manga() -> list[Manga]:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("select * from mangas where status = 'Ongoing'")
        mangas = cursor.fetchall()
        mangas = [Manga(*manga) for manga in mangas]

        conn.close()
        return mangas


    @staticmethod
    def query_manga(query: str) -> list[Manga]:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute(query)
        mangas = cursor.fetchall()
        mangas = [Manga(*manga) for manga in mangas]
        return mangas

    @staticmethod
    def update_chapter(chapter_id, new_name, new_hash) -> bool:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute(
            "UPDATE chapters SET name = ?, hash = ? WHERE id = ?",
            (new_name, new_hash, chapter_id)
        )

        conn.commit()
        conn.close()
        return True

    @staticmethod
    def delete_chapter(chapter_id) -> bool:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("DELETE FROM chapters WHERE id = ?", (chapter_id,))

        conn.commit()
        conn.close()
        return True

    @staticmethod
    def delete_manga(manga_id) -> bool:
        conn = sqlite3.connect(DB_PATH)
        cursor = conn.cursor()

        cursor.execute("DELETE FROM mangas WHERE id = ?", (manga_id,))

        conn.commit()
        conn.close()
        return True
