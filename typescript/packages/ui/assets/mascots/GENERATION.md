# Mascot generation provenance

## Tool and model

- Generation date: 2026-10-30 (KST).
- Image model identifier verified in current source: **`gpt-image-2`**.
- Source checked on 2026-10-30: public image-generation implementation at commit `f2ac9983552d187aa1c9a53c4707cf3b2c4afde6`, [`IMAGE_MODEL` constant](https://github.com/openai/codex/blob/f2ac9983552d187aa1c9a53c4707cf3b2c4afde6/codex-rs/ext/image-generation/src/tool.rs#L65), [generation request](https://github.com/openai/codex/blob/f2ac9983552d187aa1c9a53c4707cf3b2c4afde6/codex-rs/ext/image-generation/src/tool.rs#L505-L511), [edit request](https://github.com/openai/codex/blob/f2ac9983552d187aa1c9a53c4707cf3b2c4afde6/codex-rs/ext/image-generation/src/tool.rs#L554-L560). Both paths use the same constant. This identifies the current code default, not every model available through OpenAI's broader image API.
- Observed interface in this Session: `image_generation`, prompt-only input. The exact prompts below were used for the selected reduced-fur five-variant round.
- Per-execution server model/revision: **not exposed in the tool response or PNG metadata**. The source-verified default above is distinguished from unavailable historical runtime telemetry; it is not proof that an unseen server override was absent.
- No reference-image input, seed, sampler settings or model override was supplied through the visible interface. These were separate prompt-based generations, not pixel-preserving image edits.
- Outputs: PNG, 1254 × 1254 pixels, retained unchanged. A prompt alone does not guarantee the identical output on a future generation.

## 01-retriever.png — recovered original

The requester supplied `retriever-f00e65fbe81e-lossless.zip` on 2026-10-31 KST
and confirmed this is the original golden dog-blob selected before the five
breed variations. The archive's `generated-image-f00e65fbe81e.png` was copied
byte-for-byte, not regenerated, resized or re-encoded.

- Original SHA-256 (independently computed on receipt):
  `f00e65fbe81ee5004ccb3fe708113630d230a831fe8e7260eb37a43fdf3dc895`
- Original format/dimensions: RGB PNG, 1254 × 1254; 1,647,500 bytes.
- The original selection is recorded in the 2026-10-30 conversation. The exact
  original prompt and per-execution model telemetry were not recovered with the
  archive. The source-default model statement above is not execution evidence
  for this recovered file.
- Transparent derivative: rembg 2.0.86, IS-Net `isnet-general-use`, foreground
  mask threshold 127, enclosed-hole fill, MinFilter(3), GaussianBlur(0.5).
  Foreground RGB is retained from the supplied original. The same 216px Lanczos,
  WebP quality 92 / method 6 treatment supplies production art. Derivative hashes
  are in `transparent/manifest.json`.

The following sections describe the separate five-variation generation round.

## 02-bichon.png

SHA-256: `fb1b9341059b7e33dc5a450300e256268ecf128f23ee5f1ab3d9e7647767f46e`

```text
One BICHON variation of a consistent family of abstract puppy BLOBS. Main priority: MORE SOLID BLOB, MUCH LESS FUR. Absolutely NO visible hair strands, shaggy edges, curls, tufts, fleece, fibers, felt or plush fabric. Surface is smooth soft matte with only an almost imperceptible peach-skin softness, broad simple 2.5D shading, clean uninterrupted silhouette, not glossy plastic. A LOW WIDE chubby mochi lump, about 1.3 times as wide as tall. Single uninterrupted volume, NO separate head or neck, NO chest anatomy. Two tiny foot nubs continuous with bottom. Two small floppy rounded ear flaps on upper corners and one tiny curved tail on side. Simple face directly on blob surface: small cream-white oval eye shapes and flat cocoa oval pupils, tiny charcoal rounded triangular nose and short curved smile. No highlights or iris detail, no projecting snout, no cheek pads, no chin. This BICHON is one soft WHITE solid rounded blob with rounded white floppy ears. NO fluffy cloud tufts or curly fur; breed hint ONLY from white color and rounded ears. Keep very simple, squishy and gently cheerful, original creature not realistic puppy. Pale warm cream square background, faint shadow. Character fills 85 percent. No text or accessories. Single character.
```

## 03-corgi.png

SHA-256: `4f711181aed5c3b8f9c1659af671f0e918170c145a469ad6702af7bf7a8ddc35`

```text
One CORGI variation of a consistent family of abstract puppy BLOBS. Main priority: MORE SOLID BLOB, MUCH LESS FUR. Absolutely NO visible hair strands, shaggy edges, curls, tufts, fleece, fibers, felt or plush fabric. Surface is smooth soft matte with only an almost imperceptible peach-skin softness, broad simple 2.5D shading, clean uninterrupted silhouette, not glossy plastic. A LOW WIDE chubby mochi lump, about 1.3 times as wide as tall. Single uninterrupted volume, NO separate head or neck, NO chest anatomy. Two tiny foot nubs continuous with bottom. Two small upright rounded triangular corgi ears with peach inner shapes, one tiny curved tail on side. Simple face directly on blob surface: small cream-white oval eye shapes and flat cocoa oval pupils, tiny warm brown rounded triangular nose and short curved smile. No highlights or iris detail, no projecting snout, no cheek pads, no chin. This CORGI is soft apricot ORANGE with a broad CREAM lower-face/bottom color field and simple narrow white vertical forehead blaze. Color patches are FLAT coloration of single blob, never raised muzzle. Breed hint ONLY from color pattern and small upright ears. Keep very simple, squishy and gently cheerful, original creature not realistic puppy. Pale warm cream square background, faint shadow. Character fills 85 percent. No text or accessories. Single character.
```

## 04-dachshund.png

SHA-256: `4ed7b91771c4ad5b7bf5550b8b023530d854e13a733418806ec8729265c010fd`

```text
One DACHSHUND variation of a consistent family of abstract puppy BLOBS. Main priority: MORE SOLID BLOB, MUCH LESS FUR. Absolutely NO visible hair strands, shaggy edges, curls, tufts, fleece, fibers, felt or plush fabric. Surface is smooth soft matte with only an almost imperceptible peach-skin softness, broad simple 2.5D shading, clean uninterrupted silhouette, not glossy plastic. A LOW WIDE chubby mochi lump, about 1.3 times as wide as tall. Single uninterrupted volume, NO separate head or neck, NO chest anatomy. Two tiny foot nubs continuous with bottom. Two soft elongated drooping ear flaps on upper sides and one tiny curved tail on side. Simple face directly on blob surface: small cream-white oval eye shapes and flat cocoa oval pupils, tiny charcoal rounded triangular nose and short curved smile. No highlights or iris detail, no projecting snout, no cheek pads, no chin. This DACHSHUND has warm CHOCOLATE brown body and ears, two small caramel eyebrow color spots, a subtle flat caramel patch around the tiny nose and mouth. NO projecting dachshund snout. Breed hint ONLY from color pattern and longer floppy ears. Keep very simple, squishy and gently cheerful, original creature not realistic puppy. Pale warm cream square background, faint shadow. Character fills 85 percent. No text or accessories. Single character.
```

## 05-husky.png

SHA-256: `5d515389f51a0d85f2d3ae47e768eca7b0090aa5dc6b3bcf3ea6c827274ab95d`

```text
One HUSKY variation of a consistent family of abstract puppy BLOBS. Main priority: MORE SOLID BLOB, MUCH LESS FUR. Absolutely NO visible hair strands, shaggy edges, curls, tufts, fleece, fibers, felt or plush fabric. Surface is smooth soft matte with only an almost imperceptible peach-skin softness, broad simple 2.5D shading, clean uninterrupted silhouette, not glossy plastic. A LOW WIDE chubby mochi lump, about 1.3 times as wide as tall. Single uninterrupted volume, NO separate head or neck, NO chest anatomy. Two tiny foot nubs continuous with bottom. Two small upright rounded triangular husky ears, one tiny curved tail on side. Simple face directly on blob surface: small cream-white oval eye shapes and flat muted blue oval pupils, tiny charcoal rounded triangular nose and short curved smile. No highlights or iris detail, no projecting snout, no cheek pads, no chin. This HUSKY has cool silver-gray upper body, creamy white lower half and simplified symmetric white husky face mask, gray ears with cream interiors. Color patches are FLAT coloration of single blob, never raised muzzle. Breed hint ONLY from color pattern and small ears. Keep very simple, squishy and gently cheerful, original creature not realistic puppy. Pale warm cream square background, faint shadow. Character fills 85 percent. No text or accessories. Single character.
```

## 06-border-collie.png

SHA-256: `d2fa486f76d4ec2bca4839eefa0716e7cb4ec8b3054371d9746af4e425ec1c17`

```text
One BORDER COLLIE variation of a consistent family of abstract puppy BLOBS. Main priority: MORE SOLID BLOB, MUCH LESS FUR. Absolutely NO visible hair strands, shaggy edges, curls, tufts, fleece, fibers, felt or plush fabric. Surface is smooth soft matte with only an almost imperceptible peach-skin softness, broad simple 2.5D shading, clean uninterrupted silhouette, not glossy plastic. A LOW WIDE chubby mochi lump, about 1.3 times as wide as tall. Single uninterrupted volume, NO separate head or neck, NO chest anatomy. Two tiny foot nubs continuous with bottom. Two small semi-floppy rounded ear flaps on upper corners, one slightly folded, and one tiny curved white-tipped tail on side. Simple face directly on blob surface: small cream-white oval eye shapes and flat cocoa oval pupils, tiny charcoal rounded triangular nose and short curved smile. No highlights or iris detail, no projecting snout, no cheek pads, no chin. This BORDER COLLIE has soft charcoal BLACK upper body and ears, narrow WHITE forehead blaze connecting to broad white lower-face and bottom color field. Color patches are FLAT coloration of single blob, never raised muzzle. Breed hint ONLY from black-white pattern and ear shape. Keep very simple, squishy and gently cheerful, original creature not realistic puppy. Pale warm cream square background, faint shadow. Character fills 85 percent. No text or accessories. Single character.
```
