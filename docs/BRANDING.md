# System Designer iconography

## One selected master

All displayed product icons derive from the owner's selected file:

`assets/branding/svg/system-designer-atom-mark-color.svg`

SHA-256: `52c1a21a91c8769b0f67c9c97c02e0af77fa75f07b20064aa295feb44772a3ad`

This is the cyan-and-silver interwoven mark, not the other icon/tile in the
original package. The SVG is preserved byte for byte. It wraps one 1254x1254
RGBA PNG; it is not a newly traced vector. Extraction preserves the embedded
image exactly. Smaller outputs are resized directly from that image using
Lanczos, with its original transparent canvas and colors: no crop, added tile,
recoloring, or redrawing.

The initial platform exports in the supplied ZIP used a different design.
They have been replaced with derivatives of this selected master. The initial
42-file import remains available at commit
`6ddc31c3bda0365a07263309118df1f3c21f31ab`; the original ZIP remains in upload
commit `b03fd1e0b1315868f901348931124488ed065cde`.
`assets/branding/manifest.json` distinguishes original archive provenance,
current file checksums, the selected master, and the 33 derived display assets.
Unselected original artwork under `svg/`, `sources/`, and `branding/` is retained
only as historical reference; it is not an alternative input for app packaging.
The repository license is unchanged.

## Regeneration and verification

Ordinary Rust builds consume the checked-in assets. They need no Python imaging
library, renderer, network service, or runtime icon sidecar.
For artwork maintenance only:

```sh
python -m pip install Pillow==12.3.0
python packaging/generate-branding.py
python packaging/verify-branding.py
python packaging/generate-branding.py --check
```

The generator reads only the selected SVG, never the platform folders or an
independently selected PNG. It writes all Linux, Windows, macOS and web outputs,
including the full-size README/logo PNG. `--check` writes nothing: it regenerates
in memory and compares every PNG's RGBA pixels and every ICO/ICNS frame, plus
byte-exact SVG copies. PNG compression differences do not change that result.
The source checksum is pinned in the generator; changing the chosen master
requires an explicit owner decision rather than silently refreshing a manifest.

The dependency-free `verify-branding.py` checks the current inventory, hashes,
PNG dimensions, ICO/ICNS structure and selection binding. Its optional
`--pe target/release/system-designer.exe` check inspects the linked PE resource
hierarchy and every selected ICO frame. `--installer dist/system-designer-setup.exe`
checks for a selected installer icon. Normal CI runs integrity checks on all
platforms and the Pillow source-to-output check on Linux. Three Rust branding
tests cover decoding, the selected mark's 256px RGBA fingerprint, and texture reuse.

## Consumers

- Native app: `src/ui/branding.rs` embeds the generated 256px Linux PNG for the
  window, toolbar and Help. Decoding happens once, and textures are reused per
  egui context. The same artwork is used on all three operating systems.
- Windows: the generated multi-resolution ICO is embedded in the GUI executable
  and used by NSIS for installer/uninstaller icons. Shortcuts and Installed Apps
  reference the branded executable. The headless validator is unchanged.
- Linux: the generated PNG sizes and byte-exact master SVG are installed into
  the user's XDG hicolor theme. The desktop/application identity is unchanged.
  Install/uninstall checks preserve unrelated icons and user project data.
- macOS: the generated ICNS is copied into `Contents/Resources` and referenced by
  `CFBundleIconFile`; distribute the `.app`/DMG, not a bare executable.
- Repository/web: README uses the full-size PNG extracted from the selected SVG.
  Favicons, touch/PWA icons and logo exports use the same selected mark. No
  website deployment or GitHub account-avatar change is performed.

Project data, initialization prompts, scoped exchange, canvas behavior and
application identities are unchanged. The existing self-design's host/controls
responsibilities still own branding; no new application subsystem is added.

Resource and pixel checks do not establish desktop-shell appearance. Before
release, inspect freshly installed window, launcher, shortcuts/taskbar/menu/Dock
and Help icons on each target. Existing shell caches can retain an older icon.
Signing, notarization, merging and release publication remain separate actions.
