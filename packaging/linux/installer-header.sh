#!/usr/bin/env bash
# System Designer self-extracting installer.
#
# This file is script text followed by a gzipped tar payload. make-installer.sh
# substitutes PAYLOAD_LINE with the first line of that payload, so the script
# never has to search its own body for a marker.
set -euo pipefail

VERSION="@VERSION@"
PAYLOAD_LINE=@PAYLOAD_LINE@

prefix="${HOME}/.local"
data_home="${XDG_DATA_HOME:-${HOME}/.local/share}"
desktop_icon=""
do_uninstall=0

usage() {
    cat <<USAGE
System Designer ${VERSION} installer

Usage: $0 [options]

  --desktop-icon        Also place a shortcut on the desktop
  --no-desktop-icon     Skip the desktop shortcut (no prompt)
  --prefix DIR          Install root (default: ${HOME}/.local)
  --uninstall           Remove a previous installation
  -h, --help            Show this message

Installs for the current user only. No root access is used or required.
USAGE
}

while [ $# -gt 0 ]; do
    case "$1" in
        --desktop-icon) desktop_icon=1 ;;
        --no-desktop-icon) desktop_icon=0 ;;
        --prefix) [ $# -ge 2 ] || { echo "--prefix needs a directory" >&2; exit 2; }; prefix="$2"; shift ;;
        --uninstall) do_uninstall=1 ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
    shift
done

bin_dir="${prefix}/bin"
app_dir="${data_home}/applications"
desktop_dir="$( { command -v xdg-user-dir >/dev/null 2>&1 && xdg-user-dir DESKTOP; } || echo "${HOME}/Desktop" )"
[ -n "$desktop_dir" ] || desktop_dir="${HOME}/Desktop"

icon_root="${data_home}/icons/hicolor"
refresh_icons() {
    command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t "$icon_root" 2>/dev/null || true
}

if [ "$do_uninstall" -eq 1 ]; then
    for size in 16 24 32 48 64 128 256 512; do
        rm -f "$icon_root/${size}x${size}/apps/system-designer.png"
    done
    rm -f "$icon_root/scalable/apps/system-designer.svg"
    refresh_icons
    rm -f "${bin_dir}/system-designer"
    rm -f "${app_dir}/system-designer.desktop"
    rm -f "${desktop_dir}/system-designer.desktop"
    command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "${app_dir}" 2>/dev/null || true
    echo "Removed System Designer. Project files and backups were left untouched."
    exit 0
fi

# Ask only when nothing was specified and someone is actually at the terminal.
if [ -z "$desktop_icon" ]; then
    if [ -t 0 ]; then
        printf 'Place a shortcut on the desktop? [y/N] '
        read -r reply || reply=""
        case "$reply" in [yY]*) desktop_icon=1 ;; *) desktop_icon=0 ;; esac
    else
        desktop_icon=0
    fi
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
tail -n +"${PAYLOAD_LINE}" "$0" | tar xz -C "$tmp"

for size in 16 24 32 48 64 128 256 512; do
    install -Dm644 "$tmp/icons/png/system-designer-${size}.png" "$icon_root/${size}x${size}/apps/system-designer.png"
done
install -Dm644 "$tmp/icons/scalable/system-designer.svg" "$icon_root/scalable/apps/system-designer.svg"
refresh_icons
mkdir -p "$bin_dir" "$app_dir"
install -m 755 "${tmp}/system-designer" "${bin_dir}/system-designer"

# The launcher records an absolute quoted path rather than trusting PATH, which
# may not include ~/.local/bin in a desktop session.
escaped="$(printf '%s' "${bin_dir}/system-designer" | sed 's/\\/\\\\/g;s/"/\\"/g')"
sed "s|^Exec=.*|Exec=\"${escaped}\" %f|" "${tmp}/system-designer.desktop" > "${app_dir}/system-designer.desktop"
chmod 644 "${app_dir}/system-designer.desktop"

if [ "$desktop_icon" -eq 1 ]; then
    mkdir -p "$desktop_dir"
    cp "${app_dir}/system-designer.desktop" "${desktop_dir}/system-designer.desktop"
    chmod 755 "${desktop_dir}/system-designer.desktop"
    # GNOME refuses to launch desktop files it does not consider trusted.
    command -v gio >/dev/null 2>&1 && gio set "${desktop_dir}/system-designer.desktop" metadata::trusted true 2>/dev/null || true
fi

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$app_dir" 2>/dev/null || true

echo "Installed System Designer ${VERSION} for the current user."
echo "  Executable:  ${bin_dir}/system-designer"
echo "  Menu entry:  ${app_dir}/system-designer.desktop"
[ "$desktop_icon" -eq 1 ] && echo "  Desktop:     ${desktop_dir}/system-designer.desktop"
case ":${PATH}:" in
    *":${bin_dir}:"*) ;;
    *) echo "Note: ${bin_dir} is not on your PATH; run it from the menu or by full path." ;;
esac
echo "Uninstall with: $0 --uninstall"

exit 0
