"use client";
import { use, useCallback, useEffect, useMemo, useState } from "react";
import { ComicCard } from "../components/ComicCard";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { AppApi, ComicsApiReturn } from "../typings";
import { Loading } from "./loading";

const comicPerPage = 10;
export function ListComic({ app }: { app: AppApi }) {
  const searchParams = useSearchParams();
  const page = Number(searchParams.get("page") || 0);
  const [{ data, error, loading }, setComic] = useState<{
    data?: ComicsApiReturn[] | null;
    error: any;
    loading: boolean;
  }>({
    data: null,
    error: null,
    loading: true,
  });
  useEffect(() => {
    app.api.comics
      .get({
        $query: { skip: page * comicPerPage, take: comicPerPage },
      })
      .then((data) => {
        if (data.error) {
          setComic({ data: null, error: data.error, loading: false });
          return;
        }
        console.log({ data });
        setComic({ data: data.data.data, error: data.error, loading: false });
      })
      .catch((err) => {
        setComic({ data: null, error: err, loading: false });
      })
      .finally(() => {
        setComic((pre) => ({ ...pre, loading: false }));
      });
  }, [app.api.comics, page]);
  if (loading) {
    return <Loading/>;
  }
  if (!data && error) {
    return <div>Server have some error</div>;
  }
  const comics = data!;
  return (
    <div className="col-span-3">
      {comics?.map((comic) => (
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
  );
}
