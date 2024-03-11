"use client";
import { edenTreaty } from "@elysiajs/eden";
import { notFound } from "next/navigation";
import { useEffect, useState } from "react";
import { ChapterApiReturn, ElysiaServerApi } from "../../../typings";
import { beUrl } from "../../../constant";

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
  return <div>Chapter {params.id}</div>;
}
