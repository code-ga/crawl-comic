import { Elysia } from "elysia";
import { swagger } from "@elysiajs/swagger";
import { cors } from "@elysiajs/cors";
import { apiRouter } from "./routes";

const app = new Elysia()
  .use(swagger())
  .use(cors())
  .get("/", () => "Hello Elysia")
  .use(apiRouter)
  .onError(({ code, error }) => {
    if (code === "VALIDATION") return error.validator.Errors(error.value).First().message;
    if (code === "NOT_FOUND")
      return new Response("Not Found :(", {
        status: 404,
      });
    if (code === "INTERNAL_SERVER_ERROR") {
    }
    if (code === "UNKNOWN") {
    }
    if (code === "PARSE") {
    }
  })
  .listen(process.env.PORT || 8000);
console.log(`🦊 Elysia is running at http://${app.server?.hostname}:${app.server?.port}`);
