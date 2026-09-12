#!/usr/bin/env bash
# Produces dist/system-designer-setup.sh: one self-extracting file that installs
# System Designer for the current user, mirroring system-designer-setup.exe.
set -euo pipefail
cd "$(dirname "$0")/../.."

version="$(sed -n 's/^version *= *"\([^"]*\)".*/\1/p' Cargo.toml | head -n 1)"
[ -n "$version" ] || { echo "Could not read version from Cargo.toml" >&2; exit 1; }

binary="${APP_BINARY:-target/release/system-designer}"
[ -f "$binary" ] || { echo "Missing $binary -- run cargo build --release first." >&2; exit 1; }

out="dist/system-designer-setup.sh"
mkdir -p dist

stage="$(mktemp -d)"
trap 'rm -rf "$stage"' EXIT
install -m 755 "$binary" "$stage/system-designer"
install -m 644 packaging/linux/system-designer.desktop "$stage/system-designer.desktop"
install -m 644 LICENSE "$stage/LICENSE"
mkdir -p "$stage/icons"
cp -R assets/branding/linux/png assets/branding/linux/scalable "$stage/icons/"
tar -C "$stage" -czf "$stage/payload.tar.gz" system-designer system-designer.desktop LICENSE icons

sed "s/@VERSION@/${version}/g" packaging/linux/installer-header.sh > "$out"
# The payload begins on the line after the header. Substituting the count in
# place keeps the header the same length, so the number stays correct.
header_lines="$(wc -l < "$out")"
sed -i "s/@PAYLOAD_LINE@/$((header_lines + 1))/" "$out"
cat "$stage/payload.tar.gz" >> "$out"
chmod 755 "$out"

echo "Built $out ($(du -h "$out" | cut -f1))"
command -v sha256sum >/dev/null 2>&1 && sha256sum "$out"
