import { ColorSchemeScript } from "@mantine/core";
import { GeistMono } from "geist/font/mono";
import { GeistSans } from "geist/font/sans";
import { NextIntlClientProvider } from "next-intl";
import { getLocale, getMessages } from "next-intl/server";
import { AppMantineProvider } from "@/shared/providers/mantine";
import type { Metadata, Viewport } from "next";
import type { ReactElement, ReactNode } from "react";

import "@mantine/core/styles.css";
import "./globals.css";

export const metadata: Metadata = {
  title: "chapchoo",
  description: "chapchoo development scaffold",
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
            {children}
          </AppMantineProvider>
        </NextIntlClientProvider>
      </body>
    </html>
  );
}
