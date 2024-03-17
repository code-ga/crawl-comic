import swagger from "@elysiajs/swagger";
import { Elysia } from "elysia";
import { appRoute } from "./router";
import cors from "@elysiajs/cors";
import createSubscriber from "pg-listen"
import { prisma } from "./db";

const subscriber = createSubscriber({
  connectionString: process.env.DATABASE_URL,
});

(async () => {
  await subscriber.connect()
  await subscriber.listenTo("new_update_or_create")
})();

const wsIntervalMap = new Map<string, any>()

const PORT = Number(process.env.PORT) || 8080;
const app = new Elysia({
  name: "Comic Database",
})
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
  .use(cors())
  .get("/", () => "Hello Elysia")
  .ws("/url/fetching", {
    open: (ws) => {
      console.log("open");
      subscriber.notifications.on("new_update_or_create", (data) => {
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