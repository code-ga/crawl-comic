import { MainPageCard } from "../components/Card";

export default function Page() {
  return (
    <div className="w-full my-3">
      <div className="w-full text-center text-3xl">
        Bạn có thể giúp chúng về truyện bằng cách:
      </div>
      <div>
        <MainPageCard>
          <p className="text-2xl text-blue-500">
            Thêm nguồn truyện bằng URL (chỉ hỗ trợ blogtruyenmoi.com,
            nettruyenff.com)
          </p>
          p/s từ dev: chủ yếu là lấy truyện cho mấy ông đọc thôi
        </MainPageCard>
        <MainPageCard>
          <p className="text-2xl text-blue-500">
            Phân loại các truyện về các nhóm
          </p>
          <p>
            p/s: vì có nhiều truyện bị trùng nên cần phân loại các truyện bị
           trùng thành 1 nhóm
          </p>
          <br></br>
          <p>p/s từ dev: vì chúng tui ko có LLM để dùng</p>
        </MainPageCard>
      </div>
    </div>
  );
}
