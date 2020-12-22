#!/usr/bin/env python3

import os
import argparse
import shutil
import zipfile

def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("url", help="manga url")
    args = parser.parse_args()

    return args

def decompress(manga_name):

    os.chdir(manga_name)
    for vol_name in os.listdir():
        if "vol" not in vol_name: continue

        chap_num = "0" + vol_name.split("_")[1][:3]
        chap_name = "chapter_" + chap_num

        if chap_name not in os.listdir():
            os.mkdir(chap_name)
            shutil.copy(vol_name, f"{chap_name}/{vol_name}")

        os.chdir(chap_name)

        with zipfile.ZipFile(vol_name, "r") as zip_ref:
            zip_ref.extractall(".")

        os.remove("info.txt")

        os.chdir("..")
        os.remove(vol_name)

    os.chdir("..")

def main():
    os.chdir("/mnt/nasHDD/manga/")

    args = parse_args()

    download_cmd = "manga-py -R -d . " + args.url
    manga_name = args.url.split("/")[-1]

    os.system(download_cmd)

    shutil.move(manga_name, manga_name.lower().replace("-", "_"))
    manga_name = manga_name.lower().replace("-", "_")

    decompress(manga_name.lower())

if __name__ == "__main__":
    main()
