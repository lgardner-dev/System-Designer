# System Designer iconography

The owner-selected artwork is stored in `assets/branding/`. All 42 source files
from `system-designer-icon-package.zip` are preserved byte for byte (the manifest
is the authoritative inventory if the package's count is revised). Original
artwork, color and monochrome marks, PNGs, ICO, ICNS, iconset and website assets
are included. The root upload ZIP was unpacked, not redrawn; its original bytes
remain in commit `b03fd1e0b1315868f901348931124488ed065cde`.
`manifest.json` binds the archive checksum and each extracted file's checksum.
The supplied SVGs wrap raster artwork; they are not a newly traced vector logo.
The existing repository license is unchanged.

## Native app

`src/ui/branding.rs` decodes the compiled-in 256px PNG once and caches one texture
per egui context. The icon appears beside the application heading and in Help.
`src/main.rs` supplies the same image as the native window icon and sets the
Linux application ID to match `system-designer.desktop`. The executable needs
no sidecar icon files and performs no network or project-model operations for
branding. Existing projects, scope packets, prompts and design semantics are
unchanged. The self-design records branding under the existing host/controls
responsibilities instead of adding a new subsystem.

## Platform packaging

- Windows: `build.rs` uses the optional, build-only `embed-resource` helper to
  link the approved multi-resolution ICO into `system-designer.exe`. It does
  not modify the headless validator. NSIS uses the same ICO for its installer
  and uninstaller; application shortcuts and Installed Apps use the executable
  icon. Missing resource tooling fails a Windows desktop build explicitly.
- Linux: installers copy the exact supplied PNG sizes and SVG into the user's
  XDG hicolor icon theme, set `Icon=system-designer`, and refresh icon caches
  when that utility exists. Uninstall removes only this application's icon
  files, preserving unrelated theme files and user designs. Custom binary
  prefixes do not override `XDG_DATA_HOME`.
- macOS: the app bundle includes the original ICNS in `Contents/Resources`,
  referenced by `CFBundleIconFile`. A bare Unix executable has no Finder bundle
  icon; distribute the `.app`/DMG.
- Repository: README displays the approved tile. Web and monochrome variants
  are checked in for later use; no website or GitHub account avatar is changed.

## Verification

`python packaging/verify-branding.py` verifies the entire preserved inventory,
PNG dimensions, ICO and ICNS structure without third-party Python modules.
On Windows, `--pe target/release/system-designer.exe` checks the linked resource
hierarchy and every original ICO frame; `--installer dist/system-designer-setup.exe`
checks for an approved installer icon. These checks are included in normal CI,
as are Linux icon install/uninstall checks and a macOS bundle-resource check.
The two Rust branding tests cover PNG decoding and texture reuse.

Automated resource checks are not a desktop-shell visual review. Before release,
inspect a freshly installed app's window, shortcuts/taskbar/menu/Dock and Help
on each target. Shells may cache icons from older builds. Signing, notarization
and release publication remain separate owner actions.
