/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  i18n: {
    locales: ["en", "cs"],
    defaultLocale: 'en',
  },
  experimental: {
    swcPlugins: [
      ['@lingui/swc-plugin', {}],
    ],
  },
};

module.exports = nextConfig;
