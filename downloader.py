import os
import shutil

import zipfile

class Downloader:

    def __init__(self, manga_dir=None, manga_url=None):
        self.manga_dir = manga_dir
        self.manga_url = manga_url

    def decompress(self):
        os.chdir(self.manga_dir)

        for vol_name in os.listdir():
            if "vol" not in vol_name: continue

            chap_subnum = vol_name.split("-")[1].split(".")[0]
            chap_num = vol_name.split("_")[1].split(".")[0].split("-")[0].zfill(4)
            chap_name = "chapter_" + chap_num + "-" + chap_subnum

            if chap_name not in os.listdir():
                os.mkdir(chap_name)
                shutil.copy(vol_name, f"{chap_name}/{vol_name}")

            os.chdir(chap_name)

            with zipfile.ZipFile(vol_name, "r") as zip_ref:
                zip_ref.extractall(".")

            os.remove("info.txt")

            if vol_name in os.listdir(): os.remove(vol_name)
            os.chdir("..")
            os.remove(vol_name)

        os.chdir("..")

    def skip_chaps(self):
        chaps = [c.split("_")[1]
                for c in os.listdir(self.manga_dir) if "chapter" in c]
        latest = max(chaps)

        count = 0
        for chap in chaps:
            if chap <= latest: count += 1

        return str(count)

    def dir_to_url(self):
        url_name = "-".join([w.capitalize() for w in self.manga_dir.split("_")])
        self.manga_url = "https://manga4life.com/manga/" + url_name
        return self.manga_url

    def update(self):
        self.dir_to_url()

        download_cmd = ("manga-py --global-progress -s " + self.skip_chaps() +
                        " -R -d . " + self.manga_url)

        os.system(download_cmd)

        if self.manga_url.split("/")[-1] in os.listdir():
            for vol in os.listdir(self.manga_url.split("/")[-1]):
                if "vol" not in vol: continue

                shutil.move(self.manga_url.split("/")[-1] + "/" + vol,
                            self.manga_dir)

            os.rmdir(self.manga_url.split("/")[-1])

            self.decompress()

    def download(self):
        if self.manga_url is None:
            raise Exception("No manga url")

        download_cmd = "manga-py --global-progress -R -d . " + self.manga_url
        manga_name = self.manga_url.split("/")[-1]
        self.manga_dir = manga_name.lower().replace("-", "_")

        print(f"Downloading {manga_name}...")

        os.system(download_cmd)

        os.listdir()
        shutil.move(manga_name, self.manga_dir)

        self.decompress()
