import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Suppress hydration warnings for browser extensions
  reactStrictMode: true,
  
  webpack: (config, { isServer }) => {
    // Enable async WebAssembly
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
    };

    // Handle .wasm files
    config.module.rules.push({
      test: /\.wasm$/,
      type: "webassembly/async",
    });

    // Add specific rule for WASM files in node_modules
    config.module.rules.push({
      test: /\.wasm$/,
      include: /node_modules/,
      type: "asset/resource",
      generator: {
        filename: "static/wasm/[name].[hash][ext]",
      },
    });

    // Fallback for Node.js modules in client-side bundles
    if (!isServer) {
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        path: false,
        crypto: false,
      };
    }

    // Add alias for the WASM file
    config.resolve.alias = {
      ...config.resolve.alias,
      "luminair_web_bg.wasm": require.resolve("@gizatech/luminair-web/luminair_web_bg.wasm"),
    };

    return config;
  },
};

export default nextConfig;
