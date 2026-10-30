import { FontFaces } from "@chapchoo/ui/FontFaces";
import {
  bodyColorModeBootstrap,
  colorModeBootstrap,
  preferenceCookie,
  resolvedCookie,
  resolveServerColorMode,
} from "@chapchoo/web-core/color-mode";
import { NextIntlClientProvider } from "next-intl";
import { getLocale, getMessages } from "next-intl/server";
import { cookies } from "next/headers";
import { getPublicConfig } from "@/config";
import { AppearanceProvider } from "@/shared/appearance/AppearanceProvider";
import { RpcProvider } from "@/shared/rpc/provider";
import type { Metadata, Viewport } from "next";
import type { ReactElement, ReactNode } from "react";

import "@mantine/core/styles.css";
import "./globals.css";

export const metadata: Metadata = {
  title: "chapchoo",
  description: "Chapchoo accounts and team spaces",
};

export const viewport: Viewport = {
  width: "device-width",
  initialScale: 1,
  viewportFit: "cover",
};

export default async function RootLayout({
  children,
}: {
  children: ReactNode;
}): Promise<ReactElement> {
  const { publicBasePath } = getPublicConfig();
  const locale = await getLocale();
  const messages = await getMessages();
  const store = await cookies();
  const initial = resolveServerColorMode(
    store.get(preferenceCookie)?.value,
    store.get(resolvedCookie)?.value,
  );
  return (
    <html
      lang={locale}
      data-mantine-color-scheme={initial.mode}
      data-color-mode-preference={initial.preference}
      suppressHydrationWarning
    >
      <head>
        <FontFaces basePath={publicBasePath} />
        <script
          dangerouslySetInnerHTML={{ __html: colorModeBootstrap(initial) }}
        />
      </head>
      <body data-color-mode={initial.mode} suppressHydrationWarning>
        <script dangerouslySetInnerHTML={{ __html: bodyColorModeBootstrap }} />
        <NextIntlClientProvider locale={locale} messages={messages}>
          <AppearanceProvider initial={initial}>
            <RpcProvider basePath={publicBasePath}>{children}</RpcProvider>
          </AppearanceProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
