import requests
from bs4 import BeautifulSoup

def search(name, author="", year=""):
    query_url = ("https://manga4life.com/search/?sort=s&desc=false" +
                 f"&name={name}&author={author}&year={year}")

    soup = BeautifulSoup(requests.get(query_url).content, 'html.parser')

    results = (soup.find("div", {"class": "BoxBody"})
               .find_all("div", {"class": "row"})[2]
               # .find("div", {"class": "col-md-8 order-md-1 order-12"})
               # .find_all("div", {"class": "ng-scope"})
               )

    print(results)

    # for(result in results):
    #     name = result.find()
    #     print()


search("yu")
