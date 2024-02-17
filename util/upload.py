import io
import re
from typing import Tuple
import aiohttp
import mimetypes
import os
from urllib.parse import urlparse


guilded_bot_token = os.getenv("GUILDED_BOT_TOKEN")
upload_url = (
    "https://media.guilded.gg/media/upload?dynamicMediaTypeId=ContentMediaGenericFiles"
)


def process_url(url):
    return re.sub(
        r"https://.*?.amazonaws\.com/www\.guilded\.gg/", "https://cdn.gilcdn.com/", url
    )


async def send_to_guilded(file_name: str, file: bytes):
    # try to get from env
    global guilded_bot_token
    if not guilded_bot_token:
        guilded_bot_token = os.getenv("GUILDED_BOT_TOKEN")
    data = aiohttp.FormData()
    content_type = mimetypes.guess_type(file_name)[0]
    data.add_field("file", io.BytesIO(file), content_type=content_type)
    async with aiohttp.ClientSession(
        headers={"Authorization": f"Bearer {guilded_bot_token}"}
    ) as s:
        async with s.post(upload_url, data=data) as resp:
            if resp.ok:
                out_url = process_url((await resp.json())["url"])
                return {
                    "success": True,
                    "url": out_url,
                    "file_name": file_name,
                }
            else:
                print(await resp.text())
                return {
                    "success": False,
                    "error": resp.status,
                    "message": await resp.text(),
                }


async def get_file(url) -> Tuple[str, bytes] | Tuple[None, None]:
    async with aiohttp.ClientSession() as s:
        async with s.get(
            url, headers={"Referer": "https://blogtruyenmoi.com/"}
        ) as resp:
            if resp.ok:
                return os.path.basename(urlparse(url).path), await resp.read()
            else:
                return None
