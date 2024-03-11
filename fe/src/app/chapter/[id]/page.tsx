"use client";
import { edenTreaty } from "@elysiajs/eden";
import { notFound } from "next/navigation";
import { useEffect, useState } from "react";
import { ChapterApiReturn, ElysiaServerApi } from "../../../typings";
import { beUrl, cdnUrl } from "../../../constant";
import Link from "next/link";
import Image from "next/image";

export default function Page({ params }: { params: { id: string } }) {
  const app = edenTreaty<ElysiaServerApi>(beUrl);

  const [
    { data: chapterData, error: chapterError, loading: chapterLoading },
    setChapter,
  ] = useState<{
    data?: ChapterApiReturn | null;
    error: any;
    loading: boolean;
  }>({
    data: null,
    error: null,
    loading: true,
  });
  useEffect(() => {
    app.api.chapter[params.id]
      .get()
      .then((data) => {
        if (data.error) {
          setChapter({ data: null, error: data.error, loading: false });
          return;
        }

        setChapter({ data: data.data.data, error: data.error, loading: false });
      })
      .catch((err) => {
        setChapter({ data: null, error: err, loading: false });
      })
      .finally(() => {
        setChapter((pre) => ({ ...pre, loading: false }));
      });
  }, []);
  if (chapterLoading) {
    return <div>Loading...</div>;
  }
  if (!chapterData && chapterError) {
    notFound();
  }
  const chapter = chapterData!;
  console.log({ chapter });
  return (
    <div>
      <div className="text-center mb-4">
        {/* header */}
        <div>
          <Link href={`/comic/${chapter.comicId}`} className="text-blue-500">
            Back to comic
          </Link>
        </div>
        <div>{chapter.name}</div>
        <div>Tạo ngày {chapter.createdDate}</div>
      </div>
      <div className="mb-4 text-center flex justify-center content-center">
        {/* navigation */}
        {chapter.previousId && (
          <Link
            href={`/chapter/${chapter.previousId}`}
            className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md mx-3"
          >
            Trang trước
          </Link>
        )}
        {chapter.nextId && (
          <Link
            href={`/chapter/${chapter.nextId}`}
            className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md mx-3"
          >
            Trang sau
          </Link>
        )}
      </div>
      <div className="text-center mb-4 mx-auto">
        {/* reader */}
        {chapter.images.map((image) => (
          <Image
            src={cdnUrl + "/image?url=" + image}
            alt={image}
            key={image}
            width={window.innerWidth / 2}
            height={window.innerHeight / 2}
          />
        ))}
      </div>
    </div>
  );
}
