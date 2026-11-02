import type { ReactElement } from "react";

/** Standard CSS font faces use only the caller's safe runtime mount path. */
export function FontFaces({ basePath }: { basePath: string }): ReactElement {
  const sans = `${basePath}/fonts/pretendard-subsets.css`;
  const mono = `${basePath}/fonts/GeistMono-Variable.woff2`;
  const core = `${basePath}/fonts/pretendard/PretendardVariable.subset.91.woff2`;
  return (
    <>
      <link
        rel="preload"
        href={core}
        as="font"
        type="font/woff2"
        crossOrigin="anonymous"
      />
      <link rel="stylesheet" href={sans} precedence="default" />
      <style>{`
        @font-face {
          font-family: "Geist Mono";
          src: url("${mono}") format("woff2");
          font-style: normal;
          font-weight: 100 900;
          font-display: optional;
        }
        :root {
          --font-chapchoo-sans: "Pretendard", sans-serif;
          --font-chapchoo-mono: "Geist Mono", ui-monospace, monospace;
        }
      `}</style>
    </>
  );
}
