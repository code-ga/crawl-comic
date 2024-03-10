import { ComicsApiReturn } from "../typings";
import Image from "next/image";
import Link from "next/link";
import { cdnUrl } from "../constant";

export const ComicCard = ({ comic }: { comic: ComicsApiReturn }) => {
  if (!comic) {
    return null;
  }
  return (
    <div className="flex flex-col gap-2 bg-slate-900 m-3 border border-slate-700 justify-center content-center">
      <div className="flex justify-center content-center">
        <Image
          src={cdnUrl+"/image?url=" + comic.thumbnail || ""}
          alt={comic.name}
          width={300}
          height={300}
        />
      </div>

      <div className="text-center">
        <Link href={`/comic/${comic.id}`}>
          <h1 className="text-2xl mb-2">{comic.name}</h1>
        </Link>

        <div className="text-sm mb-2">
          {comic.content
            ? comic.content.split(" ").slice(0, 100).join(" ")
            : "No content available"}
        </div>
        <div className="text-sm mb-2">
          Thể loại:
          {Object.keys(comic.genre).map((genre) => (
            <span
              key={genre}
              className="border border-slate-700 bg-slate-700 mx-2 p-1 rounded-lg"
            >
              {genre}
            </span>
          ))}
        </div>
        <div className="text-lg mb-2">
          <Link href={`/comic/${comic.id}`}>Xem trên trang</Link>
        </div>
      </div>
    </div>
  );
};
