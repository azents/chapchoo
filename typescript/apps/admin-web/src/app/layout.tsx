import { documentCanvasBackground } from "@chapchoo/ui/designTokens";
import { FontFaces } from "@chapchoo/ui/FontFaces";
import { identityMetadata } from "@chapchoo/ui/webIdentity";
import {
  bodyColorModeBootstrap,
  colorModeBootstrap,
  preferenceCookie,
  resolvedCookie,
  resolveServerColorMode,
  toDocumentColorScheme,
} from "@chapchoo/web-core/color-mode";
import { getLocale } from "next-intl/server";
import { cookies } from "next/headers";
import { getPublicConfig } from "@/config/index";
import { AppearanceProvider } from "@/shared/appearance/AppearanceProvider";
import { DisplayProvider } from "@/shared/preferences/DisplayProvider";
import { RpcProvider } from "@/shared/rpc/provider";
import { readDisplay } from "@/shared/server/display";
import type { Metadata, Viewport } from "next";
import type { ReactElement, ReactNode } from "react";

import "@mantine/core/styles.css";
import "./globals.css";

export function generateMetadata(): Metadata {
  return {
    title: "Chapchoo Admin",
    description: "Chapchoo server account administration",
    ...identityMetadata(getPublicConfig().publicBasePath),
  };
}

export async function generateViewport(): Promise<Viewport> {
  const store = await cookies();
  const initial = resolveServerColorMode(
    store.get(preferenceCookie)?.value,
    store.get(resolvedCookie)?.value,
  );
  return {
    width: "device-width",
    initialScale: 1,
    viewportFit: "cover",
    colorScheme: toDocumentColorScheme(initial),
  };
}

export default async function RootLayout({
  children,
}: {
  children: ReactNode;
}): Promise<ReactElement> {
  const locale = await getLocale();
  const display = await readDisplay();
  const { publicBasePath } = getPublicConfig();
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
      style={{
        colorScheme: toDocumentColorScheme(initial),
        backgroundColor: documentCanvasBackground,
      }}
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
        <AppearanceProvider initial={initial}>
          <RpcProvider
            basePath={publicBasePath}
            initialIdentity={display.identity}
          >
            <DisplayProvider initial={display} basePath={publicBasePath} admin>
              {children}
            </DisplayProvider>
          </RpcProvider>
        </AppearanceProvider>
      </body>
    </html>
  );
}
