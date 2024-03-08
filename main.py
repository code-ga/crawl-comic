import asyncio
import aiohttp
from dotenv import load_dotenv
from prisma import Prisma

from modules.blogtruyenmoi import parseComicHtmlPage


load_dotenv(dotenv_path=".env")


async def main():
    db = Prisma(auto_register=True)
    await db.connect()

    while True:
        comic_in_db = await db.comic.find_first(
            where={"pythonFetchInfo": False}, order={"createdDate": "asc"}
        )
        if not comic_in_db:
            break
        async with aiohttp.ClientSession() as session:
            async with session.get(comic_in_db.url) as response:
                if response.status == 200:
                    print(f"parsing url {comic_in_db.url} - {comic_in_db.id}")
                    html = await response.text()
                    if "Truyện đã bị xóa hoặc không tồn tại!" in html:
                        print("comic not found")
                        await db.comic.update(
                            where={"id": comic_in_db.id},
                            data={"pythonFetchInfo": True},
                        )
                        continue
                    comic = parseComicHtmlPage(html, comic_in_db)
                    await db.comic.update(
                        where={"id": comic_in_db.id},
                        data=comic,
                    )
                    print("updated comic", comic_in_db.id)
                    # await asyncio.sleep(1)

    await db.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
