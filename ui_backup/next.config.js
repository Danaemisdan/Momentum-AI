/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  distDir: 'out',
  devIndicators: { buildActivity: true },
  images: {
    unoptimized: true,
  },
  // Disable server features incompatible with static export
  trailingSlash: true,
}

module.exports = nextConfig
