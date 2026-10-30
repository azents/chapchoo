import { FontFaces } from "@chapchoo/ui/FontFaces";
import { AppMantineProvider } from "@chapchoo/ui/provider";
import { ColorSchemeScript } from "@mantine/core";
import { NextIntlClientProvider } from "next-intl";
import { getLocale, getMessages } from "next-intl/server";
import { getPublicConfig } from "@/config/index";
import { RpcProvider } from "@/shared/rpc/provider";
import type { Metadata, Viewport } from "next";
import type { ReactElement, ReactNode } from "react";

import "@mantine/core/styles.css";
import "./globals.css";

export const metadata: Metadata = {
  title: "Chapchoo Admin",
  description: "Chapchoo server account administration",
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
  const locale = await getLocale();
  const messages = await getMessages();
  const { publicBasePath } = getPublicConfig();
  return (
    <html lang={locale} suppressHydrationWarning>
      <head>
        <FontFaces basePath={publicBasePath} />
        <ColorSchemeScript forceColorScheme="light" />
      </head>
      <body>
        <NextIntlClientProvider locale={locale} messages={messages}>
          <AppMantineProvider>
            <RpcProvider basePath={publicBasePath}>{children}</RpcProvider>
          </AppMantineProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
