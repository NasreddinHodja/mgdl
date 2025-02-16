#!/usr/bin/env python3

import os
import argparse
import toml
from downloader import Downloader

def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "-d", "--download", nargs="+", metavar="URL", help="url of manga to download"
    )
    parser.add_argument(
        "-a", "--add", nargs="+", metavar="URL", help="url of manga to add"
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
    parser.add_argument(
        "-o",
        "--organize",
        default=None,
        metavar="MANGA_DIR",
        help="path of manga directory to organize",
    )
    parser.add_argument("-q", "--query", metavar="QUERY", help="query manga")
    parser.add_argument("-r", "--remove", metavar="MANGA", help="remove manga")
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
    if not isinstance(local_mangas, str):
        raise Exception("Couldn't parse configs")
    os.chdir(local_mangas)

    parser, args = parse_args()

    dldr = Downloader(local_dir=local_mangas)

    if args.query:
        if isinstance(args.query, list):
            raise Exception("Provide only 1 query string")
        else:
            from db import Db
            result = Db.query_manga(args.query)
            for r in result:
                print(r, end="\n\n")

    # TODO
    # elif args.search:
    #     if isinstance(args.query, list):
    #         raise Exception("Provide only 1 search string")
    #     else:
    #         from db import Db
    #         result = Db.search_manga(args.search)
    #         print(result)

    elif not args.download is None:
        if isinstance(args.download, list):
            for url in args.download:
                dldr.download(url)
        else:
            dldr.download(args.download)

    elif not args.add is None:
        if isinstance(args.add, list):
            for url in args.add:
                dldr.add(url)
        else:
            dldr.add(args.add)

    elif not args.organize is None:
        dldr.organize(args.organize)

    elif not args.remove is None:
        if isinstance(args.remove, list):
            for manga in args.remove:
                dldr.remove(manga)
        else:
            dldr.remove(args.remove)

    elif not args.organize is None:
        dldr.organize(args.organize)

    elif args.update is not None:
        if args.update == "*":
            ongoing = dldr.get_updatables()

            for manga in ongoing:
                print(f"Trying to update {manga.name} ...")
                dldr.update(manga.normalized_name)
        else:
            print(f"Trying to update {args.update} ...")
            dldr.update(args.update)

    else:
        parser.print_help()

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
