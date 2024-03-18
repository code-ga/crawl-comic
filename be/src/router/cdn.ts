import { Elysia, t } from "elysia";

export const cdnRoute = new Elysia({
    prefix: "/cdn",
    name: "Cdn routing"
})
    .get("/image", async ({ query }) => {

        let url = query.url
        if (!url) return new Response("url not found", { status: 400 })
        if (url.startsWith("//st.nettruyen")) {
            url = `https:${url}`
        }
        if (url.startsWith("https::////")) {
            url = url.replace("https::////", "https://")
        }
        const resp = await fetch(url,
            {
                headers: {
                    ...url.startsWith("https://i") ? {
                        "Referer": "https://blogtruyenmoi.com/"
                    } : {},
                    ...new URL(url).hostname == "i.ntcdntempv26.com" || (/i\d+?.ntcdntempv\d+?.com/g).test(url) ? {
                        "Referer": "https://nettruyenee.com/"
                    } : {},
                    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.142.86 Safari/537.36"
                }
            }
        )
        const buffer = await resp.arrayBuffer()
        return new Response(buffer, {
            headers: {
                "Content-Type": resp.headers.get("Content-Type") || "image/png"
            }
        })

    },
        {
            query: t.Object({
                url: t.String()
            })
            , detail: {
                tags: ["CDN"],
                description: "Fetch image from url"
        }
        }
    )