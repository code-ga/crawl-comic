import asyncio

import aiohttp
from dotenv import load_dotenv
from modules.blogtruyenmoi import crawlAllComics
from prisma import Prisma


load_dotenv(dotenv_path=".env")

COMIC_URL = "https://blogtruyenmoi.com/34314/thinh-thich-moi-som-mai"  # https://blogtruyenmoi.com/31115/oneshot-nhung-lai-la-series

PendingUrl: list[str] = []


async def main():
    db = Prisma(auto_register=True)
    await db.connect()
    async with aiohttp.ClientSession() as session:
        await crawlAllComics(session, COMIC_URL)
        # print(comic.toJson())
    await db.disconnect()


asyncio.run(main())
