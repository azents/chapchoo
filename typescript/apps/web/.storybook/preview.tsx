import { FontFaces } from "@chapchoo/ui/FontFaces";
import { AppMantineProvider } from "@chapchoo/ui/provider";
import { NextIntlClientProvider } from "next-intl";
import en from "../messages/en-US.json";
import fr from "../messages/fr-FR.json";
import ja from "../messages/ja-JP.json";
import ko from "../messages/ko-KR.json";
import { isSupportedLocale } from "../src/shared/lib/locale";
import type { Preview } from "@storybook/nextjs-vite";

import "@mantine/core/styles.css";
import "../src/app/globals.css";

const messages = { "en-US": en, "ko-KR": ko, "ja-JP": ja, "fr-FR": fr };

const preview: Preview = {
  parameters: { layout: "fullscreen", nextjs: { appDirectory: true } },
  globalTypes: {
    locale: {
      toolbar: { items: ["en-US", "ko-KR", "ja-JP", "fr-FR"] },
    },
    colorScheme: {
      toolbar: { items: ["light", "dark"] },
    },
  },
  initialGlobals: { locale: "en-US", colorScheme: "light" },
  decorators: [
    (Story, context) => {
      const value: unknown = context.globals.locale;
      const locale =
        typeof value === "string" && isSupportedLocale(value) ? value : "en-US";
      if (context.parameters.ownsAppearanceProvider === true) {
        return (
          <NextIntlClientProvider locale={locale} messages={messages[locale]}>
            <FontFaces basePath="" />
            <Story />
          </NextIntlClientProvider>
        );
      }
      return (
        <NextIntlClientProvider locale={locale} messages={messages[locale]}>
          <AppMantineProvider
            forceColorScheme={
              context.globals.colorScheme === "dark" ? "dark" : "light"
            }
          >
            <FontFaces basePath="" />
            <Story />
          </AppMantineProvider>
        </NextIntlClientProvider>
      );
    },
  ],
};

export default preview;
