import Link from "next/link";
import { MainPageCard } from "./components/Card";

export default function Page() {
  return (
    <div className="w-full my-3">
      <div className="w-full text-center text-3xl">
        Bạn có thể giúp chúng bằng cách:
      </div>
      <div>
        <MainPageCard>
          <Link className="text-2xl text-blue-500" href={"/help/chapter"}>Hỗ trợ về Chapter</Link>
        </MainPageCard>
        <MainPageCard>
          <Link className="text-2xl text-blue-500" href={"/help/comic"}>Hỗ trợ về truyện</Link>
        </MainPageCard>
      </div>
    </div>
  );
}
