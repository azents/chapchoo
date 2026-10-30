import { defineConfig } from "eslint/config";
import tseslint from "typescript-eslint";
import prettier from "eslint-plugin-prettier/recommended";

export default defineConfig([
  { ignores: ["test-results/**", "artifacts/**"] },
  {
    files: ["src/**/*.ts", "playwright.config.ts"],
    extends: [tseslint.configs.recommendedTypeChecked],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      curly: ["error", "all"],
      "@typescript-eslint/no-non-null-assertion": "error",
    },
  },
  prettier,
]);
