#!/usr/bin/env bash
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
bin="$HOME/.local/bin"
apps="$HOME/.local/share/applications"
mkdir -p "$bin" "$apps"
install -m 755 "$here/system-designer" "$bin/system-designer"
# The desktop launcher uses an absolute quoted executable path, not PATH assumptions.
escaped="$(printf '%s' "$bin/system-designer" | sed 's/\\/\\\\/g;s/"/\\"/g')"
sed "s|^Exec=.*|Exec=\"$escaped\" %f|" "$here/system-designer.desktop" > "$apps/system-designer.desktop"
printf 'Installed for this user. Run: "%s/system-designer"\n' "$bin"
