import swagger from "@elysiajs/swagger";
import { Elysia } from "elysia";
import { appRoute } from "./router";
import cors from "@elysiajs/cors";
import createSubscriber from "pg-listen"

const subscriber = createSubscriber({
  connectionString: process.env.DATABASE_URL,
});

  (async () => {
    await subscriber.connect()
    await subscriber.listenTo("new_update_or_create")
  })();


const PORT = Number(process.env.PORT) || 8080;
const app = new Elysia()
  .use(swagger())
  .use(cors())
  .get("/", () => "Hello Elysia")
  .ws("/url/fetching", {
    open: (ws) => {
      console.log("open");
      subscriber.notifications.on("new_update_or_create", (data) => {
        ws.send(JSON.stringify(data))
      })
    },
    close: () => {
      console.log("close");
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