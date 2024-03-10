import { TSchema, t } from "elysia";

export const Comic = t.Object({
    id: t.String(),
    name: t.String(),
    anotherName: t.Array(t.String()),
    source: t.Record(t.String(), t.String()),
    translatorTeam: t.Record(t.String(), t.String()),
    postedBy: t.Record(t.String(), t.String()),
    genre: t.Record(t.String(), t.String()),
    content: t.String(),
    thumbnail: t.String(),
    createdDate: t.Date(),
    updatedDate: t.Date(),
    status: t.String(),
    pythonFetchInfo: t.Boolean()
})

export const ComicIncludeChapter = t.Composite([
    Comic,
    t.Object({
        Chapter: t.Array(t.Object({
            id: t.String(),
            name: t.String(),
            createdDate: t.String()
        }))
    })
])
export const Chapter = t.Object({
    id: t.String(),
    name: t.String(),
    createdDate: t.String(),
    url: t.String(),
    images: t.Array(t.String()),
    updatedDate: t.Date(),
    comicId: t.String()
})

export const BaseResponse = <T extends TSchema>(st: T) => t.Object({
    status: t.Number(),
    message: t.String(),
    data: t.Optional(st)
})