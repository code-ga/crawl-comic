import asyncio
import aiohttp
from dotenv import load_dotenv
from prisma import Prisma
from prisma.models import Comic
import os

from modules.blogtruyenmoi import parseComicHtmlPage as parseComicHtmlPage_blogtruyenmoi
from modules.nettruyenff import parseComicHtmlPage as parseComicHtmlPage_nettruyenff
import meilisearch
from meilisearch.index import Index
import typing

load_dotenv(dotenv_path=".env")

index: typing.Union[Index, None] = None
if os.getenv("MEILISEARCH_URL") and os.getenv("MEILISEARCH_API_KEY"):
    client = meilisearch.Client(
        os.getenv("MEILISEARCH_URL"), os.getenv("MEILISEARCH_API_KEY")
    )
    index = client.index(os.getenv("MEILISEARCH_INDEX"))


async def parseComicHtmlPage(html: str, comic_in_db: Prisma.comic):
    comic = {}
    if "blogtruyenmoi" in comic_in_db.url:
        if "Truyện đã bị xóa hoặc không tồn tại!" in html:
            print("comic not found")
            await Comic.prisma().update(
                where={"id": comic_in_db.id},
                data={"pythonFetchInfo": True},
            )
            return
        comic = parseComicHtmlPage_blogtruyenmoi(html, comic_in_db)
    elif "nettruyen" in comic_in_db.url:
        comic = parseComicHtmlPage_nettruyenff(html, comic_in_db)
    updated = await Comic.prisma().update(
        where={"id": comic_in_db.id},
        data=comic,
    )
    if index and updated:
        index.add_documents([updated])
    print("updated comic", comic_in_db.id)


async def main():
    db = Prisma(auto_register=True)
    await db.connect()

    while True:
        comic_in_db = await db.comic.find_many(
            where={"pythonFetchInfo": False}, take=10
        )
        if not comic_in_db:
            break
        for c in comic_in_db:
            cachedHtml = await db.html.find_unique(where={"url": {"equals": c.url}})
            if cachedHtml:
                print(f"parsing url {comic_in_db.url} - {comic_in_db.id}")
                await parseComicHtmlPage(cachedHtml.html, c)
                await db.html.delete(where={"url": {"equals": c.url}})
                await asyncio.sleep(1)
            else:
                async with aiohttp.ClientSession() as session:
                    print(f"fetching url {comic_in_db.url} - {comic_in_db.id}")
                    async with session.get(
                        comic_in_db.url, headers={"User-Agent": "Mozilla/5.0"}
                    ) as response:
                        if response.status == 404 and "nettruyen" in comic_in_db.url:
                            print("comic not found")
                            await db.comic.update(
                                where={"id": comic_in_db.id},
                                data={"pythonFetchInfo": True},
                            )
                            continue
                        if response.status == 200:
                            print(f"parsing url {comic_in_db.url} - {comic_in_db.id}")
                            html = await response.text()
                            await parseComicHtmlPage(html, comic_in_db)
                            await asyncio.sleep(1)
    await db.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
