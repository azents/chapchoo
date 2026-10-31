import { defaultMascot, mascotAssets, toneColors } from "./designTokens";
import type { MascotId } from "./designTokens";

interface IconLink {
  url: string;
  sizes: string;
  type: string;
}

interface IdentityMetadata {
  icons: {
    icon: IconLink[];
    shortcut: string;
    apple: IconLink[];
  };
  manifest: string;
}

/** Browser identity remains relative to the application's runtime mount. */
export function identityMetadata(basePath: string): IdentityMetadata {
  const root = `${basePath}/design-assets/icons/${mascotAssets[defaultMascot].asset}`;
  return {
    icons: {
      icon: [16, 32, 48].map((size) => ({
        url: `${root}/favicon-${size}.png`,
        sizes: `${size}x${size}`,
        type: "image/png",
      })),
      shortcut: `${basePath}/favicon.ico`,
      apple: [
        {
          url: `${root}/apple-touch-icon.png`,
          sizes: "180x180",
          type: "image/png",
        },
      ],
    },
    manifest: `${basePath}/manifest.webmanifest`,
  };
}

interface IdentityManifest {
  id: string;
  name: string;
  short_name: string;
  start_url: string;
  scope: string;
  display: "browser";
  background_color: string;
  icons: {
    src: string;
    sizes: string;
    type: string;
    purpose: "any" | "maskable";
  }[];
}

/** Supplies home-screen artwork, without adding offline or standalone behavior. */
export function identityManifest(
  basePath: string,
  name: string,
  character: MascotId = defaultMascot,
): IdentityManifest {
  const root = `${basePath}/design-assets/icons/${mascotAssets[character].asset}`;
  return {
    id: `${basePath}/`,
    name,
    short_name: name,
    start_url: `${basePath}/`,
    scope: `${basePath}/`,
    display: "browser",
    background_color: toneColors.canvas,
    icons: [
      ...[192, 512].map((size) => ({
        src: `${root}/icon-${size}.png`,
        sizes: `${size}x${size}`,
        type: "image/png",
        purpose: "any" as const,
      })),
      {
        src: `${root}/icon-maskable-512.png`,
        sizes: "512x512",
        type: "image/png",
        purpose: "maskable",
      },
    ],
  };
}
