/** Shared web tokens validated against canonical values in root DESIGN.md. */
export const toneColors = {
  canvas: "#fdfcf8",
  navigation: "#f3f2e9",
  surface: "#ffffff",
  overlay: "#ffffff",
  surfaceWarm: "#fffefa",
  selected: "#e9eadf",
  border: "#e1e3d7",
  borderStrong: "#9aab82",
  ink: "#353a36",
  secondary: "#626c58",
  muted: "#707765",
  accent: "#edca6e",
  accentHover: "#e5bd54",
  onAccent: "#51431e",
  accentText: "#51431e",
  accentSoft: "#fbf2d0",
  success: "#4f6b3b",
  successSoft: "#edf3e6",
  warning: "#805a1e",
  warningSoft: "#fbf0d8",
  danger: "#a24237",
  dangerSoft: "#fbece8",
  focus: "#866716",
} satisfies Record<string, string>;

/** Adaptive warm olive-charcoal appearance; light metrics remain unchanged. */
export const darkToneColors: Record<keyof typeof toneColors, string> = {
  canvas: "#181c18",
  navigation: "#1d221c",
  surface: "#222820",
  overlay: "#2c3528",
  surfaceWarm: "#20261f",
  selected: "#333d2d",
  border: "#3f4a3b",
  borderStrong: "#77856f",
  ink: "#e6e9df",
  secondary: "#b8c0af",
  muted: "#a0ab94",
  accent: "#d9bc78",
  accentHover: "#e4ca8b",
  onAccent: "#272718",
  accentText: "#e5c987",
  accentSoft: "#393421",
  success: "#b4ce98",
  successSoft: "#293623",
  warning: "#e5c282",
  warningSoft: "#3b3222",
  danger: "#edb1a5",
  dangerSoft: "#402b27",
  focus: "#edca6e",
};

/** Inline root canvas remains available before external CSS or hydration. */
export const documentCanvasBackground = `light-dark(${toneColors.canvas}, ${darkToneColors.canvas})`;

export const toneMetrics = {
  space: { xs: 4, sm: 8, md: 12, lg: 16, xl: 24, xxl: 32 },
  radius: { control: 12, surface: 18, avatar: 24 },
  type: { caption: 12, small: 14, body: 16, title: 24, display: 32 },
  layout: {
    navigation: 258,
    readingMeasure: 790,
    mobileGutter: 16,
    touchTarget: 44,
  },
};

export type ReferencePlatform = "web" | "ios";
export const platformFonts: Record<ReferencePlatform, string> = {
  web: "Pretendard, sans-serif",
  ios: 'Helvetica, "Apple SD Gothic Neo", sans-serif',
};

export const mascotIds = [
  "retriever",
  "bichon",
  "corgi",
  "dachshund",
  "husky",
  "collie",
] as const;
export type MascotId = (typeof mascotIds)[number];

export const defaultMascot: MascotId = "retriever";

export const mascotAssets = {
  bichon: { name: "비숑", asset: "01-bichon" },
  corgi: { name: "코기", asset: "02-corgi" },
  dachshund: { name: "닥스훈트", asset: "03-dachshund" },
  husky: { name: "허스키", asset: "04-husky" },
  collie: { name: "보더콜리", asset: "05-border-collie" },
  retriever: { name: "리트리버", asset: "06-retriever" },
} satisfies Record<MascotId, { name: string; asset: string }>;

export const mascots = Object.entries(mascotAssets).map(([key, value]) => ({
  key,
  name: value.name,
  file: `transparent/${value.asset}.png`,
}));
