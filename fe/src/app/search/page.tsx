"use client";
import { edenTreaty } from "@elysiajs/eden";
import { notFound, useSearchParams } from "next/navigation";
import { beUrl } from "../../constant";
import { AppApi, ComicsApiReturn, ElysiaServerApi } from "../../typings";
import { Suspense, useEffect, useState } from "react";
import { ListComic } from "../../components/ListComic";
import { Loading } from "../../components/loading";
import { SideBar } from "../../components/Sidebar";

export default function Page() {
  const app = edenTreaty<ElysiaServerApi>(beUrl);

  return (
    <div className="grid grid-cols-4 gap-4 text-center">
      {/* content */}
      <div className="hidden md:flex w-full">
        <Suspense fallback={<Loading />}>
          <SideBar app={app}></SideBar>
        </Suspense>
      </div>
      <div className="col-span-4 md:col-span-3">
        <Suspense fallback={<Loading />}>
          <Comics app={app}></Comics>
        </Suspense>
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
  const q = searchParams.get("q");
  if (!q) {
    notFound();
  }
  useEffect(() => {
    setComic((pre) => ({ ...pre, loading: true }));
    app.api.search
      .get({
        $query: { query: q },
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
  }, [q]);
  return (
    <>
      {loading ? (
        <Loading />
      ) : !comic && error ? (
        <div>Server have some error</div>
      ) : (
        <ListComic comics={comic!} />
      )}
    </>
  );
}
