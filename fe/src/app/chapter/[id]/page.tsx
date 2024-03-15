"use client";
import { edenTreaty } from "@elysiajs/eden";
import { notFound } from "next/navigation";
import { useEffect, useState } from "react";
import { ChapterApiReturn, ElysiaServerApi } from "../../../typings";
import { beUrl, cdnUrl } from "../../../constant";
import Link from "next/link";
import Image from "next/image";
import { Loading } from "../../../components/loading";

export default function Page({ params }: { params: { id: string } }) {
  const app = edenTreaty<ElysiaServerApi>(beUrl);
  const [images, setImages] = useState<string[]>([]);
  const [nowServer, setNowServer] = useState<string | null>(null);

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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [params.id]);
  if (chapterLoading) {
    return <Loading />;
  }
  if (!chapterData && chapterError) {
    notFound();
  }
  const chapter = chapterData!;
  if (chapter.images.length > 0) {
    setImages((pre) => {
      for (const image of chapter.images) {
        const url = cdnUrl + "/image?url=" + image;
        if (!pre.includes(url)) {
          pre.push(url);
        }
      }
      return pre;
    });
  }
  if (chapter.serverImage.length > 0) {
    const preImages = [] as string[];
    for (const image of chapter.serverImage) {
      // key is the image server like 1 and 2 and value is the url just get value now
      const url =
        cdnUrl +
        "/image?url=" +
        image[nowServer || Object.keys(chapter.serverImage[0])[0] || ""];
      if (!preImages.includes(url)) {
        preImages.push(url);
      }
    }

    if (JSON.stringify(preImages) != JSON.stringify(images)) {
      setImages(preImages);
    }
  }
  const selectImageServer = (server: string) => {
    setNowServer(server);
  };
  console.log({ chapter });
  return (
    <div>
      <button className="fixed text-center top-[50%] bg-blue-400 rotate-90">
        Phụ đề
      </button>
      <div className="text-center mb-4">
        {/* header */}
        <div>
          <Link href={`/comic/${chapter.comicId}`} className="text-blue-500">
            Back to comic
          </Link>
        </div>
        <div className="flex justify-center content-center">
          <span className="mr-1 hidden sm:inline">Cào tại :</span>
          <Link href={chapter.url} className="text-blue-500" target="_blank">
            {chapter.url}
          </Link>
        </div>
        <div>{chapter.name}</div>
        <div>Tạo ngày: {chapter.createdDate}</div>
        <div>Update lần cuối lúc : {chapter.updatedDate.toString()}</div>
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
        <div className="flex justify-center content-center my-4">
          {chapter.serverImage
            .reduce((pre, cur) => {
              const server = Object.keys(cur);
              server.forEach((s) => {
                if (!pre.includes(s)) {
                  pre.push(s);
                }
              });
              return pre;
            }, [] as string[])
            .map((server, index) => (
              <button
                key={index}
                className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md mx-3"
                onClick={() => selectImageServer(server)}
              >
                server {index + 1} ({server})
              </button>
            ))}
        </div>
        {/* reader */}
        {images.map((image) => (
          <Image
            src={image}
            alt={image}
            key={image}
            width={window.innerWidth / 2 + 2000}
            height={window.innerHeight / 2}
            // layout="fill" // required
            objectFit="cover" // change to suit your needs
            className="mx-auto w-auto h-auto"
          />
        ))}
      </div>
      <div className="text-center my-5">
        {/* footer */}
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
    </div>
  );
}
