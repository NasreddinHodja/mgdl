import requests
import os
import re
import json
from bs4 import BeautifulSoup
from iterfzf import iterfzf
import pandas as pd


def search():
    if "manga_index.csv" not in os.listdir():
        dl_manga_index()

    mangas = pd.read_csv("manga_index.csv")

    selected = iterfzf(mangas["s"], exact=True, multi=True)

    if selected is None:
        return None

    names = mangas[mangas["s"].isin(selected)].loc[:, ["i"]]
    return names["i"]


def get_updatables():
    if "manga_index.csv" not in os.listdir():
        dl_manga_index()

    mangas = pd.read_csv("manga_index.csv")
    ongoing = mangas[mangas["ps"] == "Ongoing"]["i"]
    return list(ongoing)


def dl_manga_index():
    query_url = "https://manga4life.com/search/"

    soup = BeautifulSoup(requests.get(query_url).content, "html.parser")

    pattern = re.compile(r"vm.Directory\s+=\s+\[(.*)\];")
    script = soup.find("script", text=pattern)
    directory = pattern.search(script.__str__())

    if directory is None:
        raise Exception("Manga index not found.")

    data = '{ "mangas": ' + ("".join(directory.group().split(" = ")[1:])[:-1]) + "}"

    mangas = pd.DataFrame(json.loads(data)["mangas"])
    mangas.to_csv("manga_index.csv")
