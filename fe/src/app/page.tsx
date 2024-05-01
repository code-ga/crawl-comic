"use client";
import { AppApi, ComicsApiReturn, ElysiaServerApi } from "@/typings";
import { edenTreaty } from "@elysiajs/eden";
import { Suspense, useEffect, useState } from "react";
import { ListComic } from "../components/ListComic";
import { Loading } from "../components/loading";
import { SideBar } from "../components/Sidebar";
import { beUrl } from "../constant";
import { notFound, useSearchParams } from "next/navigation";

const comicPerPage = 10;
export default function Home() {
  const app = edenTreaty<ElysiaServerApi>(beUrl);

  return (
    <div className="grid grid-cols-4 gap-4 text-center">
      {/* content */}
      <div className="hidden md:flex w-full">
        <Suspense fallback={<Loading />}>
          <SideBar app={app}></SideBar>
        </Suspense>
        <Suspense fallback={<Loading />}>
          <Comics app={app}></Comics>
        </Suspense>{" "}
      </div>
    </div>
  );
}

function Comics({ app }: { app: AppApi }) {
  const [{ comic, error, loading }, setComic] = useState<{
    comic?: ComicsApiReturn[];
    error: any;
    loading: boolean;
  }>({
    comic: undefined,
    error: undefined,
    loading: true,
  });
  const searchParams = useSearchParams();
  const page = Number(searchParams.get("page") || 0);
  if (page < 0) {
    notFound();
  }

  useEffect(() => {
    setComic((pre) => ({ ...pre, loading: true }));
    app.api.news
      .get({
        $query: { skip: page * comicPerPage, take: comicPerPage },
      })
      .then((data) => {
        if (data.error) {
          setComic({ comic: undefined, error: data.error, loading: false });
          return;
        }
        setComic({ comic: data.data.data, error: data.error, loading: false });
      })
      .catch((err) => {
        setComic({ comic: undefined, error: err, loading: false });
      })
      .finally(() => {
        setComic((pre) => ({ ...pre, loading: false }));
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [page]);
  return (
    <>
      {loading ? (
        <Loading />
      ) : !comic && error ? (
        <div>Server have some error</div>
      ) : (
        <ListComic comics={comic!} page={page} />
      )}
    </>
  );
}
