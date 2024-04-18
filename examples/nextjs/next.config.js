const path = require('node:path');

const plugin = process.env.USE_LOCAL_PLUGIN_BINARY
    ? path.join(__dirname, '../../target/wasm32-wasi/release/lingui_macro_plugin.wasm')
    : '@lingui/swc-plugin';


/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  i18n: {
    locales: ["en", "cs"],
    defaultLocale: 'en',
  },
  experimental: {
    swcPlugins: [
      [plugin, {}],
    ],
  },
};

module.exports = nextConfig;
