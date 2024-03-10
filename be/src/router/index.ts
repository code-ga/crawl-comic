import { Elysia } from "elysia";
import { cdnRoute } from "./cdn";
import { apiRoute } from "./api";

export const appRoute = new Elysia({
    name: "App routing"
})
    .use(apiRoute)
    .use(cdnRoute)