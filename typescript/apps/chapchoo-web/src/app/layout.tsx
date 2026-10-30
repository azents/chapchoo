import { AppMantineProvider } from "@chapchoo/ui/provider";
import { ColorSchemeScript } from "@mantine/core";
import { GeistMono } from "geist/font/mono";
import { GeistSans } from "geist/font/sans";
import { NextIntlClientProvider } from "next-intl";
import { getLocale, getMessages } from "next-intl/server";
import { getPublicConfig } from "@/config";
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
  return (
    <html
      lang={locale}
      className={`${GeistSans.variable} ${GeistMono.variable}`}
      suppressHydrationWarning
    >
      <head>
        <ColorSchemeScript defaultColorScheme="auto" />
      </head>
      <body>
        <NextIntlClientProvider locale={locale} messages={messages}>
          <AppMantineProvider defaultColorScheme="auto">
            <RpcProvider basePath={publicBasePath}>{children}</RpcProvider>
          </AppMantineProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
