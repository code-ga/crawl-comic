"use client";
import { ElysiaServerApi } from "@/typings";
import { edenTreaty } from "@elysiajs/eden";
import { use } from "react";
import { ComicCard } from "../components/ComicCard";
import { useSearchParams } from "next/navigation";
import Link from "next/link";

const comicPerPage = 10;
export default function Home() {
  const searchParams = useSearchParams();
  const page = Number(searchParams.get("page") || 0);
  const app = edenTreaty<ElysiaServerApi>("https://ai-datalake.nz.io.vn/");
  const { data } = use(
    app.api.comics.get({
      $query: { skip: page * comicPerPage, take: comicPerPage },
    })
  );

  // console.log(data);

  return (
    <div className="grid grid-cols-4 gap-4 text-center">
      {/* content */}
      <div>Side Bar</div>
      <div className="col-span-3">
        {data?.map((comic) => (
          <ComicCard key={comic.id} comic={comic}></ComicCard>
        ))}
        {/* next page */}
        <div className="flex justify-center content-center mb-3">
          {page > 0 && (
            <Link className="text-center mx-3" href={`/?page=${page - 1}`}>
              Previous Page
            </Link>
          )}
          <span className="text-center mx-3">{page}</span>
          <Link className="text-center mx-3" href={`/?page=${page + 1}`}>
            Next Page
          </Link>
        </div>
      </div>
    </div>
  );
}
