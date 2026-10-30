import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import tseslint from "typescript-eslint";
import { defineConfig } from "eslint/config";

const root = dirname(fileURLToPath(import.meta.url));

export default defineConfig([
  tseslint.configs.recommendedTypeChecked,
  {
    files: ["src/**/*.{ts,tsx,mts}"],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: root,
      },
    },
    rules: {
      "@typescript-eslint/explicit-function-return-type": "error",
      "@typescript-eslint/no-non-null-assertion": "error",
      "@typescript-eslint/no-unnecessary-condition": "error",
      "@typescript-eslint/consistent-type-assertions": [
        "error",
        { assertionStyle: "never" },
      ],
      curly: ["error", "all"],
      eqeqeq: ["error", "always"],
      "no-restricted-imports": [
        "error",
        {
          patterns: [
            {
              group: [
                "@chapchoo/*",
                "@chapchoo/*/**",
                "next",
                "next/*",
                "next-intl",
                "@trpc/*",
                "**/apps/**",
              ],
              message:
                "Shared UI accepts presentation data and callbacks only.",
            },
          ],
        },
      ],
    },
  },
]);
