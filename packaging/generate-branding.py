#!/usr/bin/env python3
"""Generate/check display assets from the owner's exact selected SVG.

Maintenance/CI only: python -m pip install Pillow==12.3.0
Ordinary application builds use the checked-in outputs and need no Pillow.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import io
import json
from pathlib import Path
import xml.etree.ElementTree as ET

import PIL
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
BRAND = ROOT / "assets/branding"
MASTER = "svg/system-designer-atom-mark-color.svg"
# Changing the chosen artwork is an explicit owner decision, not a checksum refresh.
MASTER_SHA256 = "52c1a21a91c8769b0f67c9c97c02e0af77fa75f07b20064aa295feb44772a3ad"
PILLOW_VERSION = "12.3.0"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def outputs() -> dict[str, bytes]:
    require(PIL.__version__ == PILLOW_VERSION,
            f"Use Pillow=={PILLOW_VERSION} for the recorded resampling recipe")
    svg = (BRAND / MASTER).read_bytes()
    require(digest(svg) == MASTER_SHA256, "Selected master changed; obtain owner approval")
    # This exact SVG contains one embedded PNG, with no transforms or extra drawing.
    # Decode it losslessly rather than screenshotting, tracing, or inventing artwork.
    element = ET.fromstring(svg)
    require(len(element) == 1 and element[0].tag == "{http://www.w3.org/2000/svg}image",
            "Unsupported master SVG structure")
    href = element[0].get("href", "")
    require(href.startswith("data:image/png;base64,"), "Master must embed its PNG")
    png = base64.b64decode(href.partition(",")[2], validate=True)
    with Image.open(io.BytesIO(png)) as opened:
        master = opened.convert("RGBA")
    require(master.size == (1254, 1254), "Unexpected selected-master dimensions")
    master.info.clear()
    sizes = (16, 24, 32, 48, 64, 128, 180, 192, 256, 512, 1024)
    images = {s: master.resize((s, s), Image.Resampling.LANCZOS) for s in sizes}
    pngs = {}
    for size, image in images.items():
        stream = io.BytesIO()
        image.save(stream, format="PNG")
        pngs[size] = stream.getvalue()

    result = {f"linux/png/system-designer-{s}.png": pngs[s]
              for s in (16, 24, 32, 48, 64, 128, 256, 512)}
    result.update({f"windows/system-designer-{s}.png": pngs[s]
                   for s in (16, 32, 48, 256)})
    for size in (16, 32, 128, 256, 512):
        for scale in (1, 2):
            suffix = "@2x" if scale == 2 else ""
            result[f"macos/system-designer.iconset/icon_{size}x{size}{suffix}.png"] = pngs[size * scale]
    for name in ("linux/scalable/system-designer.svg", "web/favicon.svg", "web/logo-mark.svg"):
        result[name] = svg
    for name in ("branding/system-designer-atom-mark-color.png", "web/logo-mark.png"):
        result[name] = png
    result.update({"web/apple-touch-icon.png": pngs[180], "web/icon-192.png": pngs[192],
                   "web/icon-512.png": pngs[512]})
    for name, icon_sizes in (("windows/system-designer.ico", (16, 24, 32, 48, 64, 128, 256)),
                             ("web/favicon.ico", (16, 32, 48))):
        stream = io.BytesIO()
        images[max(icon_sizes)].save(stream, format="ICO", sizes=[(s, s) for s in icon_sizes],
                                    append_images=[images[s] for s in icon_sizes])
        result[name] = stream.getvalue()
    stream = io.BytesIO()
    images[1024].save(stream, format="ICNS",
                      append_images=[images[s] for s in (32, 64, 128, 256, 512, 1024)])
    result["macos/system-designer.icns"] = stream.getvalue()
    return result


def pixels(image: Image.Image) -> tuple[tuple[int, int], bytes]:
    return image.size, image.convert("RGBA").tobytes()


def signature(name: str, data: bytes) -> object:
    """Compare every decoded frame, independent of PNG/zlib encoding differences."""
    if name.endswith(".svg"):
        return data  # SVG copies must preserve the source byte for byte.
    with Image.open(io.BytesIO(data)) as image:
        if name.endswith(".ico"):
            return {size: pixels(image.ico.getimage(size)) for size in image.ico.sizes()}
        if name.endswith(".icns"):
            frames = {}
            for width, height, scale in image.info["sizes"]:
                with Image.open(io.BytesIO(data)) as frame:
                    frame.size = (width, height)
                    frame.load(scale=scale)
                    frames[(width, height, scale)] = pixels(frame)
            return frames
        return pixels(image)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="Check without changing any files")
    args = parser.parse_args()
    expected = outputs()
    manifest_path = BRAND / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    selection = {"path": MASTER, "sha256": MASTER_SHA256,
                 "recipe": f"embedded PNG; direct RGBA LANCZOS resizing; Pillow {PILLOW_VERSION}"}
    if args.check:
        require(manifest.get("selected_master") == selection, "Selection metadata is stale")
        require(manifest.get("derived_files") == sorted(expected), "Derived-file inventory is stale")
        for name, data in expected.items():
            actual = (BRAND / name).read_bytes()
            require(signature(name, actual) == signature(name, data),
                    f"Not derived from the selected color mark: {name}")
        print(f"Verified all frames/pixels in {len(expected)} outputs against {MASTER}")
        return
    # Preserve the exact source archive's provenance; current files are not all originals.
    manifest.setdefault("original_import_commit", "6ddc31c3bda0365a07263309118df1f3c21f31ab")
    manifest["selected_master"] = selection
    manifest["derived_files"] = sorted(expected)
    actual_files = {p.relative_to(BRAND).as_posix() for p in BRAND.rglob("*")
                    if p.is_file() and p.name != "manifest.json"}
    require(actual_files == set(manifest["files"]), "Unexpected branding files; inspect inventory first")
    require(set(expected) <= actual_files, "Expected display-asset path is missing")
    for name, data in expected.items():
        (BRAND / name).write_bytes(data)
    manifest["files"] = {name: digest((BRAND / name).read_bytes()) for name in sorted(actual_files)}
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8", newline="\n")
    print(f"Generated {len(expected)} display assets from {MASTER}; source unchanged")


if __name__ == "__main__":
    main()
