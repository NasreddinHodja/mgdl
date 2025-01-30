import os
import shutil

class Downloader:

    def __init__(self, local_dir=None, manga_dir=None, manga_url=None):
        self.local_dir = local_dir
        self.manga_dir = manga_dir
        self.manga_url = manga_url

    def organize(self, manga_dir=None):
        if not manga_dir is None:
            self.manga_dir = manga_dir

        os.chdir(f"{self.local_dir}{self.manga_dir}")

        dir_contents = os.listdir()

        for page_name in dir_contents:
            if "_c" not in page_name : continue

            chapter_and_page = page_name[:-4]

            [_, chapter_number, page_number] = chapter_and_page.split("_")
            chapter_number = chapter_number.split('.')
            if len(chapter_number) == 1:
                chapter_number += ['1']
            chapter_number[0] = chapter_number[0][1:].zfill(4)
            chapter_number = '_'.join(chapter_number)
            chapter_dir = f"chapter_{chapter_number}"

            if chapter_dir not in os.listdir():
                os.mkdir(chapter_dir)

            shutil.move(page_name, f"{chapter_dir}/{page_number}.jpg")

        os.chdir('..')


    def skip_chaps(self):
        chaps = set([
            c.split("_")[1].split("-")[0]
            for c in os.listdir(self.manga_dir)
            if ("chapter" in c)
        ])
        if len(chaps) == 0: chaps = [0]

        return str(max(chaps)).lstrip("0")

    def dir_to_url(self):
        url_name = "-".join([w.capitalize() for w in self.manga_dir.split("_")])
        self.manga_url = "https://manga4life.com/manga/" + url_name
        return self.manga_url

    def update(self):
        self.dir_to_url()

        download_cmd = (f"gallery-dl -D {self.local_dir}{self.manga_dir} --chapter-filter '" + self.skip_chaps() + " < chapter' " + self.manga_url)

        os.system(download_cmd)

        self.organize()

    def download(self):
        if self.manga_url is None:
            raise Exception("No manga url")

        manga_name = list(filter(lambda s: s != "", self.manga_url.split("/")))[-1]
        self.manga_dir = manga_name.lower().replace("-", "_")
        download_cmd = f"gallery-dl -D {self.local_dir}{self.manga_dir} {self.manga_url}"

        print(f"Downloading {manga_name}...")

        os.system(download_cmd)

        # self.organize()
