import { NextRequest } from "next/server"

export async function GET(request: NextRequest) {
    const url = request.nextUrl.searchParams.get("url")
    console.log(url)
    if (!url) return new Response("url not found", { status: 400 })
    const resp = await fetch(url,
        {
            headers: {
                ...url.startsWith("https://i") ? {
                    "Referer": "https://blogtruyenmoi.com/"
                } : {},
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.142.86 Safari/537.36"
            }
        }
    )
    const buffer = await resp.arrayBuffer()
    return new Response(buffer, {
        headers: {
            "Content-Type": "image/png"
        }
    })
}