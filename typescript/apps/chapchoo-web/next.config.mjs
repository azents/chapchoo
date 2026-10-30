import { resolve } from "node:path";
import {
  PHASE_DEVELOPMENT_SERVER,
  PHASE_PRODUCTION_SERVER,
} from "next/constants.js";

/** @returns {Promise<import("next").NextConfig>} */
export default async function nextConfig(phase) {
  const shared = {
    reactStrictMode: true,
    transpilePackages: ["@chapchoo/ui", "@chapchoo/web-core"],
    serverExternalPackages: ["@grpc/grpc-js"],
    allowedDevOrigins: ["127.0.0.1"],
  };
  if (phase === PHASE_PRODUCTION_SERVER) {
    const { getConfig } = await import("./src/config/index.ts");
    return { ...shared, assetPrefix: getConfig().basePath };
  }
  const { default: createNextIntlPlugin } = await import("next-intl/plugin");
  const development = {};
  if (phase === PHASE_DEVELOPMENT_SERVER) {
    const { getConfig } = await import("./src/config/index.ts");
    development.basePath = getConfig().basePath;
  }
  return createNextIntlPlugin("./src/i18n/request.ts")({
    ...shared,
    ...development,
    webpack(config, { isServer }) {
      if (!isServer && phase !== PHASE_DEVELOPMENT_SERVER) {
        config.output.publicPath = "auto";
      }
      return config;
    },
    output: "standalone",
    outputFileTracingRoot: resolve(import.meta.dirname, "../.."),
    outputFileTracingIncludes: {
      "/*": [
        "./server.mjs",
        "./next.config.mjs",
        "./src/config/index.ts",
        "./node_modules/@chapchoo/web-core/package.json",
        "./node_modules/@chapchoo/web-core/dist/config.js",
        "./node_modules/next/**/*",
      ],
    },
  });
}
