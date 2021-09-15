#!/usr/bin/env python3

import os

import argparse
import shutil
import zipfile

from downloader import Downloader
import query

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
    os.chdir("/media/manga/")

    parser, args = parse_args()

    dldr = Downloader()

    if args.search:
        manga_names = query.search()

        if manga_names is None:
            return

        args.download = []
        for manga_name in manga_names:
            args.download += ["https://manga4life.com/manga/" + manga_name]

    if args.download is not None:
        for url in args.download:
            dldr.manga_url = url
            dldr.download()
    elif args.update is not None:
        dldr.manga_dir = args.update
        dldr.update()
    else:
        print(parser.print_help())


if __name__ == '__main__':
  try:
    main()
  except KeyboardInterrupt:
    pass
