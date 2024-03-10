import { Elysia, t } from "elysia";
import { prisma } from "../db";
import { parseComicHtmlPage } from "../utils/fetchComicInfo";
import { BaseResponse, Chapter, Comic, ComicIncludeChapter } from "../typings";



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
                data: (await prisma.comic.findMany({
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
        .get("/comic/:id", async ({ params }) => {
            console.log(params)
            return {
                status: 200,
                message: "Fetched successfully",
                data: (await prisma.comic.findUnique({
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
        .get("/search/name/:name", async ({ params }) => {
            console.log(params)
            return {
                status: 200,
                message: "Fetched successfully",
                data: (await prisma.comic.findMany({
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
                data: (await prisma.comic.findMany({
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
        .get("/chapter/:id", async ({ params }) => {
            console.log(params)
            const chap = await prisma.chapter.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!chap) return {
                status: 404,
                message: "Chapter not found",
                data: null
            }
            const filteredImages = []
            for (const url of chap.images) {
                if (url.startsWith("https://i")) {
                    filteredImages.push(url)
                }
            }
            return {
                status: 200,
                message: "Fetched successfully",
                data: (await prisma.chapter.update({
                    where: {
                        id: params.id
                    },
                    data: {
                        images: filteredImages
                    }
                })) as any
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
                data: (await prisma.comic.findMany({
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
        .get("/refetch/comic/info/:id", async ({ params }) => {
            console.log(params)
            const comic = await prisma.comic.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!comic) return {
                status: 404,
                message: "Not found",
                data: null
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
                                createdDate: true
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
        .get("/refetch/comic/chaps/:id", async ({ params }) => {
            console.log(params)
            const comic = await prisma.comic.findFirst({
                where: {
                    id: params.id
                }
            })
            if (!comic) return {
                status: 404,
                message: "Not found",
                data: {
                    fetching: false,
                    message: "Not found"
                }
            }
            const url = await prisma.urls.findFirst({
                where: {
                    url: {
                        contains: comic.url
                    }
                }
            })
            if (!url) return {
                status: 404,
                message: "Not found",
                data: {
                    fetching: false,
                    message: "Not found"
                }
            }
            // if url.updatedDate < 2 days then return already fetched
            if (url.updatedDate < new Date(Date.now() - 1000 * 60 * 60 * 24 * 2)) return {
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
                        contains: comic.url
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