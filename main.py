#!/usr/bin/env python3

import os

import argparse
import shutil
import zipfile

from query import search

def parse_args():
    parser = argparse.ArgumentParser()
    subparser = parser.add_subparsers()
    parser.add_argument("-d", "--download",
                        metavar="URL",
                        help="url of manga to download")
    parser.add_argument("-u", "--update",
                        metavar="MANGA",
                        help="name of manga to update")
    parser.add_argument("-s", "--search",
                        help="search for manga",
                        action="store_true")
    args = parser.parse_args()

    return args

def decompress(manga_name):
    os.chdir(manga_name)

    for vol_name in os.listdir():
        if "vol" not in vol_name: continue

        chap_num = "0" + vol_name.split("_")[1].split(".")[0]
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

def skip_chaps(manga):
    chaps = [c.split("_")[1]
             for c in os.listdir(manga) if "chapter" in c]
    latest = max(chaps)

    count = 0
    for chap in chaps:
        if chap <= latest: count += 1

    return str(count)

def name_to_url(name):
    name = "-".join([w.capitalize() for w in name.split("_")])
    return "https://manga4life.com/manga/" + name

def main():
    os.chdir("/mnt/nasHDD/manga/")

    args = parse_args()


    if args.search:
        manga_url = search()
        args.download = "https://manga4life.com/manga/" + manga_url

    if args.download is not None:
        download_cmd = "manga-py -R -d . " + args.download
        manga_name = args.download.split("/")[-1]

        os.system(download_cmd)

        shutil.move(manga_name, manga_name.lower().replace("-", "_"))
        manga_name = manga_name.lower().replace("-", "_")

        decompress(manga_name.lower())

    elif args.update is not None:
        url = name_to_url(args.update)
        download_cmd = "manga-py -s " + skip_chaps(args.update) + " -R -d . " + url

        os.system(download_cmd)

        if url.split("/")[-1] in os.listdir():
            for vol in os.listdir(url.split("/")[-1]):
                if "vol" not in vol: continue

                shutil.move(url.split("/")[-1] + "/" + vol,
                            args.update)

            os.rmdir(url.split("/")[-1])

            decompress(args.update)

if __name__ == "__main__":
    main()
