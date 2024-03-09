// fe/src/typings/index.ts
import { edenTreaty } from "@elysiajs/eden";
import type { App as ElysiaServerApi } from "../../../be/src"
export type ArrayChildren<T> = T extends any[] ? T[number] : never
export type AppApi = ReturnType<typeof edenTreaty<ElysiaServerApi>>
export type ComicsApiReturn = Awaited<ReturnType<AppApi["api"]["comics"]["get"]>>["data"]

export type { ElysiaServerApi }