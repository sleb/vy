/** @type {import('next').NextConfig} */
const nextConfig = {
  experimental: {
    appDir: true,
  },
  async rewrites() {
    return [
      {
        source: '/api/vy/:path*',
        destination: process.env.VY_API_URL ? `${process.env.VY_API_URL}/:path*` : 'http://localhost:3001/api/:path*',
      },
    ];
  },
  env: {
    VY_API_URL: process.env.VY_API_URL || 'http://localhost:3001',
  },
};

module.exports = nextConfig;
