import { Static, TSchema, t } from "elysia";

export const OptionalOrNull = <T extends TSchema>(st: T) => (t.Union([t.Null(), st, t.Undefined()]))

export const Comic = t.Object({
    id: t.String(),
    name: t.String(),
    url: t.String(),
    genre: t.Record(t.String(), t.String()),
    content: OptionalOrNull(t.String()),
    translatorTeam: t.Record(t.String(), t.String()),
    anotherName: t.Array(t.String()),
    source: t.Record(t.String(), t.String()),
    author: t.Record(t.String(), t.String()),
    postedBy: t.Record(t.String(), t.String()),
    status: t.String(),
    thumbnail: OptionalOrNull(t.String()),
    createdDate: t.Date(),
    updatedDate: t.Date(),
    pythonFetchInfo: t.Boolean()
})

export type ComicStatic = Static<typeof Comic>


export const ComicIncludeChapter = t.Composite([
    Comic,
    t.Object({
        Chapter: t.Array(t.Object({
            id: t.String(),
            name: t.String(),
            createdDate: t.String(),
            previousId: OptionalOrNull(t.String()),
            nextId: OptionalOrNull(t.String()),
            url: t.String()
        }))
    })
])

export type ComicIncludeChapterStatic = Static<typeof ComicIncludeChapter>

export const Chapter = t.Object({
    id: t.String(),
    name: t.String(),
    createdDate: t.String(),
    url: t.String(),
    images: t.Array(t.String()),
    updatedDate: t.Date(),
    comicId: t.String(),
    previousId: OptionalOrNull(t.String()),
    nextId: OptionalOrNull(t.String()),
})

export type ChapterStatic = Static<typeof Chapter>

export const BaseResponse = <T extends TSchema>(st: T) => t.Object({
    status: t.Number(),
    message: t.String(),
    data: t.Optional(st)
})
