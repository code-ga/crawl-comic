/** @type {import('next').NextConfig} */
const nextConfig = {
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "ai-datalake.nz.io.vn",
      },
    ],
  },
};

export default nextConfig;
