import asyncio
import time

import aiohttp
from dotenv import load_dotenv
from modules.blogtruyenmoi import (
    crawlAllLinksInHTML,
    fullFetchComic,
    isComicPage,
)
from prisma import Prisma
from prisma.models import PendingUrl as PendingUrlModel
from prisma.models import HistoryUrl as HistoryUrlModel


load_dotenv(dotenv_path=".env")

COMIC_URL = "https://blogtruyenmoi.com/31115/oneshot-nhung-lai-la-series"  # https://blogtruyenmoi.com/31115/oneshot-nhung-lai-la-series

PendingUrl: list[str] = []


async def main():
    db = Prisma(auto_register=True)
    await db.connect()
    await crawlAllComics(COMIC_URL)
    # print(comic.toJson())
    await db.disconnect()


async def crawlAllComics(start_url: str = "https://blogtruyenmoi.com/"):
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
        async with aiohttp.ClientSession() as session, session.get(
            current_url
        ) as response:
            html = await response.text()
            if isComicPage(html):
                asyncio.create_task(
                    fullFetchComic(
                        current_url,
                        html,
                    )
                )

            await PendingUrlModel.prisma().create_many(
                data=[{"url": u} for u in await crawlAllLinksInHTML(html)],
                skip_duplicates=True,
            )
            await PendingUrlModel.prisma().delete_many(
                where={"url": {"equals": current_url}}
            )
            await HistoryUrlModel.prisma().create({"url": current_url})
            current_url = await PendingUrlModel.prisma().find_first()
            if current_url is None:
                break
            current_url = current_url.url
            await asyncio.sleep(1)


if __name__ == "__main__":
    asyncio.run(main())
