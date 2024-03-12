import { Elysia, t } from "elysia";
import { prisma } from "../db";
import { parseComicHtmlPage, processArrayComic } from "../utils/fetchComicInfo";
import { BaseResponse, Chapter, Comic, ComicIncludeChapter } from "../typings";

const acceptedHost = ["blogtruyenmoi.com"]

export const apiRoute =
    new Elysia({
        prefix: "/api",
        name: "Api routing"
    })
        .get("/comics", async ({ query }) => {
            console.log(query)
            return {
                status: 200,
                message: "Fetched successfully",
                data: await processArrayComic(await prisma.comic.findMany({
                    skip: query.skip,
                    take: query.take,
                    orderBy: { createdDate: 'asc' },
                })) as any
            }
        }, {
            query: t.Object({
                skip: t.Numeric({
                    default: 0
                }),
                take: t.Numeric({
                    default: 10
                }),
            }),
            response: {
                200: BaseResponse(t.Array(Comic))
            }
        })
        .get("/comic/:id", async ({ params, set }) => {
            console.log(params)
            let comic = await prisma.comic.findUnique({
                where: {
                    id: params.id
                },
                include: {
                    Chapter: {
                        select: {
                            id: true,
                            name: true,
                            createdDate: true,
                            previousId: true,
                            nextId: true,
                            url: true
                        },
                        orderBy: { createdDate: 'desc' }
                    }
                }
            })
            if (!comic) {
                set.status = 404
                return {
                    status: 404,
                    message: "Not found",
                }
            }
            if (!comic.thumbnail) {
                // refetch comic 
                const resp = await (await fetch(comic.url)).text()
                const parsed = (parseComicHtmlPage(resp))
                comic = await prisma.comic.update({
                    where: {
                        id: params.id
                    },
                    data: parsed,
                    include: {
                        Chapter: {
                            select: {
                                id: true,
                                name: true,
                                createdDate: true,
                                previousId: true,
                                nextId: true,
                                url: true
                            },
                            orderBy: { createdDate: 'desc' }
                        }
                    }
                })
            }
            console.log(comic)
            if (comic.Chapter.length <= 1) {
                return {
                    status: 200,
                    message: "Fetched successfully",
                    data: comic as any
                }
            }

            const chapterUpdateInfo = []
            // [>2 element] => [{here} , {}]
            for (let i = 0; i < comic.Chapter.length; i++) {
                const current = comic.Chapter[i]
                if (i != 0) {
                    const previous = comic.Chapter[i - 1]
                    if (!previous) break
                    if (!current.previousId || previous.id != current.previousId) {
                        chapterUpdateInfo.push(prisma.chapter.update({
                            where: {
                                id: current.id
                            },
                            data: {
                                previousId: previous.id
                            }
                        }))
                    }
                }
                // update next

                const next = comic.Chapter[i + 1]
                if (!next) break
                if (!current.nextId || next.id != current.nextId) {
                    chapterUpdateInfo.push(prisma.chapter.update({
                        where: {
                            id: current.id
                        },
                        data: {
                            nextId: next.id
                        }
                    }))
                }

            }
            if (chapterUpdateInfo.length > 0)
                await prisma.$transaction(chapterUpdateInfo)

            return {
                status: 200,
                message: "Fetched successfully",
                data: await prisma.comic.findUnique({
                    where: {
                        id: params.id
                    },
                    include: {
                        Chapter: {
                            select: {
                                id: true,
                                name: true,
                                createdDate: true,
                                previousId: true,
                                nextId: true,
                                url: true
                            },
                            orderBy: { createdDate: 'desc' }
                        }
                    }
                }) as any
            }
        }, {
            params: t.Object({
                id: t.String()
            }),
            response: {
                200: BaseResponse(ComicIncludeChapter)
            }
        })
        .get("/search/name/:name", async ({ params }) => {
            console.log(params)
            return {
                status: 200,
                message: "Fetched successfully",
                data: await processArrayComic(await prisma.comic.findMany({
                    where: {
                        name: {
                            contains: params.name
                        }
                    }
                })) as any
            }
        }, {
            params: t.Object({
                name: t.String()
            }),
            response: {
                200: BaseResponse(t.Array(Comic))
            }
        })
        .get("/search/url", async ({ query }) => {
            console.log(query)
            return {
                status: 200,
                message: "Fetched successfully",
                data: await processArrayComic(await prisma.comic.findMany({
                    where: {
                        url: {
                            contains: query.url
                        }
                    }
                })) as any
            }
        },
            {
                query: t.Object({
                    url: t.String()
                }),
                response: {
                    200: BaseResponse(t.Array(Comic))
                }
            }
        )
        .get("/chapter/:id", async ({ params, set }) => {
            console.log(params)
            const chap = await prisma.chapter.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!chap) {
                set.status = 404
                return {
                    status: 404,
                    message: "Chapter not found",
                    data: null
                }
            }
            const filteredImages = []
            for (const url of chap.images) {
                if (url.startsWith("https://i")) {
                    filteredImages.push(url)
                }
            };
            const chapter = await prisma.chapter.update({
                where: {
                    id: params.id
                },
                data: {
                    images: filteredImages
                }
            })
            if (!chapter) {
                set.status = 404
                return {
                    status: 404,
                    message: "Chapter not found",
                    data: null
                }
            }
            if (!chapter.nextId && !chapter.previousId) {
                const comic = await prisma.comic.findUnique({
                    where: {
                        id: chap.comicId
                    },
                    include: {
                        Chapter: {
                            select: {
                                id: true,
                            }
                        }
                    }
                })
                if (!comic) {
                    set.status = 404
                    return {
                        status: 404,
                        message: "Comic not found",
                    }
                }
                // 2 element
                if (comic.Chapter.length <= 1) {
                    return {
                        status: 200,
                        message: "Fetched successfully",
                        data: chapter as any
                    }
                }
                const currentChapterIndex = comic.Chapter.findIndex(c => c.id == chapter.id)
                if (currentChapterIndex == -1) {
                    return {
                        status: 200,
                        message: "Fetched successfully",
                        data: chapter as any
                    }
                }
                if (currentChapterIndex == 0) {
                    return {
                        status: 200,
                        message: "Fetched successfully",
                        data: await prisma.chapter.update({
                            where: {
                                id: chapter.id
                            },
                            data: {
                                previousId: null,
                                nextId: comic.Chapter[1].id
                            }
                        }) as any
                    }
                } else {
                    return {
                        status: 200,
                        message: "Fetched successfully",
                        data: await prisma.chapter.update({
                            where: {
                                id: chapter.id
                            },
                            data: {
                                previousId: comic.Chapter[currentChapterIndex - 1].id,
                                nextId: comic.Chapter[currentChapterIndex + 1].id
                            }
                        })
                    }
                }
            }
            return {
                status: 200,
                message: "Fetched successfully",
                data: chapter as any
            }
        }, {
            params: t.Object({
                id: t.String()
            }),
            response: {
                200: BaseResponse(Chapter)
            }
        })
        .get("/stats", async () => {
            return {
                comic: await prisma.comic.count(),
                chapter: await prisma.chapter.count({
                    where: {
                        images: {
                            isEmpty: false
                        }
                    }
                })
            }
        })
        .get("/news", async ({ query }) => {
            console.log(query)
            return {
                status: 200,
                message: "Fetched successfully",
                data: await processArrayComic(await prisma.comic.findMany({
                    skip: query.skip,
                    take: query.take,
                    orderBy: { createdDate: 'desc' },
                })) as any
            }
        }, {
            query: t.Object({
                skip: t.Numeric({
                    default: 0
                }),
                take: t.Numeric({
                    default: 10
                }),
            }),
            response: {
                200: BaseResponse(t.Array(Comic))
            }
        })
        .get("/refetch/comic/info/:id", async ({ params, set }) => {
            console.log(params)
            const comic = await prisma.comic.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!comic) {
                set.status = 404
                return {
                    status: 404,
                    message: "Not found",
                    data: null
                }
            }
            const resp = await (await fetch(comic.url)).text()
            const parsed = (parseComicHtmlPage(resp))
            console.log({ parsed })
            return {
                status: 200,
                message: "Fetched successfully",
                data: (await prisma.comic.update({
                    where: {
                        id: params.id
                    },
                    data: {
                        ...parsed
                    },
                    include: {
                        Chapter: {
                            select: {
                                id: true,
                                name: true,
                                createdDate: true,
                                updatedDate: true,
                                previousId: true,
                                nextId: true,
                                url: true
                            },
                            orderBy: {
                                createdDate: "desc"
                            }
                        }
                    }
                })) as any
            }
        }, {
            params: t.Object({
                id: t.String()
            }),
            response: {
                200: BaseResponse(ComicIncludeChapter)
            }
        })
        .get("/refetch/comic/chaps/:id", async ({ params, set }) => {
            console.log(params)
            const comic = await prisma.comic.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!comic) {
                set.status = 404
                return {
                    status: 404,
                    message: "Not found",
                    data: {
                        fetching: false,
                        message: "Not found"
                    }
                }
            }
            const url = await prisma.urls.findFirst({
                where: {
                    url: {
                        equals: comic.url
                    }
                }
            })
            if (!url) {
                set.status = 404
                return {
                    status: 404,
                    message: "Not found",
                    data: {
                        fetching: false,
                        message: "Not found"
                    }
                }
            }
            // if url.updatedDate > 2 days then return already fetched
            if (url.updatedDate > new Date(Date.now() - 1000 * 60 * 60 * 24 * 2)) return {
                status: 200,
                message: "Already fetched",
                data: {
                    fetching: false,
                    message: "Already fetched"
                }
            }
            if (url.fetching) return {
                status: 200,
                message: "Already fetching",
                data: {
                    fetching: true,
                    message: "Already fetching"
                }
            }
            await prisma.urls.updateMany({
                where: {
                    url: {
                        equals: comic.url
                    }
                },
                data: {
                    fetched: false
                }
            })
            return {
                status: 200,
                message: "Fetched successfully",
                data: {
                    fetching: true,
                    message: "Start fetching ( added to queue )"
                }
            }
        }, {
            params: t.Object({
                id: t.String()
            }),
            response: {
                200: (BaseResponse(t.Object({
                    fetching: t.Boolean(),
                    message: t.String()
                })))
            }
        })
        .get("/add/comic/source", async ({ query, set }) => {
            const urlDoc = await prisma.urls.findFirst({
                where: {
                    url: {
                        equals: query.url
                    }
                }
            })
            if (urlDoc) {
                return {
                    status: 200,
                    message: "Already added",
                }
            }
            const hostname = new URL(query.url).hostname
            if (!hostname) {
                set.status = 400
                return {
                    status: 400,
                    message: "Invalid url",
                }
            }
            if (!acceptedHost.includes(hostname)) {
                set.status = 400
                return {
                    status: 400,
                    message: "Not accepted host",
                }

            }
            await prisma.urls.create({
                data: {
                    url: query.url
                }
            })
            return {
                status: 200,
                message: "Added successfully",
            }
        }, {
            query: t.Object({
                url: t.String()
            }),
            response: {
                200: (BaseResponse(t.Object({})))
            }
        })