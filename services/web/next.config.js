/** @type {import('next').NextConfig} */
const nextConfig = {
  async rewrites() {
    return [
      {
        source: '/api/crawl',
        destination: 'http://crawler:8000/crawl',
      },
      {
        source: '/api/crawl/status/:path*',
        destination: 'http://crawler:8000/job-status/:path*',
      },
    ]
  },
}

module.exports = nextConfig