import Link from "next/link";
import React from "react";
import { Search } from "../components/Search";

export default function Template({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <main className="w-full min-h-screen">
      <div className="w-full flex justify-between p-3 text-center content-center py-4 mb-4 bg-slate-900">
        {/* navbar */}
        <Link href="/">Back to home here</Link>
        <Search></Search>
        {/* <h1 className="text-3xl">this is the navbar {"=>>"}</h1> */}
        <div className="text-center">
          <div>viết gì đây</div>
          <div>
            <Link
              href="/help"
              title="bạn có thể giúp chúng tôi tạo phụ đề cho hình ảnh được ko"
              className="text-blue-500"
              target="_blank"
            >
              giúp chúng tôi đi
            </Link>
          </div>
        </div>
      </div>
      {children}
      <footer className="w-full flex justify-center content-center py-4 bg-slate-900 h-full mt-3">
        {/* footer */}
        <h1 className="text-3xl">this is the footer {"=>>"}</h1>
      </footer>
    </main>
  );
}
