import asyncio
import aiohttp
from dotenv import load_dotenv
from prisma import Prisma

from modules.blogtruyenmoi import parseComicHtmlPage as parseComicHtmlPage_blogtruyenmoi
from modules.nettruyenff import parseComicHtmlPage as parseComicHtmlPage_nettruyenff


load_dotenv(dotenv_path=".env")


async def main():
    db = Prisma(auto_register=True)
    await db.connect()

    while True:
        comic_in_db = await db.comic.find_first(where={"pythonFetchInfo": False})
        if not comic_in_db:
            break
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
                    comic = {}
                    if "blogtruyenmoi" in comic_in_db.url:
                        if "Truyện đã bị xóa hoặc không tồn tại!" in html:
                            print("comic not found")
                            await db.comic.update(
                                where={"id": comic_in_db.id},
                                data={"pythonFetchInfo": True},
                            )
                            continue
                        comic = parseComicHtmlPage_blogtruyenmoi(html, comic_in_db)
                    elif "nettruyen" in comic_in_db.url:
                        comic = parseComicHtmlPage_nettruyenff(html, comic_in_db)
                    await db.comic.update(
                        where={"id": comic_in_db.id},
                        data=comic,
                    )
                    print("updated comic", comic_in_db.id)
                    await asyncio.sleep(1)
    await db.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
