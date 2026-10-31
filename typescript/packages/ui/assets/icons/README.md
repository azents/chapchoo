# Browser and home-screen identity

Each of the six characters has the same generated asset set. The default is
`01-retriever`, derived from the requester-supplied original, not a redesign.
Originals, transparent masters and generation/masking provenance live under
`../mascots/`. These are raster exports; no vector source is claimed.

- `favicon.ico`: 16 / 32 / 48px entries for conventional browser discovery.
- `favicon-{16,32,48}.png`: transparent browser icons.
- `apple-touch-icon.png`: opaque 180px image with an unrounded cream canvas;
  the OS applies its own corner treatment.
- `icon-{192,512,1024}.png`: opaque regular app/home-screen exports.
- `icon-maskable-512.png`: separate opaque maskable composition. The complete
  alpha silhouette fits in a 56%-wide square inside the central 80%-diameter
  safe circle. Ears and tail stay within that zone instead of being cropped.

Rebuild derivatives from the repository root:

```sh
uv run scripts/generate-identity-assets.py
```

The script pins Pillow 13.0.0. It contains artwork by alpha bounds, centers it,
resamples with Lanczos, and preserves foreground colors. `manifest.json` records
source and output SHA-256 digests. Regeneration does not modify original PNGs,
transparent masters or the existing 216px illustration derivatives. Use the
same pinned tool to reproduce icon bytes. The cream canvas follows the
canonical light design token in root DESIGN.md; icons are intentionally stable
across application color modes, while inline mascot illustrations retain alpha.

Both General and Admin stage every character's icon directory under
`design-assets/icons/` and publish the default at `/favicon.ico`. Shared
`identityMetadata` builds runtime-prefix-safe head links. Each app exposes a
request-time `/manifest.webmanifest` with its own name, scope and start URL.
The manifest keeps browser display behavior; it does not claim offline support,
installation eligibility or physical Apple-device verification. The 1024px
export is available for future consumers; no unimplemented native app asset
catalog is introduced.
