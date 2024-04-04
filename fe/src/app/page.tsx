"use client";
import { ElysiaServerApi } from "@/typings";
import { edenTreaty } from "@elysiajs/eden";
import { Suspense } from "react";
import { ListComic } from "../components/ListComic";
import { beUrl } from "../constant";
import { Loading } from "../components/loading";
import { SideBar } from "../components/Sidebar";

export default function Home() {
  const app = edenTreaty<ElysiaServerApi>(beUrl);

  return (
    <div className="grid grid-cols-4 gap-4 text-center">
      {/* content */}
      <div className="hidden md:flex w-full">
        <Suspense fallback={<Loading />}>
          <SideBar app={app}></SideBar>
        </Suspense>
      </div>
      <Suspense fallback={<Loading />}>
        <ListComic app={app} />
      </Suspense>
    </div>
  );
}
