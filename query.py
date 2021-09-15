import requests
import os
import re
import json
from bs4 import BeautifulSoup
from iterfzf import iterfzf
import pandas as pd

class Query:

    def search():
        if("manga_index.csv" not in os.listdir()):
            Query.dl_manga_index()

        mangas = pd.read_csv("manga_index.csv")

        manga = iterfzf(mangas["s"], exact=True)

        if manga is None:
            return None

        url = str(mangas[mangas["s"] == manga]["i"].values[0])

        return url

    def dl_manga_index():
        query_url = ("https://manga4life.com/search/")

        soup = BeautifulSoup(requests.get(query_url).content, "html.parser")

        pattern = re.compile(r"vm.Directory\s+=\s+\[(.*)\];")
        script = soup.find("script", text=pattern)
        data = ("{ \"mangas\": " +
                "".join(pattern.search(script.__str__()).group().split(" = ")[1:])[:-1]
                + "}")

        mangas = pd.DataFrame(json.loads(data)["mangas"])
        mangas.to_csv("manga_index.csv")
