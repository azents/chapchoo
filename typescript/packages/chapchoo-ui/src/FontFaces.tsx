import type { ReactElement } from "react";

/** Standard CSS font faces use only the caller's safe runtime mount path. */
export function FontFaces({ basePath }: { basePath: string }): ReactElement {
  const sans = `${basePath}/fonts/Geist-Variable.woff2`;
  const mono = `${basePath}/fonts/GeistMono-Variable.woff2`;
  return (
    <>
      <link
        rel="preload"
        href={sans}
        as="font"
        type="font/woff2"
        crossOrigin=""
      />
      <link
        rel="preload"
        href={mono}
        as="font"
        type="font/woff2"
        crossOrigin=""
      />
      <style>{`
        @font-face {
          font-family: "Geist Sans";
          src: url("${sans}") format("woff2");
          font-style: normal;
          font-weight: 100 900;
          font-display: swap;
        }
        @font-face {
          font-family: "Geist Mono";
          src: url("${mono}") format("woff2");
          font-style: normal;
          font-weight: 100 900;
          font-display: swap;
        }
        :root {
          --font-geist-sans: "Geist Sans", Arial, sans-serif;
          --font-geist-mono: "Geist Mono", ui-monospace, monospace;
        }
      `}</style>
    </>
  );
}
