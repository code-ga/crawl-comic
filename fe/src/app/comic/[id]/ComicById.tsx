"use client";
import { edenTreaty } from "@elysiajs/eden";
import Link from "next/link";
import { notFound } from "next/navigation";
import { useEffect, useState } from "react";
import { Bounce, toast, ToastContainer } from "react-toastify";
import { Loading } from "../../../components/loading";
import { beUrl, cdnUrl } from "../../../constant";
import { ComicIncludeChapter, ElysiaServerApi } from "../../../typings";
import Image from "next/image"

export function Comic({ params }: { params: { id: string } }) {
  const app = edenTreaty<ElysiaServerApi>(beUrl);
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
        setComic({ data: data.data.data, error: data.error, loading: false });
      })
      .catch((err) => {
        setComic({ data: null, error: err, loading: false });
      })
      .finally(() => {
        setComic((pre) => ({ ...pre, loading: false }));
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [params.id]);
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
      (e.target as HTMLButtonElement).disabled = false;
      return;
    }
    setComic((pre) => ({ ...pre, data: comic }));
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
    console.log({ data, error });
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
      return;
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
      <div className="flex flex-col gap-2 md:p-3 py-4 bg-slate-900 m-3 justify-center content-center text-center rounded-lg shadow-2xl shadow-slate-500 border border-slate-700 hover:border-slate-500">
        <h1 className="text-2xl mb-2">{comic.name}</h1>
        <div className="text-sm mb-2">
          Cập nhập cuối lúc : {comic.updatedDate.toLocaleString()}
        </div>
        <div className="grid md:grid-cols-4 sm:grid-cols-1">
          <div className="md:col-span-1 mx-2 sm:col-span-1 sm:my-1 md:my-0 flex justify-center content-center">
            <Image
              src={cdnUrl + "/image?url=" + comic.thumbnail}
              alt={comic.name}
              width={300}
              height={300}
            />
          </div>
          <div className="md:col-span-3 sm:col-span-1 text-start ml-4">
            <div className="text-md mb-2 mt-2 flex text-wrap">
              <span className="text-lg">Thể loại: </span>
              <br />
              <p className="text-center mx-2 md:mx-1 break-words">
                {Object.keys(comic.genre).map((genre, index) => (
                  <span
                    key={genre}
                    className="border border-slate-700 bg-slate-700 mb-2 md:mb-0 md:mx-2 p-1 rounded-lg block md:inline"
                  >
                    {genre}
                  </span>
                ))}
              </p>
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
                        className="border border-slate-700 bg-slate-700 mx-2 p-1 rounded-lg "
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
              <div className="text-md mb-2 break-words">
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
              {comic.Chapter[0] && (
                <Link
                  href={`/chapter/${comic.Chapter[0].id}`}
                  className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md md:mx-3 mr-3 my-4 inline-block"
                >
                  Đọc từ đầu
                </Link>
              )}
              {comic.Chapter[comic.Chapter.length - 1] && (
                <Link
                  href={`/chapter/${
                    comic.Chapter[comic.Chapter.length - 1].id
                  }`}
                  className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md md:mx-3 my-4 inline-block"
                >
                  Đọc mới nhất
                </Link>
              )}
              <button
                className="bg-red-700 p-3 px-5 border border-slate-700 rounded-md md:mx-3 my-4 inline-block"
                onClick={(e) => refetchComicInfo(e)}
              >
                Refetch Comic
              </button>
            </div>
          </div>
        </div>
        <div className="text-md mb-2 break-words mx-2">
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
            Refetch Chapters
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
              {comic.Chapter.sort((a, b) => b.index - a.index).map(
                (chapter) =>
                  chapter && (
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
                  )
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
