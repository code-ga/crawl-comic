import swagger from "@elysiajs/swagger";
import { Elysia, t } from "elysia";
import { apiRouter } from "./router";
import cors from "@elysiajs/cors";

const PORT = Number(process.env.PORT) || 8080;
const app = new Elysia()
  .use(swagger())
  .use(cors())
  .get("/", () => "Hello Elysia")
  .use(apiRouter)
  .listen(PORT);

export type App = typeof app
export * as types from "./typings"
console.log(
  `🦊 Elysia is running at ${app.server?.hostname}:${app.server?.port}`
);