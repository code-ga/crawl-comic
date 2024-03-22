from util.parse import CustomBeautifulSoup
from prisma.models import Comic
from prisma import Json


def parseComicHtmlPage(html: str, comic_in_db: Comic):
    # Assuming you have the HTML content in a variable called html_content

    soup = CustomBeautifulSoup(html, "html.parser")

    result = {}
    result["pythonFetchInfo"] = True

    # Get thumbnail
    thumbnail = soup.select_one(
        "#item-detail > div.detail-info > div > div.col-xs-4.col-image > img"
    )
    if thumbnail:
        result["thumbnail"] = thumbnail.get("src")

    # Get status
    status = soup.select_one(
        "#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.status.row > p.col-xs-8"
    )
    if status:
        result["status"] = status.text.strip()

    # Get another name
    another_name = soup.select_one(
        "#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.othername.row > h2"
    )
    if another_name:
        result["anotherName"] = another_name.text.strip().split(";")

    # Get genre
    genre = soup.select_one(
        "#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.kind.row > p.col-xs-8"
    )
    if genre:
        result["genre"] = {}
        for link in genre.find_all("a"):
            result["genre"][link.text.strip()] = link.get("href", "")

    # Get content
    content = soup.select_one("#item-detail > div.detail-content > p")
    if content:
        result["content"] = content.text.strip()

    # Get author
    author = soup.select_one(
        "#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.author.row > p.col-xs-8"
    )
    if author:
        result["author"] = {}
        a_element = author.find("a")
        if a_element:
            for link in a_element.find_all("a"):
                result["author"][link.text.strip()] = link.get("href", "")
        else:
            for author_text in author.text.strip().split(";"):
                result["author"][author_text.strip()] = ""

    for key, value in result.items():
        if isinstance(value, dict):
            result[key] = Json(value)

    return result
