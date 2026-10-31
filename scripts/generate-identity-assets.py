# /// script
# requires-python = ">=3.12"
# dependencies = ["Pillow==13.0.0"]
# ///
"""Derive browser/home-screen icons from the approved transparent mascot masters."""

import hashlib
import json
from pathlib import Path

from PIL import Image, ImageColor

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "typescript/packages/ui/assets"
TOKENS = json.loads(
    (ROOT / "DESIGN.md").read_text().split("```yaml\n")[1].split("\n```")[0]
)
BACKGROUND = (*ImageColor.getrgb(TOKENS["colors"]["canvas"]), 255)


def compose(
    master: Image.Image, size: int, fraction: float, opaque: bool
) -> Image.Image:
    """Contain the entire alpha silhouette, retaining a platform-safe margin."""
    bounds = master.getchannel("A").getbbox()
    if bounds is None:
        raise ValueError("Mascot master has no visible artwork")
    artwork = master.crop(bounds)
    edge = round(size * fraction)
    artwork.thumbnail((edge, edge), Image.Resampling.LANCZOS)
    canvas = Image.new("RGBA", (size, size), BACKGROUND if opaque else (0, 0, 0, 0))
    canvas.alpha_composite(
        artwork, ((size - artwork.width) // 2, (size - artwork.height) // 2)
    )
    return canvas.convert("RGB") if opaque else canvas


def main() -> None:
    """Generate the same complete icon family for every available character."""
    records = []
    for source in sorted((ASSETS / "mascots/transparent").glob("*.png")):
        master = Image.open(source).convert("RGBA")
        output = ASSETS / "icons" / source.stem
        output.mkdir(parents=True, exist_ok=True)
        for size in (16, 32, 48):
            compose(master, size, 0.92, False).save(output / f"favicon-{size}.png")
        compose(master, 48, 0.92, False).save(
            output / "favicon.ico", sizes=[(16, 16), (32, 32), (48, 48)]
        )
        compose(master, 180, 0.84, True).save(output / "apple-touch-icon.png")
        for size in (192, 512, 1024):
            compose(master, size, 0.84, True).save(output / f"icon-{size}.png")
        # A 0.56-wide square fits entirely inside the central 0.8-diameter safe circle.
        compose(master, 512, 0.56, True).save(output / "icon-maskable-512.png")
        records.append(
            {
                "character": source.stem,
                "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                "outputs": {
                    path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in sorted(output.iterdir())
                    if path.is_file()
                },
            }
        )
    (ASSETS / "icons/manifest.json").write_text(json.dumps(records, indent=2) + "\n")


if __name__ == "__main__":
    main()
