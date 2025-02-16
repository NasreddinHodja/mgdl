import os
import shutil
import scraper
from db import Db, Manga

class Downloader:
    def __init__(self, local_dir=None):
        Db.init()
        self.local_dir = local_dir

    def organize(self, manga_dir):
        os.chdir(f"{self.local_dir}{manga_dir}")

        dir_contents = os.listdir()

        for page_name in dir_contents:
            if "_c" not in page_name : continue

            chapter_and_page = page_name[:-4]

            [_, chapter_number, page_number] = chapter_and_page.split("_")
            chapter_number = chapter_number.split('.')
            if len(chapter_number) == 1:
                chapter_number += ['1'.zfill(2)]
            chapter_number[0] = chapter_number[0][1:].zfill(4)
            chapter_number = '_'.join(chapter_number)
            chapter_dir = f"chapter_{chapter_number}"

            if chapter_dir not in os.listdir():
                os.mkdir(chapter_dir)

            shutil.move(page_name, f"{chapter_dir}/{page_number}.jpg")

        os.chdir('..')

    def add(self, manga_url: str):
        (manga, chapters) = scraper.manga_from_url(manga_url)
        Db.add_manga(manga, chapters)

    def download(self, manga_url: str):
        self.add(manga_url)

        manga = Db.get_manga(manga_url.split("/")[-2])

        if manga is not None:
            download_cmd = (
                f"gallery-dl " +
                f"-D {self.local_dir}{manga.normalized_name} {manga_url}"
            )
            os.system(download_cmd)
            self.organize(manga.normalized_name)

        else:
            raise Exception("Couldn't find manga")

    def get_updatables(self) -> list[Manga]:
        return Db.get_ongoing_manga()

    def skip_chaps(self, manga: Manga):
        chaps = set([
            c.split("_")[1].split("-")[0]
            for c in os.listdir(manga.normalized_name)
            if ("chapter" in c)
        ])
        if len(chaps) == 0: chaps = [0]
        return str(max(chaps)).lstrip("0")

    def update(self, manga_name: str):
        result = Db.query_manga(f"select * from mangas where normalized_name = '{manga_name}' limit 1")
        if len(result) == 0:
            raise Exception("Couldn't find manga in DB")

        manga = result[0]
        download_cmd = (
            f"gallery-dl " +
            f"-D {self.local_dir}{manga.normalized_name} "
            f"--chapter-filter '{self.skip_chaps(manga)} < chapter' "
            f"https://weebcentral.com/series/{manga.hash}"
        )
        os.system(download_cmd)
        self.organize(manga.normalized_name)

    def remove(self, manga_name: str):
        result = Db.query_manga(f"select * from mangas where normalized_name = '{manga_name}' limit 1")
        if len(result) == 0:
            raise Exception("Couldn't find manga in DB")

        manga = result[0]
        Db.delete_manga(manga.id)
        os.remove(f"{self.local_dir}{manga.normalized_name}")
