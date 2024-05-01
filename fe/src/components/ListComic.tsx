"use client";
import { ComicCard } from "../components/ComicCard";
import Link from "next/link";
import { ComicsApiReturn } from "../typings";

export function ListComic({
  comics,
  page,
}: {
  comics: ComicsApiReturn[];
  page?: number;
}) {
  return (
    <div className="col-span-4 md:col-span-3">
      {comics?.map((comic) => (
        <ComicCard key={comic.id} comic={comic}></ComicCard>
      ))}
      {/* next page */}
      {page !== undefined && (
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
      )}
    </div>
  );
}
