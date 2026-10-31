import { mascotAssets } from "@chapchoo/ui/designTokens";
import type { MascotId } from "@chapchoo/ui/designTokens";

/** Tight alpha bounds of the existing 1254-square transparent masters. */
export const mascotArtwork = {
  retriever: {
    file: `transparent/${mascotAssets.retriever.asset}.png`,
    viewBox: "37 167 1188 936",
    width: 1188,
    height: 936,
  },
  bichon: {
    file: `transparent/${mascotAssets.bichon.asset}.png`,
    viewBox: "35 258 1201 798",
    width: 1201,
    height: 798,
  },
  corgi: {
    file: `transparent/${mascotAssets.corgi.asset}.png`,
    viewBox: "32 176 1203 892",
    width: 1203,
    height: 892,
  },
  dachshund: {
    file: `transparent/${mascotAssets.dachshund.asset}.png`,
    viewBox: "22 249 1219 750",
    width: 1219,
    height: 750,
  },
  husky: {
    file: `transparent/${mascotAssets.husky.asset}.png`,
    viewBox: "39 201 1193 863",
    width: 1193,
    height: 863,
  },
  collie: {
    file: `transparent/${mascotAssets.collie.asset}.png`,
    viewBox: "64 245 1167 794",
    width: 1167,
    height: 794,
  },
} satisfies Record<
  MascotId,
  { file: string; viewBox: string; width: number; height: number }
>;

export type MascotCharacter = keyof typeof mascotArtwork;
