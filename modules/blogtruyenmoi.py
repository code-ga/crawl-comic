from util.parse import CustomBeautifulSoup
from prisma.models import Comic
from prisma import Json


def parseComicHtmlPage(html: str, comic_in_db: Comic):
    soup = CustomBeautifulSoup(html, "html.parser")

    result = {
        "author": {},
        "source": {},
        "translatorTeam": {},
        "postedBy": {},
        "genre": {},
    }
    result["pythonFetchInfo"] = True

    # parse category ( genre )
    # genres = soup.select(
    #     "#wrapper > section.main-content > div > div.col-md-8 > section > div.description > p:nth-child(2) > span"
    # )
    # for g in genres:
    #     genre = g.select("a")[0]
    #     result.addGenre(genre.text, genre["href"])

    # fetch description

    content_tag = soup.select_one(
        "#wrapper > section.main-content > div > div.col-md-8 > section > div.detail > div.content"
    )
    if content_tag:
        content = content_tag.text
        result["content"] = content

    thumbnail = soup.select_one(
        "#wrapper > section.main-content > div > div.col-md-8 > section > div.thumbnail > img"
    )
    if thumbnail:
        result["thumbnail"] = thumbnail["src"]

    # fetch another name
    description = soup.select_one(
        "#wrapper > section.main-content > div > div.col-md-8 > section > div.description"
    )
    if not description or not description.find_all("p"):
        pass
    else:
        for child in description.find_all("p"):
            child_text = child.text.strip()
            if child_text.startswith("Tên khác:"):
                result["anotherName"] = (
                    child_text[len("Tên khác:") - 1 :].strip().split(";")
                )
            if child_text.startswith("Tác giả:"):
                a_element = child.select("a")[0]
                result["author"][a_element.text] = a_element["href"]
                # result.addAuthor(a_element.text, a_element["href"])
            if child_text.startswith("Nguồn:"):
                a_element = child.select("a")[0]
                # result.addSource(a_element.text, a_element["href"])
                result["source"][a_element.text] = a_element["href"]
            if child_text.startswith("Nhóm dịch:"):
                a_element = child.select("a")
                for a in a_element:
                    # result.addTranslatorTeam(a.text, a["href"])
                    result["translatorTeam"][a.text] = a["href"]
            if child_text.startswith("Đăng bởi:"):
                a_element = child.select("a")[0]
                # result.addPostedBy(a_element.text, a_element["href"])
                result["postedBy"][a_element.text] = a_element["href"]
                span_element = child.select_one("span")
                if span_element:
                    # result.setStatus(span_element.text)
                    result["status"] = span_element.text
            if child_text.startswith("Thể loại:"):
                for g in child.select("a"):
                    # result.addGenre(g.text, g["href"])
                    result["genre"][g.text] = g["href"]

    for key, value in result.items():
        if isinstance(value, dict):
            result[key] = Json(value)

    return result
