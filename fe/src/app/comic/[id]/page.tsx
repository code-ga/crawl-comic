import { edenTreaty } from "@elysiajs/eden";
import { Metadata } from "next";
import { notFound } from "next/navigation";
import "react-toastify/dist/ReactToastify.css";
import { beUrl, cdnUrl } from "../../../constant";
import { ElysiaServerApi } from "../../../typings";
import { Comic } from "./ComicById";

export async function generateMetadata({
  params,
}: {
  params: { id: string };
}): Promise<Metadata> {
  const app = edenTreaty<ElysiaServerApi>(beUrl);
  const { data, error } = await app.api.comic[params.id].get();
  const comic = data?.data;
  if (error || !comic) {
    notFound();
  }
  return {
    title: comic.name,
    description: `${comic.content || "No Content provided"} - Fetched From ${
      new URL(comic.url).host
    }`,
    assets: [`${cdnUrl}/image?url=${comic.thumbnail}`],
    openGraph: {
      title: comic.name,
      description: comic.content || "",
      images: [
        {
          url: `${cdnUrl}/image?url=${comic.thumbnail}`,
          width: 800,
          height: 600,
        },
      ],
    },
  };
}

export default function Page({ params }: { params: { id: string } }) {
  return <div><Comic params={params}></Comic></div>
}
