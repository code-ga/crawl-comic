import { use } from "react";
import { ComicCard } from "../components/ComicCard";
import { useSearchParams } from "next/navigation";
import Link from "next/link";
import { AppApi } from "../typings";

const comicPerPage = 10;
export function ListComic({ app }: { app: AppApi }) {
  const searchParams = useSearchParams();
  const page = Number(searchParams.get("page") || 0);
  const { data } = use(
    app.api.comics.get({
      $query: { skip: page * comicPerPage, take: comicPerPage },
    })
  );
  if (!data || data.status !== 200 || !data.data) {
    return <>Server have some error</>;
  }
  const comics = data.data;
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
