import Link from "next/link";
import React from "react";
import { ToastContainer } from "react-toastify";

export default function Template({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <main className="w-full min-h-screen">
      <ToastContainer
        position="bottom-center"
        autoClose={5000}
        hideProgressBar={false}
        newestOnTop={false}
        closeOnClick
        rtl={false}
        pauseOnFocusLoss
        draggable
        pauseOnHover={false}
        theme="dark"
      />

      <div className="w-full flex justify-center content-center py-4 mb-4 bg-slate-900">
        {/* navbar */}
        <h1 className="text-3xl">this is the navbar {"=>>"}</h1>
        <Link href="/">Back to home here</Link>
      </div>
      {children}
      <div className="w-full flex justify-center content-center py-4 bg-slate-900 ">
        {/* footer */}
        <h1 className="text-3xl">this is the footer {"=>>"}</h1>
      </div>
    </main>
  );
}
