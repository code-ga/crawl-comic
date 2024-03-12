"use client";
import { edenTreaty } from "@elysiajs/eden";
import { ComicIncludeChapter, ElysiaServerApi } from "../../../typings";
import { useEffect, useState } from "react";
import { notFound } from "next/navigation";
import Image from "next/image";
import Link from "next/link";
import { beUrl, cdnUrl } from "../../../constant";
import { useRouter } from "next/navigation";
import { Bounce, toast } from "react-toastify";
import { ToastContainer } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";
import { Loading } from "../../../components/loading";

export default function Page({ params }: { params: { id: string } }) {
  const app = edenTreaty<ElysiaServerApi>(beUrl);
  const router = useRouter();
  const [{ data, error, loading }, setComic] = useState<{
    data?: ComicIncludeChapter | null;
    error: any;
    loading: boolean;
  }>({
    data: null,
    error: null,
    loading: true,
  });
  useEffect(() => {
    app.api.comic[params.id]
      .get()
      .then((data) => {
        if (data.error) {
          setComic({ data: null, error: data.error, loading: false });
          return;
        }
        if (data.data.status === 404) {
          notFound();
        }
        console.log({ data });
        setComic({ data: data.data.data, error: data.error, loading: false });
      })
      .catch((err) => {
        setComic({ data: null, error: err, loading: false });
      })
      .finally(() => {
        setComic((pre) => ({ ...pre, loading: false }));
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  if (loading) {
    return <Loading />;
  }
  if (!data && error) {
    notFound();
  }
  const comic = data!;
  console.log({ comic });
  const refetchComicInfo = async (
    e: React.MouseEvent<HTMLButtonElement, MouseEvent>
  ) => {
    e.preventDefault();
    (e.target as HTMLButtonElement).disabled = true;
    const { error } = await app.api.refetch.comic.info[comic.id].get();
    if (error) {
      toast.error("Update thất bại", {
        position: "bottom-center",
        autoClose: 5000,
        hideProgressBar: false,
        closeOnClick: true,
        pauseOnHover: false,
        draggable: true,
        progress: undefined,
        theme: "dark",
      });
    }
    router.refresh();
    toast.success("Update thành công", {
      position: "bottom-center",
      autoClose: 5000,
      hideProgressBar: false,
      closeOnClick: true,
      pauseOnHover: false,
      draggable: true,
      progress: undefined,
      theme: "dark",
    });
    (e.target as HTMLButtonElement).disabled = false;
  };

  const refetchChapterList = async (e: React.MouseEvent<HTMLButtonElement>) => {
    e.preventDefault();
    console.log("refetch");
    const { data, error } = await app.api.refetch.comic.chaps[comic.id].get();
    if (error) {
      toast.error("Update thất bại", {
        position: "bottom-center",
        autoClose: 5000,
        hideProgressBar: false,
        closeOnClick: true,
        pauseOnHover: false,
        draggable: true,
        progress: undefined,
        theme: "dark",
      });
    }
    if (data) {
      toast.success(data?.data?.message || "Update thành công", {
        position: "bottom-center",
        autoClose: 5000,
        hideProgressBar: false,
        closeOnClick: true,
        pauseOnHover: false,
        draggable: true,
        progress: undefined,
        theme: "dark",
      });
    }
    console.log({ data, error });
  };
  return (
    <div className="m-3">
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
        transition={Bounce}
      />
      <div className="flex flex-col gap-2 p-3 bg-slate-900 m-3 justify-center content-center text-center rounded-lg shadow-2xl shadow-slate-500 border border-slate-700 hover:border-slate-500">
        <h1 className="text-2xl mb-2">{comic.name}</h1>
        <div className="text-sm mb-2">
          Cập nhập cuối lúc : {comic.updatedDate.toLocaleString()}
        </div>
        <div className="grid grid-cols-4">
          <div className="col-span-1 ml-2">
            <Image
              src={cdnUrl + "/image?url=" + comic.thumbnail}
              alt={comic.name}
              width={300}
              height={300}
            />
          </div>
          <div className="col-span-3 text-start ml-4">
            <div className="text-md mb-2 flex text-wrap">
              <span className="text-lg">Thể loại: </span>
              {Object.keys(comic.genre).map((genre, index) => (
                <p key={genre}>
                  <span className="border border-slate-700 bg-slate-700 mx-2 p-1 rounded-lg">
                    {genre}
                  </span>
                </p>
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
              <button
                className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md mx-3"
                onClick={(e) => refetchComicInfo(e)}
              >
                Refetch Comic
              </button>
            </div>
          </div>
        </div>
        <div className="text-md mb-2">
          <div className="text-lg">Nội dung</div>
          <div>{comic.content ? comic.content : "No content available"}</div>
        </div>
      </div>
      <div className="text-md m-3 mt-5 bg-slate-900 p-3 rounded-md border border-slate-700">
        <div className="text-lg">
          <span>Chapters</span>
          <button
            className="bg-red-700 p-1 px-3 border border-slate-700 rounded-md mx-3"
            onClick={(e) => refetchChapterList(e)}
          >
            Refetch Chapter
          </button>
        </div>
        <div>
          <table className="table-auto">
            <thead>
              <tr>
                <th>Chapter</th>
                <th>Tạo ngày</th>
              </tr>
            </thead>
            <tbody>
              {comic.Chapter.map((chapter) => (
                <tr key={chapter.id}>
                  <td>
                    <Link
                      href={`/chapter/${chapter.id}`}
                      className="text-blue-500"
                    >
                      {chapter.name}
                    </Link>
                  </td>
                  <td>{chapter.createdDate}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
