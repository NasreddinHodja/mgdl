#!/usr/bin/env python3

import os

import argparse
import shutil
import zipfile

from downloader import Downloader
from query import Query

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

    return parser, args

def main():
    os.chdir("/mnt/storage/manga/")

    parser, args = parse_args()

    dldr = Downloader()

    if args.search:
        manga_url = Query.search()
        if manga_url is None:
            return

        args.download = "https://manga4life.com/manga/" + manga_url

    if args.download is not None:
        dldr.manga_url = args.download
        dldr.download()
    elif args.update is not None:
        dldr.manga_dir = args.update
        dldr.update()
    else:
        print(parser.print_help())


if __name__ == "__main__":
    main()
