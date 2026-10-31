import { mascotArtwork } from "./mascotArtwork";
import type { MascotCharacter } from "./mascotArtwork";
import type { ReactElement } from "react";

export interface MascotArtworkProps {
  character: MascotCharacter;
  /** Caller-owned asset directory, independent of the deployment mount. */
  assetBasePath: string;
  size?: number;
}

/** Static reference art. Cropping aligns the ground without changing the master. */
export function MascotArtwork({
  character,
  assetBasePath,
  size = 72,
}: MascotArtworkProps): ReactElement {
  const artwork = mascotArtwork[character];
  return (
    <svg
      viewBox={artwork.viewBox}
      width={size}
      height={(size * artwork.height) / artwork.width}
      aria-hidden="true"
      style={{ display: "block", overflow: "hidden" }}
    >
      <image
        href={`${assetBasePath}/${artwork.file}`}
        width={1254}
        height={1254}
      />
    </svg>
  );
}
