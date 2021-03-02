import requests
import re
import json
from bs4 import BeautifulSoup
from pyfzf.pyfzf import FzfPrompt
import pandas as pd

def search():
    fzf = FzfPrompt()

    query_url = ("https://manga4life.com/search/")

    soup = BeautifulSoup(requests.get(query_url).content, "html.parser")

    pattern = re.compile(r"vm.Directory\s+=\s+\[(.*)\];")
    script = soup.find("script", text=pattern)
    data = ("{ \"mangas\": " +
            "".join(pattern.search(script.__str__()).group().split(" = ")[1:])[:-1]
            + "}")

    mangas = pd.DataFrame(json.loads(data)["mangas"])

    manga = fzf.prompt(mangas["s"])[0]
    url = str(mangas[mangas["s"] == manga]["i"].values[0])

    return url
