import Link from "next/link";
import React from "react";

export default function Template({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <main className="w-full min-h-screen">
      <div className="w-full flex justify-center content-center py-4 mb-4 bg-slate-900">
        {/* navbar */}
        <h1 className="text-3xl">this is the navbar {"=>>"}</h1>
        <Link href="/">Back to home here</Link>
      </div>
      {children}
      <div className="w-full flex justify-center content-center py-4 bg-slate-900 h-full">
        {/* footer */}
        <h1 className="text-3xl">this is the footer {"=>>"}</h1>
      </div>
    </main>
  );
}
