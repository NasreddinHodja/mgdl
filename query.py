import requests
import os
import re
import json
import csv
from bs4 import BeautifulSoup
from iterfzf import iterfzf


def search():
    if "manga_index.csv" not in os.listdir():
        dl_manga_index()

    mangas = []
    with open("manga_index.csv", mode="r", newline="", encoding="utf-8") as file:
        reader = csv.DictReader(file)
        for row in reader:
            mangas.append(row)

    selected = iterfzf([manga["s"] for manga in mangas], exact=True, multi=True)

    if selected is None:
        return None

    names = [manga["i"] for manga in mangas if manga["s"] in selected]
    return names


def get_updatables():
    if "manga_index.csv" not in os.listdir():
        dl_manga_index()

    mangas = []
    with open("manga_index.csv", mode="r", newline="", encoding="utf-8") as file:
        reader = csv.DictReader(file)
        for row in reader:
            if row["ps"] == "Ongoing":
                mangas.append(row["i"])

    return mangas


def dl_manga_index():
    query_url = "https://manga4life.com/search/"

    soup = BeautifulSoup(requests.get(query_url).content, "html.parser")

    pattern = re.compile(r"vm.Directory\s+=\s+\[(.*)\];")
    script = soup.find("script", text=pattern)
    directory = pattern.search(script.__str__())

    if directory is None:
        raise Exception("Manga index not found.")

    data = '{ "mangas": ' + ("".join(directory.group().split(" = ")[1:])[:-1]) + "}"
    mangas = json.loads(data)["mangas"]

    with open("manga_index.csv", mode="w", newline="", encoding="utf-8") as file:
        writer = csv.DictWriter(file, fieldnames=mangas[0].keys())
        writer.writeheader()
        writer.writerows(mangas)

