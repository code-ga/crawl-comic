import swagger from "@elysiajs/swagger";
import { Elysia, t } from "elysia";
import { PrismaClient } from '@prisma/client'
const prisma = new PrismaClient()

const app = new Elysia()
  .use(swagger())
  .get("/", () => "Hello Elysia")
  .get("/comics", async ({ query }) => {
    console.log(query)
    return await prisma.comic.findMany({
      skip: query.skip,
      take: query.take,
      orderBy: { createdDate: 'asc' },
    })
  }, {
    query: t.Object({
      skip: t.Numeric({
        default: 0
      }),
      take: t.Numeric({
        default: 10
      }),
    })
  }).get("/comic/:id", async ({ params }) => {
    console.log(params)
    return await prisma.comic.findUnique({
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
  }, {
    params: t.Object({
      id: t.String()
    })
  }).get("/search/:name", async ({ params }) => {
    console.log(params)
    return await prisma.comic.findMany({
      where: {
        name: {
          contains: params.name
        }
      }
    })
  }, {
    params: t.Object({
      name: t.String()
    })
  }).get("/chapter/:id", async ({ params }) => {
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
    })
  })
  .listen(8080);

export type App = typeof app
console.log(
  `🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`
);