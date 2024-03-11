// fe/src/typings/index.ts
import { edenTreaty } from "@elysiajs/eden";
import type { App as ElysiaServerApi, types } from "../../../be/src"
export type ArrayChildren<T> = T extends any[] ? T[number] : never
export type AppApi = ReturnType<typeof edenTreaty<ElysiaServerApi>>
export type ComicsApiReturn = types.ComicStatic
export type ComicIncludeChapter = types.ComicIncludeChapterStatic
export type ChapterApiReturn = types.ChapterStatic

export type { ElysiaServerApi }