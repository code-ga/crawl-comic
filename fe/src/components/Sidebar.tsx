import { useState, useEffect } from "react";
import { AppApi } from "../typings";

export const SideBar = ({ app }: { app: AppApi }) => {
  const [fetching, setFetching] = useState(new Set<string>());
  useEffect(() => {
    app.url.fetching.subscribe().on("message", (d) => {
      const data = d.data as any;
      setFetching((pre) => {
        if (data.fetching) {
          pre.add(data.url);
        } else {
          pre.delete(data.url);
        }
        return new Set(pre);
      });
    });
  }, []);
  console.log({ fetching });
  return (
    <div className="text-center">
      <p>Side Bar</p>
      <span className="my-2 mt-4">Server Fetching Urls</span>
      <div className="text-sm text-left p-3">
        {[...fetching].map((f) => (
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
