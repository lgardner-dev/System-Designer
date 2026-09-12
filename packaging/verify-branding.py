#!/usr/bin/env python3
"""Dependency-free checks of current branding assets and optional Windows PE resources."""
from pathlib import Path
import argparse
import hashlib
import json
import struct

ROOT = Path(__file__).resolve().parents[1]
BRAND = ROOT / "assets/branding"

def ico_images(data):
    reserved, kind, count = struct.unpack_from("<HHH", data)
    assert (reserved, kind) == (0, 1) and count > 0, "invalid ICO header"
    frames = []
    for i in range(count):
        w, h, _, _, _, _, length, offset = struct.unpack_from("<BBBBHHII", data, 6 + i * 16)
        assert 0 < length and offset + length <= len(data), "invalid ICO frame"
        frames.append((w or 256, h or 256, data[offset:offset + length]))
    return frames

def check_pe(path, require_all=True):
    data = path.read_bytes()
    assert data[:2] == b"MZ", "not a Windows PE"
    pe = struct.unpack_from("<I", data, 0x3c)[0]
    assert data[pe:pe + 4] == b"PE\0\0", "invalid PE signature"
    sections = struct.unpack_from("<H", data, pe + 6)[0]
    optional_size = struct.unpack_from("<H", data, pe + 20)[0]
    optional = pe + 24
    magic = struct.unpack_from("<H", data, optional)[0]
    assert magic in (0x10b, 0x20b), "unsupported PE format"
    directories = optional + (112 if magic == 0x20b else 96)
    resource_rva, _ = struct.unpack_from("<II", data, directories + 2 * 8)
    table = optional + optional_size
    def file_offset(rva):
        for i in range(sections):
            _, virtual, raw_size, raw_offset = struct.unpack_from("<IIII", data, table + i * 40 + 8)
            if virtual <= rva < virtual + raw_size:
                return raw_offset + rva - virtual
        raise AssertionError("resource RVA outside file-backed sections")
    base = file_offset(resource_rva)
    resources = {}
    def visit(relative, keys=()):
        assert len(keys) <= 2, "invalid resource nesting"
        offset = base + relative
        named, numbered = struct.unpack_from("<HH", data, offset + 12)
        for i in range(named + numbered):
            name, target = struct.unpack_from("<II", data, offset + 16 + i * 8)
            key = name if name & 0x80000000 == 0 else -1
            if target & 0x80000000:
                visit(target & 0x7fffffff, keys + (key,))
            else:
                rva, size = struct.unpack_from("<II", data, base + target)
                start = file_offset(rva)
                assert start + size <= len(data)
                resources[keys + (key,)] = data[start:start + size]
    visit(0)
    expected = ico_images((BRAND / "windows/system-designer.ico").read_bytes())
    matched = False
    for key, group in resources.items():
        if len(key) != 3 or key[0] != 14:
            continue
        reserved, kind, count = struct.unpack_from("<HHH", group)
        assert (reserved, kind) == (0, 1)
        actual = []
        for i in range(count):
            w, h, _, _, _, _, size, icon_id = struct.unpack_from("<BBBBHHIH", group, 6 + i * 14)
            image = resources.get((3, icon_id, key[2]))
            assert image is not None and len(image) == size, "broken group-icon reference"
            actual.append((w or 256, h or 256, image))
        if (all(frame in actual for frame in expected) if require_all
                else any(frame in actual for frame in expected)):
            matched = True
    assert matched, f"approved icon frames not found in {path}"
    print(f"Verified approved icon resources: {path}")

def check_sources():
    manifest = json.loads((BRAND / "manifest.json").read_text(encoding="utf-8"))
    files = manifest["files"]
    selected = manifest["selected_master"]
    assert selected["path"] == "svg/system-designer-atom-mark-color.svg"
    assert files[selected["path"]] == selected["sha256"], "selected-source binding changed"
    assert set(manifest["derived_files"]) <= set(files), "missing derived assets"
    actual = {str(p.relative_to(BRAND)).replace("\\", "/") for p in BRAND.rglob("*")
              if p.is_file() and p.name != "manifest.json"}
    assert actual == set(files), "brand file inventory changed"
    for relative, digest in files.items():
        assert hashlib.sha256((BRAND / relative).read_bytes()).hexdigest() == digest, relative
    for size in (16, 24, 32, 48, 64, 128, 256, 512):
        data = (BRAND / f"linux/png/system-designer-{size}.png").read_bytes()
        assert data[:8] == b"\x89PNG\r\n\x1a\n"
        assert struct.unpack_from(">II", data, 16) == (size, size)
    frames = ico_images((BRAND / "windows/system-designer.ico").read_bytes())
    assert {16, 32, 48, 256} <= {w for w, h, _ in frames if w == h}
    data = (BRAND / "macos/system-designer.icns").read_bytes()
    assert data[:4] == b"icns" and struct.unpack_from(">I", data, 4)[0] == len(data)
    offset = 8
    while offset < len(data):
        size = struct.unpack_from(">I", data, offset + 4)[0]
        assert size >= 8 and offset + size <= len(data)
        offset += size
    assert offset == len(data)
    print(f"Verified {len(files)} branding files, selected-source binding, PNG sizes, ICO frames and ICNS structure")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pe", type=Path, action="append", default=[])
    parser.add_argument("--installer", type=Path, action="append", default=[])
    args = parser.parse_args()
    check_sources()
    for path in args.pe:
        check_pe(path)
    for path in args.installer:
        check_pe(path, require_all=False)
