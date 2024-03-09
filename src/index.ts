import { Elysia } from "elysia";
import { swagger } from "@elysiajs/swagger";
import { apiRouter } from "./routes";

const app = new Elysia()
  .use(swagger())
  .use(apiRouter)
  .get("/", () => "Hello Elysia")
  .listen(3000)
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
  });

console.log(`🦊 Elysia is running at http://${app.server?.hostname}:${app.server?.port}`);
