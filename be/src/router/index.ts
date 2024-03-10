import { Elysia, t } from "elysia";
import { prisma } from "../db";
import { parseComicHtmlPage } from "../utils/fetchComicInfo";
import { BaseResponse, Chapter, Comic, ComicIncludeChapter } from "../typings";



export const apiRouter =
    new Elysia({
        prefix: "/api",
        name: "Api routing"
    })
        .get("/comics", async ({ query }) => {
            console.log(query)
            return {
                status: 200,
                message: "Fetched successfully",
                data: await prisma.comic.findMany({
                    skip: query.skip,
                    take: query.take,
                    orderBy: { createdDate: 'asc' },
                })
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
        .get("/comic/:id", async ({ params }) => {
            console.log(params)
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
                                createdDate: true
                            }
                        }
                    }
                })
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
                data: await prisma.comic.findMany({
                    where: {
                        name: {
                            contains: params.name
                        }
                    }
                })
            }
        }, {
            params: t.Object({
                name: t.String()
            }),
            response: {
                200: BaseResponse(t.Array(Comic))
            }
        })
        .get("/search/url/:url", async ({ params }) => {
            console.log(params)
            return {
                status: 200,
                message: "Fetched successfully",
                data: await prisma.comic.findMany({
                    where: {
                        url: {
                            contains: params.url
                        }
                    }
                })
            }
        }, {
            params: t.Object({
                url: t.String()
            }),
            response: {
                200: BaseResponse(t.Array(Comic))
            }
        })
        .get("/chapter/:id", async ({ params }) => {
            console.log(params)
            const chap = await prisma.chapter.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!chap) return chap
            const filteredImages = []
            for (const url of chap.images) {
                if (url.startsWith("https://i")) {
                    filteredImages.push(url)
                }
            }
            return await prisma.chapter.update({
                where: {
                    id: params.id
                },
                data: {
                    images: filteredImages
                }
            })
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
            return await prisma.comic.findMany({
                skip: query.skip,
                take: query.take,
                orderBy: { createdDate: 'desc' },
            })
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
        .get("/refetch/comic/info/:id", async ({ params }) => {
            console.log(params)
            const comic = await prisma.comic.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!comic) return comic
            const resp = await (await fetch(comic.url)).text()
            const parsed = (parseComicHtmlPage(resp))
            console.log({ parsed })
            return await prisma.comic.update({
                where: {
                    id: params.id
                },
                data: {
                    ...parsed
                }
            })
        }, {
            params: t.Object({
                id: t.String()
            }),
            response: {
                200: BaseResponse(Comic)
            }
        })
        .get("/refetch/comic/chaps/:id", async ({ params }) => {
            console.log(params)
            const comic = await prisma.comic.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!comic) return comic
            const url = await prisma.urls.findFirst({
                where: {
                    url: {
                        contains: comic.url
                    }
                }
            })
            if (!url) return url
            // if url.updatedDate < 2 days then return already fetched
            if (url.updatedDate < new Date(Date.now() - 1000 * 60 * 60 * 24 * 2)) return {
                fetched: true,
                message: "Already fetched"
            }
            if (url.fetching) return {
                fetching: true,
                message: "Already fetching"
            }
            await prisma.urls.updateMany({
                where: {
                    url: {
                        contains: comic.url
                    }
                },
                data: {
                    fetched: false
                }
            })
            return {
                fetching: true,
                message: "Start fetching ( added to queue )"
            }
        }, {
            params: t.Object({
                id: t.String()
            }),
            response: {
                200: BaseResponse(t.Object({
                    fetched: t.Boolean(),
                    message: t.String()
                }))
            }
        })