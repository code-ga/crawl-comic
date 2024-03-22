import React from "react";
export function MainPageCard({ children }: { children: React.ReactNode }) {
  return <div className="bg-gray-900 p-5 m-3 rounded">{children}</div>;
}
