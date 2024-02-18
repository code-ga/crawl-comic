import asyncio
from typing import Optional, Tuple
from aiohttp import ClientSession
from bs4 import SoupStrainer, Tag
from typings.comic import Chapter, Comic
from util import upload
from prisma.models import PendingUrl as PendingUrlModel
from prisma.models import HistoryUrl as HistoryUrlModel

from util.parse import CustomBeautifulSoup

BASE_URL = "https://blogtruyenmoi.com"


def filterLink(link: str):
    if link.startswith("//id.blogtruyenmoi.com"):
        return "https:" + link  # id.blogtruyenmoi.com
    if link.startswith("/"):
        return BASE_URL + link
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


async def fetchComic(
    comic_url: str, html: Optional[str] = None
) -> Tuple[Comic, list[str]]:
    print(f"fetching comic: {comic_url}")
    if html:
        return (parseComicHtmlPage(html, comic_url), [])
    else:
        async with ClientSession() as session, session.get(comic_url) as response:
            html = await response.text()
            await PendingUrlModel.prisma().delete_many(
                where={"url": {"equals": comic_url}}
            )
            await HistoryUrlModel.prisma().create({"url": comic_url})
            return (parseComicHtmlPage(html, comic_url), [])


async def crawlAllLinksInHTML(html: str) -> list[str]:
    pendingUrl = []
    for link in CustomBeautifulSoup(html, "html.parser", parse_only=SoupStrainer("a")):
        if link.has_attr("href"):
            link = filterLink(link["href"])
            if link:
                if (
                    await HistoryUrlModel.prisma().find_first(
                        where={"url": {"equals": link}}
                    )
                    is not None
                ):
                    continue
                pendingUrl.append(link)
    return pendingUrl


async def fetchChapters(url: str) -> Tuple[Comic, list[str]]:
    print(f"fetching chapters: {url}")
    async with ClientSession() as session, session.get(url) as response:
        html = await response.text()
        soup = CustomBeautifulSoup(html, "html.parser")
        images_tags = soup.select("#content")[0].select("img")
        # result: list[str] = []
        images_url = [img["src"] for img in images_tags]
        # upload image to guilded
        # for url in images_url:
        #     file_name, file = await upload.get_file(url)
        #     while file is None:
        #         await asyncio.sleep(1)
        #         file_name, file = await upload.get_file(url)
        #     resp = await upload.send_to_guilded(file_name=file_name, file=file)
        #     while resp["success"] is not True:
        #         await asyncio.sleep(1)
        #         print(f"retrying upload: {url}")
        #         resp = await upload.send_to_guilded(file_name=file_name, file=file)
        #     print(f"upload success: {resp['url']}")
        #     result.append(resp["url"])
        #     await asyncio.sleep(1)
        await PendingUrlModel.prisma().delete_many(where={"url": {"equals": url}})
        await HistoryUrlModel.prisma().create({"url": url})
        return images_url, await crawlAllLinksInHTML(html)


async def fullFetchComic(
    comic_url: str,
    html: Optional[str] = None,
) -> Tuple[Comic, list[str]]:
    """
    Fetches the comic and all its chapters with images.

    Args:
    - session: aiohttp.ClientSession object
    - comic_url: URL of the comic to fetch
    - commitToDB: function to commit data to the database parameter is comic: ComicInfo and int number of step had passed

    Returns:
    - Comic object with chapters and their images
    """
    pendingUrl: list[str] = []
    # because we fetch all link of page at outer function
    comic, _ = await fetchComic(comic_url, html)
    await comic.commitToDB()
    for chapter in comic.chapters:
        image_url, chapPendingUrl = await fetchChapters(chapter.url)
        chapter.setImages(image_url)
        print(f"{chapter.name} - {chapter.url}")
        await chapter.commitToDB()
        await asyncio.sleep(1)
        pendingUrl.extend(chapPendingUrl)
        # pendingUrl = [...pendingUrl, ...chapPendingUrl]
    await comic.commitToDB()
    # remove chapter url out of pending url
    pendingUrl = [u for u in pendingUrl if u not in comic.chapters_urls]
    await PendingUrlModel.prisma().create_many(
        data=[{"url": u} for u in pendingUrl],
        skip_duplicates=True,
    )
    print(f"fetch completed: {comic.name} - {comic_url}")

    return comic, pendingUrl


def isComicPage(html: str):
    return True if "Thêm vào bookmark" in html else False
