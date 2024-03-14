/** @type {import('next').NextConfig} */
const nextConfig = {
  images: {
    unoptimized: true,
    remotePatterns: [
      {
        protocol: "https",
        hostname: "ai-datalake.nz.io.vn",
      },
    ],
  },
};

export default nextConfig;
