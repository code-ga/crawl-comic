import time
from typing import Tuple
from aiohttp import ClientSession
from bs4 import SoupStrainer, Tag
from typings.comic import Chapter, Comic
from util import upload
from prisma.models import PendingUrl as PendingUrlModel
from prisma.models import HistoryUrl as HistoryUrlModel

from util.parse import CustomBeautifulSoup

BASE_URL = "https://blogtruyenmoi.com"


def filterLink(link: str):
    if link.startswith("/"):
        return BASE_URL + link
    if link.startswith("//id.blogtruyenmoi.com"):
        return "https:" + link  # id.blogtruyenmoi.com
    return None


def parseComicHtmlPage(html: str, comic_url: str):
    soup = CustomBeautifulSoup(html, "html.parser")

    result = Comic(soup.text.split("|")[0].strip(), comic_url)

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
        result.setContent(content.strip())

    # fetch another name
    description = soup.select_one(
        "#wrapper > section.main-content > div > div.col-md-8 > section > div.description"
    )
    for child in description.find_all("p"):
        child_text = child.text.strip()
        if child_text.startswith("Tên khác:"):
            result.setAnotherName(child_text[len("Tên khác:") - 1 :].strip().split(";"))
        if child_text.startswith("Tác giả:"):
            a_element = child.select("a")[0]
            result.addAuthor(a_element.text, a_element["href"])
        if child_text.startswith("Nguồn:"):
            a_element = child.select("a")[0]
            result.addSource(a_element.text, a_element["href"])
        if child_text.startswith("Nhóm dịch:"):
            a_element = child.select("a")
            for a in a_element:
                result.addTranslatorTeam(a.text, a["href"])
        if child_text.startswith("Đăng bởi:"):
            a_element = child.select("a")[0]
            result.addPostedBy(a_element.text, a_element["href"])
            span_element = child.select_one("span")
            if span_element:
                result.setStatus(span_element.text)
        if child_text.startswith("Thể loại:"):
            for g in child.select("a"):
                result.addGenre(g.text, g["href"])

    # fetch chapters
    chapters_elements = soup.select("#list-chapters > p")
    for chapter_element in chapters_elements:
        a_element = chapter_element.select("a")[0]
        result.addChapter(
            Chapter(
                BASE_URL + a_element["href"],
                a_element["title"],
                chapter_element.select(".publishedDate")[0].text,
                result.id,
            )
        )
    return result


async def fetchComic(session: ClientSession, comic_url: str) -> Tuple[Comic, list[str]]:
    async with session.get(comic_url) as response:
        html = await response.text()

        await PendingUrlModel.prisma().delete_many(where={"url": {"equals": comic_url}})
        await HistoryUrlModel.prisma().create({"url": comic_url})
        return (parseComicHtmlPage(html, comic_url), crawlAllLinksInHTML(html))


def crawlAllLinksInHTML(html: str) -> list[str]:
    pendingUrl = []
    for link in CustomBeautifulSoup(html, "html.parser", parse_only=SoupStrainer("a")):
        if link.has_attr("href"):
            link = filterLink(link["href"])
            if link:
                pendingUrl.append(link)
    return pendingUrl


async def fetchChapters(session: ClientSession, url: str):
    print(url)
    async with session.get(url) as response:
        html = await response.text()
        soup = CustomBeautifulSoup(html, "html.parser")
        images_tags = soup.select("#content")[0].select("img")
        result: list[str] = []
        images_url = [img["src"] for img in images_tags]
        # upload image to guilded
        for url in images_url:
            file_name, file = await upload.get_file(url)
            while file is None:
                time.sleep(1)
                file_name, file = await upload.get_file(url)
            resp = await upload.send_to_guilded(file_name=file_name, file=file)
            while resp["success"] is not True:
                time.sleep(1)
                print("retrying upload")
                resp = await upload.send_to_guilded(file_name=file_name, file=file)
            print(f"upload success: {resp['url']}")
            result.append(resp["url"])
            time.sleep(1)
        await PendingUrlModel.prisma().create_many(
            data=[{"url": url} for url in crawlAllLinksInHTML(html)],
            skip_duplicates=True,
        )
        await PendingUrlModel.prisma().delete_many(where={"url": {"equals": url}})
        await HistoryUrlModel.prisma().create({"url": url})
        return result


async def fullFetchComic(
    session: ClientSession,
    comic_url: str,
):
    """
    Fetches the comic and all its chapters with images.

    Args:
    - session: aiohttp.ClientSession object
    - comic_url: URL of the comic to fetch
    - commitToDB: function to commit data to the database parameter is comic: ComicInfo and int number of step had passed

    Returns:
    - Comic object with chapters and their images
    """
    step = 0
    comic, pendingUrl = await fetchComic(session, comic_url)
    await PendingUrlModel.prisma().create_many(
        data=[{"url": url} for url in pendingUrl], skip_duplicates=True
    )
    await comic.commitToDB()
    step += 1
    for chapter in comic.chapters:
        image_url = await fetchChapters(session, chapter.url)
        chapter.setImages(image_url)
        print(f"{chapter.name} - {chapter.url}")
        await chapter.commitToDB()
        step += 1
        time.sleep(2)
    return comic


def isComicPage(html: str):
    return True if "Thêm vào bookmark" in html else False


async def crawlAllComics(
    session: ClientSession, start_url: str = "https://blogtruyenmoi.com/"
):
    current_url = start_url
    while True:
        print(f"in time crawl: {current_url}")
        if (
            await HistoryUrlModel.prisma().find_first(
                where={"url": {"equals": current_url}}
            )
            is not None
        ):
            await PendingUrlModel.prisma().delete_many(
                where={"url": {"equals": current_url}}
            )
            current_url = await PendingUrlModel.prisma().find_first()
            if current_url is None:
                break
            current_url = current_url.url
            continue
        async with session.get(current_url) as response:
            html = await response.text()
            if isComicPage(html):
                await (await fullFetchComic(session, current_url)).commitToDB()
            await PendingUrlModel.prisma().create_many(
                data=[{"url": u} for u in crawlAllLinksInHTML(html)],
            )
            await PendingUrlModel.prisma().delete_many(
                where={"url": {"equals": current_url}}
            )
            await HistoryUrlModel.prisma().create({"url": current_url})
            current_url = await PendingUrlModel.prisma().find_first()
            if current_url is None:
                break
            current_url = current_url.url
            time.sleep(1)
