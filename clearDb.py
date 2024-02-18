import asyncio
from prisma import Prisma


async def main():
    db = Prisma()
    await db.connect()
    await db.chapter.delete_many()
    await db.comic.delete_many()
    await db.historyurl.delete_many()
    await db.pendingurl.delete_many()
    await db.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
