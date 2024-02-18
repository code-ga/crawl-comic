import json
import uuid
from typing import Optional
from prisma.models import Chapter as ChapterModel
from prisma.models import Comic as ComicModel
from prisma import Json


class Chapter:
    id: str
    name: str
    url: str
    createdDate: str
    images: list[str]
    comicId: str

    def __init__(self, url, name, createdDate: str, comicID):
        self.url = url
        self.name = name
        # createdDate format: "13/02/2024 10:09"
        self.createdDate = createdDate + ":00.000"
        self.id = str(uuid.uuid4())
        self.images = []
        self.comicId = comicID

    def setImages(self, images: list[str]):
        self.images = images

    def fromDict(data: dict[str, str]):
        """
        Create a Chapter object from a dictionary.

        Args:
            data (dict[str, str]): The dictionary containing chapter data.

        Returns:
            Chapter: The Chapter object created from the dictionary.
        """
        result = Chapter(data["url"], data["name"], data["createdDate"])
        result.id = data["id"]
        if data.get("images"):
            result.images = data["images"]
        return result

    def fromString(jsonStr: str):
        data = json.loads(jsonStr)
        return Chapter.fromDict(data)

    def toJson(self):
        """
        Return the JSON representation of the Chapter object.
        """
        return json.dumps(self, default=lambda o: o.__dict__, sort_keys=True, indent=4)

    def toDict(self):
        """
        Return the dictionary representation of the Chapter object.
        """
        result = self.__dict__.copy()
        return result

    def toPrismaDict(self):
        result = self.toDict()
        result.pop("comicId")
        result["comic"] = {"connect": {"id": self.comicId}}
        return result

    async def commitToDB(self):
        await updateChapterInfo(self)
        return self


class ComicInfo:
    id: str
    name: str
    url: str
    genre: dict[str, str]
    content: Optional[str]
    translatorTeam: dict[str, str]
    anotherName: list[str]
    source: dict[str, str]
    author: dict[str, str]
    postedBy: dict[str, str]
    status: str
    changed: bool = False
    # TODO: Fetch thumbnail

    def __init__(self, name, url):
        self.url = url
        self.name = name
        self.genre = {}
        self.content = None
        self.translatorTeam = {}
        self.anotherName = []
        self.source = {}
        self.author = {}
        self.postedBy = {}
        self.status = "Không xác định"
        self.id = str(uuid.uuid4())

    def __str__(self):
        return f"{self.name} - {self.url}"

    def toJson(self):
        """
        Return the JSON representation of the ComicInfo object.
        """
        return json.dumps(self, default=lambda o: o.__dict__, sort_keys=True, indent=4)

    def print(self):
        """
        Print the JSON representation of the ComicInfo object.
        """
        print(self.toJson())

    def addGenre(self, genre: str, url: str):
        """
        Add a genre to the ComicInfo object with the given genre and URL.

        Args:
        genre (str): The genre to be added.
        url (str): The URL associated with the genre.
        """
        self.genre[genre] = url
        self.changed = True

    def addAuthor(self, authorName: str, authorUrl: str):
        self.author[authorName] = authorUrl
        self.changed = True

    def addSource(self, sourceName: str, sourceUrl: str):
        self.source[sourceName] = sourceUrl
        self.changed = True

    def addTranslatorTeam(self, translatorName: str, translatorUrl: str):
        self.translatorTeam[translatorName] = translatorUrl
        self.changed = True

    def addPostedBy(self, postedByName: str, postedByUrl: str):
        self.postedBy[postedByName] = postedByUrl
        self.changed = True

    def setAnotherName(self, anotherName: list[str]):
        self.anotherName = anotherName
        self.changed = True

    def setStatus(self, status: str):
        self.status = status
        self.changed = True

    def setContent(self, content: str):
        self.content = content
        self.changed = True

    def toDict(self):
        return self.__dict__


class Comic(ComicInfo):
    chapters: list[Chapter]
    chapters_ids: list[str]
    chapters_urls: list[str]

    def __init__(self, name, url):
        super().__init__(name, url)
        self.chapters = []
        self.chapters_ids = []
        self.chapters_urls = []

    def addChapter(self, chapter: Chapter):
        """
        Add a chapter to the ComicInfo object.

        Args:
        chapter (Chapter): The chapter to be added.
        """
        self.chapters.append(chapter)
        self.chapters_ids.append(chapter.id)
        self.chapters_urls.append(chapter.url)
        self.changed = True

    async def commitToDB(self):
        await updateComicInfo(self)
        for c in self.chapters:
            await c.commitToDB()
        self.changed = False

    def toDict(self):
        result = super().toDict().copy()
        result.pop("chapters")
        result.pop("chapters_ids")
        result.pop("chapters_urls")
        result.pop("changed")
        return result

    def toPrismaDict(self):
        result = self.toDict()
        for k, v in result.items():
            if isinstance(v, dict):
                result[k] = Json(v)
        return result

    def fromString(jsonStr: str):
        data = json.loads(jsonStr)
        result = Comic(data["name"], data["url"])
        result.genre = data["genre"]
        result.content = data["description"]
        for c in data["chapters"]:
            result.addChapter(Chapter.fromDict(c))
        return result


async def CreateComic(comic: Comic):
    await ComicModel.prisma().create(data=comic.toPrismaDict())


async def updateComicInfo(comic: Comic):
    DBComic = await ComicModel.prisma().find_first(where={"url": comic.url})
    if DBComic is None:
        return await CreateComic(comic)
    else:
        comic.id = DBComic.id
        value = {
            k: comic.toPrismaDict()[k]
            for k in set(comic.toPrismaDict()) - set(DBComic.__dict__)
        }
        if value:
            return await ComicModel.prisma().update(where={"id": comic.id}, data=value)
        else:
            return DBComic


async def createChapter(chapter: Chapter):
    return await ChapterModel.prisma().create(data=chapter.toPrismaDict())


async def updateChapterInfo(chapter: Chapter):
    DBChapter = await ChapterModel.prisma().find_first(where={"url": chapter.url})
    if DBChapter is None:
        return await createChapter(chapter)
    else:
        chapter.id = DBChapter.id
        value = {
            k: chapter.toPrismaDict()[k]
            for k in set(chapter.toPrismaDict()) - set(DBChapter.__dict__)
        }

        value["images"] = chapter.images
        if value:
            return await ChapterModel.prisma().update(
                where={"id": chapter.id}, data=value
            )
        else:
            return DBChapter
