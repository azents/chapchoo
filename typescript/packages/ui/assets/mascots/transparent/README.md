# Transparent mascot masters

These six 1254×1254 RGBA PNGs are masked derivatives of the unchanged originals
one directory above. Original prompts, generation limits and original hashes
remain in `../GENERATION.md`; these are not newly generated identities.

Prepared on 2026-10-31 KST using rembg IS-Net (`isnet-general-use`). Enclosed mask
holes were filled to preserve light ear interiors. A 1px inward MinFilter(3)
cleanup and 0.5px feather were applied at original resolution. Foreground RGB
values remain unchanged; no inversion, outline or new character detail was added.
Transparent corners have zero alpha. This is raster artwork, not editable vector
or 3D source.

The requester-supplied retriever original was added on the same date, using the
same segmentation / hole-fill / edge-cleanup method. Its foreground colors and
texture are preserved; it was not regenerated to match the smoother variations.

`manifest.json` records the original, transparent PNG and production WebP SHA-256
hashes. Production alpha-preserving 216×216 WebPs in `../../illustrations/` use
Pillow Lanczos resampling, quality 92 and method 6. Render with normal composition
on either appearance. Baked black review images are not product assets.
