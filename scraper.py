#!/usr/bin/env python3

from typing import Tuple
import requests
from bs4 import BeautifulSoup
from db import Chapter, Manga

REQ_HEADER = {
    "User-Agent": "Mozilla/5.0 (X11; Linux x86_64; rv:135.0) Gecko/20100101 Firefox/135.0",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.5",
    "DNT": "1",
    "Sec-GPC": "1",
    "Upgrade-Insecure-Requests": "1",
    "Sec-Fetch-Dest": "document",
    "Sec-Fetch-Mode": "navigate",
    "Sec-Fetch-Site": "none",
    "Sec-Fetch-User": "?1",
    "Connection": "keep-alive",
}

def get_chapters(manga_hash: str, manga_id: str) -> list[Chapter]:
    chapters = []
    url = f"https://weebcentral.com/series/{manga_hash}/full-chapter-list"

    response = requests.get(url, headers=REQ_HEADER)

    if response.status_code == 200:
        soup = BeautifulSoup(response.text, "html.parser")
        mangas_soup = soup.select("div > a")
        for manga in mangas_soup:
            url = manga["href"]
            if isinstance(url, list):
                url = url[0]
            hash = url.split("/")[-1]
            number = manga.select("span > span")[0].get_text(strip=True).split(" ")[-1]
            number = number.replace(".", "-")
            if "-" in number:
                numbers = number.split("-")
                number = numbers[0].zfill(4) + "-" + numbers[1].zfill(2)
            else:
                number = number.zfill(4) + "-01"

            chapter = Chapter.create(hash, number, manga_id)
            chapters.append(chapter)

    else:
        raise Exception(f"Error fetching: {response.status_code} {response.text}")

    return chapters

def manga_from_url(url: str) -> Tuple[Manga, list[Chapter]]:
    response = requests.get(url, headers=REQ_HEADER)

    if response.status_code == 200:
        soup = BeautifulSoup(response.text, "html.parser")
        name = soup.select("main > div > section > section > h1")[0].get_text(strip=True)
        hash = url.split("/")[-2]
        authors = ""
        status = ""
        infos = soup.select(
            "main > div > section > section > section > ul.flex.flex-col.gap-4 > li"
        )

        for info in infos:
            key = (
                info.select("strong")[0]
                .get_text(strip=True)
                .replace(":", "")
                .replace("(s)", "")
            )

            if key == "Author":
                value = [
                    element.get_text(strip=True).replace(":", "").replace("(s)", "")
                    for element in info.select("span > a")
                ]
                authors = ",".join(value)

            elif key == "Status":
                value = [
                    element.get_text(strip=True).replace(":", "").replace("(s)", "")
                    for element in info.select("a")
                ][0]
                status = value

        normalized_name = url.split("/")[-1].lower()
        manga = Manga.create(hash, name, normalized_name, authors, status)
        chapters = get_chapters(manga.hash, manga.id)

    else:
        raise Exception(f"Error fetching: {response.status_code} {response.text}")

    return (manga, chapters)
