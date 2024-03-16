import { useState, useEffect, useRef } from "react";
import { AppApi } from "../typings";
import { EdenWS } from "@elysiajs/eden/treaty";

export const SideBar = ({ app }: { app: AppApi }) => {
  const [fetching, setFetching] = useState(new Set<string>());
  const wsRef = useRef<
    EdenWS<{
      body: unknown;
      params: never;
      query: unknown;
      headers: unknown;
      response: unknown;
    }>
  >();
  useEffect(() => {
    wsRef.current = app.url.fetching
      .subscribe()
      .on("message", (d) => {
        const data = d.data as any;
        if (Array.isArray(data)) {
          setFetching(new Set(data.map((d) => d.url)));
          return;
        }
        setFetching((pre) => {
          if (data.fetching) {
            pre.add(data.url);
          } else {
            pre.delete(data.url);
          }
          return new Set(pre);
        });
      })
      .on("close", () => {
        wsRef.current = app.url.fetching.subscribe().on("message", (d) => {
          const data = d.data as any;
          // if data is array
          if (Array.isArray(data)) {
            setFetching(new Set(data.map((d) => d.url)));
            return;
          }
          setFetching((pre) => {
            if (data.fetching) {
              pre.add(data.url);
            } else {
              pre.delete(data.url);
            }
            return new Set(pre);
          });
        });
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  console.log({ fetching });
  return (
    <div className="text-center" onBlur={() => setFetching(new Set())}>
      <p>Side Bar</p>
      <span className="my-2 mt-4">Server Fetching Urls</span>
      <div className="text-sm text-left p-3">
        {Array.from(fetching).map((f) => (
          <p
            key={f}
            className="p-2 bg-slate-900 my-3 break-words border rounded"
          >
            {f}
          </p>
        ))}
      </div>
    </div>
  );
};
