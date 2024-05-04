#!/usr/bin/env python3

import os

import argparse
import toml

from downloader import Downloader
import query


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-d", "--download", metavar="URL", help="url of manga to download"
    )
    parser.add_argument(
        "-u",
        "--update",
        nargs="?",
        const="*",
        default=None,
        metavar="MANGA",
        help="name of manga to update",
    )
    parser.add_argument("-s", "--search", help="search for manga", action="store_true")
    args = parser.parse_args()

    return parser, args


def parse_configs():
    try:
        with open(os.path.expanduser("~/.config/mgdl/config.toml"), "r") as config_file:
            config = toml.load(config_file)
            return os.path.expanduser(config["manga_dir"])
    except Exception:
        print("No config file found. Make sure you have a ~/.config/mgdl/config.toml")


def main():
    local_mangas = parse_configs()
    os.chdir(local_mangas)

    parser, args = parse_args()

    dldr = Downloader(local_dir=local_mangas)

    if args.search:
        manga_names = query.search()

        if manga_names is None:
            return

        args.download = []
        for manga_name in manga_names:
            args.download += ["https://manga4life.com/manga/" + manga_name]

    if args.download is not None:
        if isinstance(args.download, list):
            for url in args.download:
                dldr.manga_url = url
                dldr.download()
        else:
            dldr.manga_url = args.download
            dldr.download()
    elif args.update is not None:
        if args.update == "*":
            updatables = list(
                set([x.lower().replace("-", "_") for x in query.get_updatables()])
                & set(os.listdir())
            )
            for manga in updatables:
                print(f"Trying to update {manga} ...")
                dldr.manga_dir = manga
                dldr.update()
        else:
            print(f"Trying to update {args.update} ...")
            dldr.manga_dir = args.update
            dldr.update()
    else:
        parser.print_help()


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
