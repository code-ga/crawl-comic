export * from "./fetchComicInfo"
type ChapterInfo = {
    name: string;
    id: string;
    createdDate: string;
}
export function sortChapter<T extends ChapterInfo>(chapter: T[]) {
    return chapter.sort((a, b) => {
        const a_created_raw = a.createdDate + ":00" // 26/12/2018 17:16:00
        const b_created_raw = b.createdDate + ":00"
        /**
         * Convert date string to Date object
         * @param date String date in format DD/MM/YYYY HH:mm, e.g. '26/12/2018 17:16'
         * @returns Date object with time in format YYYY-MM-DD HH:mm:ss.SSS
         */
        const parse_date = (date: string): Date => {
            const year = date.substring(6, 10) as string;
            const month = date.substring(3, 5) as string;
            const day = date.substring(0, 2) as string;
            const hour = date.substring(11, 13) as string;
            const minute = date.substring(14, 16) as string;
            const second = date.substring(17, 19) as string;
            return new Date(`${year}-${month}-${day} ${hour}:${minute}:${second}.000`);
        }
        const a_created = parse_date(a_created_raw)
        const b_created = parse_date(b_created_raw)
        if (a_created.getTime() == b_created.getTime()) {
            // sort by name
            return a.name.localeCompare(b.name)
        }
        return a_created.getTime() - b_created.getTime()
    })
}