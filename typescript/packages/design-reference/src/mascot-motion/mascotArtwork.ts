/** Tight alpha bounds of the existing 1254-square transparent masters. */
export const mascotArtwork = {
  bichon: {
    file: "transparent/01-bichon.png",
    viewBox: "35 258 1201 798",
    width: 1201,
    height: 798,
  },
  corgi: {
    file: "transparent/02-corgi.png",
    viewBox: "32 176 1203 892",
    width: 1203,
    height: 892,
  },
  dachshund: {
    file: "transparent/03-dachshund.png",
    viewBox: "22 249 1219 750",
    width: 1219,
    height: 750,
  },
  husky: {
    file: "transparent/04-husky.png",
    viewBox: "39 201 1193 863",
    width: 1193,
    height: 863,
  },
  collie: {
    file: "transparent/05-border-collie.png",
    viewBox: "64 245 1167 794",
    width: 1167,
    height: 794,
  },
};

export type MascotCharacter = keyof typeof mascotArtwork;
