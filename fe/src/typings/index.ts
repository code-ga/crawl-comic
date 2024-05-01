// fe/src/typings/index.ts
import { EdenTreaty } from "@elysiajs/eden/treaty";
import type { App as ElysiaServerApi, types } from "../../../be/src";
export type ArrayChildren<T> = T extends any[] ? T[number] : never
export type ComicsApiReturn = types.ComicStatic
export type ComicIncludeChapter = types.ComicIncludeChapterStatic
export type ChapterApiReturn = types.ChapterStatic
export type AppApi = EdenTreaty.Create<ElysiaServerApi>

export type { ElysiaServerApi };
