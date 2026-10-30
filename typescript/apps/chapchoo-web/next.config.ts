import { resolve } from "node:path";
import createNextIntlPlugin from "next-intl/plugin";
import type { NextConfig } from "next";
import { getBuildConfig } from "./src/config/index";

const withNextIntl = createNextIntlPlugin("./src/i18n/request.ts");

const { basePath } = getBuildConfig();

const nextConfig: NextConfig = {
  basePath,
  env: { CHAPCHOO_COMPILED_BASE_PATH: basePath },
  reactStrictMode: true,
  output: "standalone",
  outputFileTracingRoot: resolve(import.meta.dirname, "../.."),
  transpilePackages: ["@chapchoo/ui", "@chapchoo/web-core"],
  serverExternalPackages: ["@grpc/grpc-js"],
  allowedDevOrigins: ["127.0.0.1"],
};

export default withNextIntl(nextConfig);
