import { edenTreaty } from "@elysiajs/eden";
import { ElysiaServerApi } from "../../../typings";
import { use } from "react";
import { notFound } from "next/navigation";
import Image from "next/image";
import Link from "next/link";

export default function Page({ params }: { params: { id: string } }) {
  const app = edenTreaty<ElysiaServerApi>("https://ai-datalake.nz.io.vn/");

  const { data, error } = use(app.api.comic[params.id].get());
  if (!data || error || !data.data) {
    notFound();
    return;
  }
  const comic = data.data;
  const refetchComicInfo = async () => {
    await app.api.refetch.comic.info[comic.id].get();
  }
  return (
    <div className="flex flex-col gap-2 bg-slate-900 m-3 justify-center content-center text-center">
      <h1 className="text-2xl mb-2">{comic.name}</h1>
      <div className="text-sm mb-2">
        Cập nhập cuối lúc : {comic.updatedDate.toLocaleString()}
      </div>
      <div className="grid grid-cols-4">
        <div className="col-span-1">
          <Image
            src={"/api/images?url=" + comic.thumbnail}
            alt={comic.name}
            width={300}
            height={300}
          />
        </div>
        <div className="col-span-3 text-start ml-4">
          <div className="text-md mb-2 flex">
            <span className="text-lg">Thể loại: </span>
            {Object.keys(comic.genre).map((genre) => (
              <span
                key={genre}
                className="border border-slate-700 bg-slate-700 mx-2 p-1 rounded-lg"
              >
                {genre}
              </span>
            ))}
          </div>
          <div className="text-md mb-2">
            <span>Trạng Thái : </span>
            {comic.status ? comic.status : "Unknown"}
          </div>
          <div className="text-md mb-2">
            <span>Tác giả : </span>
            {comic.author ? Object.keys(comic.author).join(", ") : "Unknown"}
          </div>
          <div className="text-md mb-2">
            <span>Fetched From: </span>
            <Link href={comic.url} className="text-blue-500" target="_blank">
              {comic.url}
            </Link>
          </div>
          {Object.keys(comic.translatorTeam).length > 0 && (
            <div className="text-md mb-2">
              <span>Team dịch: </span>
              {comic.translatorTeam
                ? Object.keys(comic.translatorTeam).map((team) => (
                    <span
                      key={team}
                      className="border border-slate-700 bg-slate-700 mx-2 p-1 rounded-lg"
                    >
                      {team}
                    </span>
                  ))
                : "Unknown"}
            </div>
          )}
          {comic.anotherName.length > 0 && (
            <div className="text-md mb-2">
              <span>Tên khác: </span>
              {comic.anotherName.map((name) => (
                <span key={name}>{name}</span>
              ))}
            </div>
          )}
          {Object.keys(comic.source).length > 0 && (
            <div className="text-md mb-2">
              <span>Source: </span>
              {Object.keys(comic.source).map((source) => (
                <span
                  key={source}
                  className="border border-slate-700 bg-slate-700 mx-2 p-1 rounded-lg"
                >
                  {source}
                </span>
              ))}
            </div>
          )}
          <div className="text-md mb-2 flex">
            <div className="text-lg">Đăng Bởi: </div>
            {comic.postedBy
              ? Object.keys(comic.postedBy).map((author) => (
                  <div
                    key={author}
                    className="border border-slate-700 bg-slate-700 mx-2 p-1 rounded-lg"
                  >
                    {author}
                  </div>
                ))
              : "Unknown"}
          </div>
          <div className="text-md mb-2">
            <span className="text-md">Tạo ngày : </span>
            {comic.createdDate.toLocaleString()}
          </div>
          <div className="text-md mb-2 mt-7">
            {" "}
            <Link
              href={`/chapter/${comic.Chapter[0].id}`}
              className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md mx-3"
            >
              Đọc từ đầu
            </Link>
            <Link
              href={`/chapter/${comic.Chapter[0].id}`}
              className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md mx-3"
            >
              Đọc mới nhất
            </Link>
            <Link
              href={`/chapter/${comic.Chapter[0].id}`}
              className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md mx-3"
            >
              Thêm vô nhìn cho đủ chứ vẫn là đọc mới nhất
            </Link>
          </div>
        </div>
      </div>
      <div className="text-md mb-2">
        <div className="text-lg">Nội dung</div>
        <div>{comic.content ? comic.content : "No content available"}</div>
      </div>
    </div>
  );
}
