import Image from "next/image";
import { ElysiaServerApi } from "@/typings";
import { edenTreaty } from "@elysiajs/eden";
import { use } from "react";

export default function Home() {
  const app = edenTreaty<ElysiaServerApi>("http://localhost:8080");
  const { data } = use(app.comics.get({ $query: { skip: 0, take: 10 } }));

  console.log(data);

  return (
    <main>
      <div className="w-full flex justify-center content-center my-4">
        {/* navbar */}
        <h1 className="text-3xl">this is the navbar {"=>>"}</h1>
      </div>
      <div className="grid grid-cols-4 gap-4 text-center">
        {/* content */}
        <div>Side Bar</div>
        <div className="col-span-3">
          {data?.map((comic) => (
            <div key={comic.id}>{comic.name}</div>
          ))}
        </div>
      </div>
    </main>
  );
}
