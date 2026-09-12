#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
bin="$HOME/.local/bin"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
apps="$data_home/applications"
icon_root="$data_home/icons/hicolor"
for size in 16 24 32 48 64 128 256 512; do
    install -Dm644 "$here/icons/png/system-designer-${size}.png" "$icon_root/${size}x${size}/apps/system-designer.png"
done
install -Dm644 "$here/icons/scalable/system-designer.svg" "$icon_root/scalable/apps/system-designer.svg"
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t "$icon_root" 2>/dev/null || true
mkdir -p "$bin" "$apps"
install -m 755 "$here/system-designer" "$bin/system-designer"
# The desktop launcher uses an absolute quoted executable path, not PATH assumptions.
escaped="$(printf '%s' "$bin/system-designer" | sed 's/\\/\\\\/g;s/"/\\"/g')"
sed "s|^Exec=.*|Exec=\"$escaped\" %f|" "$here/system-designer.desktop" > "$apps/system-designer.desktop"
printf 'Installed for this user. Run: "%s/system-designer"\n' "$bin"
