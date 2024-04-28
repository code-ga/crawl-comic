import swagger from "@elysiajs/swagger";
import { Elysia } from "elysia";
import { appRoute } from "./router";
import cors from "@elysiajs/cors";
import createSubscriber from "pg-listen"
import { prisma } from "./db";
import MeiliSearch from "meilisearch";


const subscriber = createSubscriber({
  connectionString: process.env.DATABASE_URL,
});

(async () => {
  await prisma.urls.updateMany({
    where: {
      fetching: true
    },
    data: {
      fetching: false
    }
  })
  await subscriber.connect()
  await subscriber.listenTo("new_update_or_create_Urls")
  await subscriber.listenTo("new_update_or_create_Comic")
})();

const meili = process.env.MEILISEARCH_HOST ? new MeiliSearch({
  host: process.env.MEILISEARCH_HOST,
  apiKey: process.env.MEILISEARCH_API_KEY,
}) : undefined

subscriber.notifications.on("new_update_or_create_Comic", async (data) => {
  const index = meili?.index("Comic_meilisearch")
  if (index) {
    await index.addDocuments([data])
  }
  console.log("new_update_or_create_Comic", data)
})

const wsIntervalMap = new Map<string, any>()

const PORT = Number(process.env.PORT) || 8080;
const app = new Elysia()
  .use(cors())
  .use(swagger({
    version: "0.0.1-alpha",
    documentation: {
      info: {
        title: "Crawling comic api",
        version: "0.0.1-alpha",
        description: "This is the comic,manga,novel api for comic app. if you want to provide the own read comic ui you can use this api to that or some dataset to train the model.",
        license: {
          name: "Unlicense",
        },
        contact: {
          name: "Siuu",
          email: "nbth@nzmanga.io.vn",
        },
      },
      tags: [
        {
          name: "Comic",
          description: "Comic related api"
        },
        {
          name: "CDN",
          description: "CDN related api"
        }
      ]
    }
  }))

  .get("/", () => "Hello Elysia")
  .ws("/url/fetching", {
    open: (ws) => {
      console.log("open");
      subscriber.notifications.on("new_update_or_create_Urls", (data) => {
        ws.send(JSON.stringify(data))
      })
      wsIntervalMap.set(ws.id, setInterval(async () => {
        const fetchingUrl = await prisma.urls.findMany({
          where: {
            fetching: true
          }
        })
        ws.send(JSON.stringify(fetchingUrl))
      }, 1000 * 60 * 5)) // 1 min
    },
    close: (ws) => {
      console.log("close");
      clearInterval(wsIntervalMap.get(ws.id))
    },
    message: (msg) => {
      console.log("message", msg);
    },
  })
  .use(appRoute)
  .listen(PORT);

export type App = typeof app
export * as types from "./typings"
console.log(
  `🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`
);