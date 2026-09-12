"""One-time import of the owner-supplied, checksum-bound icon package."""
from pathlib import Path, PurePosixPath
import hashlib
import json
import stat
import zipfile

ROOT = Path.cwd()
archive = ROOT / 'system-designer-icon-package.zip'
expected = '38cf6e183ca320a70a1ebdee308efe66d5e8f4ed57c48d985c52c337a7f7cbf5'
assert hashlib.sha256(archive.read_bytes()).hexdigest() == expected, 'Uploaded package differs from reviewed artwork'
brand = ROOT / 'assets/branding'
assert not brand.exists(), 'Refusing to replace an existing brand directory'
manifest = {}
with zipfile.ZipFile(archive) as package:
    for entry in package.infolist():
        name = PurePosixPath(entry.filename)
        assert name.parts[0] == 'system-designer-icon-package'
        assert not name.is_absolute() and '..' not in name.parts
        assert not stat.S_ISLNK(entry.external_attr >> 16)
        if entry.is_dir():
            continue
        relative = PurePosixPath(*name.parts[1:])
        assert relative.parts and str(relative) not in manifest
        data = package.read(entry)
        destination = brand / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(data)
        manifest[str(relative)] = hashlib.sha256(data).hexdigest()
(brand / 'manifest.json').write_text(json.dumps({
    'source_commit': 'b03fd1e0b1315868f901348931124488ed065cde',
    'source_archive': archive.name,
    'source_sha256': expected,
    'files': dict(sorted(manifest.items())),
}, indent=2) + '\n', encoding='utf-8')

def write(path, text):
    p = ROOT / path
    assert not p.exists(), f'Refusing to overwrite {path}'
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding='utf-8')

def replace(path, old, new):
    p = ROOT / path
    text = p.read_text(encoding='utf-8')
    assert text.count(old) == 1, f'Expected one matching edit in {path}: {old!r}'
    p.write_text(text.replace(old, new), encoding='utf-8')

write('src/ui/branding.rs', '''//! Embedded product artwork, independent of any user's project.
use eframe::egui::{self, ColorImage, IconData, TextureHandle, TextureOptions};
use std::sync::OnceLock;

const PNG: &[u8] = include_bytes!("../../assets/branding/linux/png/system-designer-256.png");
static ICON: OnceLock<IconData> = OnceLock::new();

/// The supplied PNG is compiled in; no file, service or image loader is required.
pub fn window_icon() -> IconData {
    ICON.get_or_init(|| {
        eframe::icon_data::from_png_bytes(PNG)
            .expect("the checked-in System Designer icon must be a valid PNG")
    }).clone()
}

fn texture(ctx: &egui::Context) -> TextureHandle {
    let id = egui::Id::new("system-designer.brand-icon");
    if let Some(texture) = ctx.data(|data| data.get_temp::<TextureHandle>(id)) {
        return texture;
    }
    let icon = window_icon();
    let image = ColorImage::from_rgba_unmultiplied(
        [icon.width as usize, icon.height as usize], &icon.rgba,
    );
    let texture = ctx.load_texture("System Designer atom", image, TextureOptions::LINEAR);
    ctx.data_mut(|data| data.insert_temp(id, texture.clone()));
    texture
}

pub(super) fn show(ui: &mut egui::Ui, size: f32) {
    let texture = texture(ui.ctx());
    ui.add(egui::Image::new((texture.id(), egui::vec2(size, size)))
        .alt_text("System Designer atom icon"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_icon_decodes_to_complete_rgba() {
        let icon = window_icon();
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
        assert!(icon.rgba.chunks_exact(4).any(|p| p[3] > 0));
        assert_eq!(window_icon().rgba, icon.rgba);
    }

    #[test]
    fn icon_texture_is_cached_per_context() {
        let ctx = egui::Context::default();
        assert_eq!(texture(&ctx).id(), texture(&ctx).id());
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                show(ui, 24.0);
                show(ui, 64.0);
            });
        });
    }
}
''')
replace('src/ui/mod.rs', 'mod canvas;', 'mod branding;\npub use branding::window_icon;\nmod canvas;')
replace('src/main.rs', '.with_title("System Designer")', '.with_title("System Designer")\n            .with_app_id("system-designer")\n            .with_icon(system_designer::ui::window_icon())')
replace('src/ui/workspace.rs', '                    ui.heading("System Designer");', '                    super::branding::show(ui, 24.0);\n                    ui.heading("System Designer");')
replace('src/ui/inspector.rs', '                        ui.heading("System Designer — native Rust");', '''                        ui.horizontal(|ui| {
                            super::branding::show(ui, 64.0);
                            ui.vertical(|ui| {
                                ui.heading("System Designer");
                                ui.label(format!("Native Rust · {}", env!("CARGO_PKG_VERSION")));
                            });
                        });''')
replace('Cargo.toml', 'desktop = ["dep:eframe", "dep:rfd"]', 'desktop = ["dep:eframe", "dep:rfd", "dep:embed-resource"]')
replace('Cargo.toml', '[profile.release]', '[build-dependencies]\n# Build-time only: link the Windows icon into the GUI executable, not the validator.\nembed-resource = { version = "=3.0.11", optional = true }\n\n[profile.release]')
write('build.rs', '''fn main() {
    println!("cargo:rerun-if-changed=packaging/windows/app.rc");
    println!("cargo:rerun-if-changed=assets/branding/windows/system-designer.ico");
    #[cfg(feature = "desktop")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let root = std::env::var("CARGO_MANIFEST_DIR").expect("Cargo supplies the manifest directory");
        let icon = std::path::Path::new(&root).join("assets/branding/windows/system-designer.ico");
        let define = format!("APP_ICON=\\\"{}\\\"", icon.to_string_lossy().replace('\\\\', "/"));
        embed_resource::compile_for("packaging/windows/app.rc", ["system-designer"], [define])
            .manifest_required()
            .expect("could not embed the System Designer Windows icon");
    }
}
''')
write('packaging/windows/app.rc', '// APP_ICON is an absolute, quoted path supplied by build.rs.\n1 ICON APP_ICON\n')
replace('packaging/windows/setup.nsi', '!define ARP ', '''!ifndef APP_ICON
  !define APP_ICON "${__FILEDIR__}\\..\\..\\assets\\branding\\windows\\system-designer.ico"
!endif
!define MUI_ICON "${APP_ICON}"
!define MUI_UNICON "${APP_ICON}"

!define ARP ''')
replace('packaging/windows/setup.nsi', '\'"$INSTDIR\\SystemDesigner.exe"\'\n', '\'"$INSTDIR\\SystemDesigner.exe",0\'\n')
replace('packaging/linux/system-designer.desktop', 'Terminal=false', 'Icon=system-designer\nStartupWMClass=system-designer\nTerminal=false')
replace('packaging/linux/make-installer.sh', 'tar -C "$stage" -czf "$stage/payload.tar.gz" system-designer system-designer.desktop LICENSE', '''mkdir -p "$stage/icons"
cp -R assets/branding/linux/png assets/branding/linux/scalable "$stage/icons/"
tar -C "$stage" -czf "$stage/payload.tar.gz" system-designer system-designer.desktop LICENSE icons''')
replace('packaging/linux/installer-header.sh', 'if [ "$do_uninstall" -eq 1 ]; then', '''icon_root="${data_home}/icons/hicolor"
refresh_icons() {
    command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t "$icon_root" 2>/dev/null || true
}

if [ "$do_uninstall" -eq 1 ]; then''')
replace('packaging/linux/installer-header.sh', '    rm -f "${bin_dir}/system-designer"', '''    for size in 16 24 32 48 64 128 256 512; do
        rm -f "$icon_root/${size}x${size}/apps/system-designer.png"
    done
    rm -f "$icon_root/scalable/apps/system-designer.svg"
    refresh_icons
    rm -f "${bin_dir}/system-designer"''')
replace('packaging/linux/installer-header.sh', 'mkdir -p "$bin_dir" "$app_dir"', '''for size in 16 24 32 48 64 128 256 512; do
    install -Dm644 "$tmp/icons/png/system-designer-${size}.png" "$icon_root/${size}x${size}/apps/system-designer.png"
done
install -Dm644 "$tmp/icons/scalable/system-designer.svg" "$icon_root/scalable/apps/system-designer.svg"
refresh_icons
mkdir -p "$bin_dir" "$app_dir"''')
replace('packaging/linux/install.sh', 'apps="$HOME/.local/share/applications"', '''data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
apps="$data_home/applications"
icon_root="$data_home/icons/hicolor"
for size in 16 24 32 48 64 128 256 512; do
    install -Dm644 "$here/icons/png/system-designer-${size}.png" "$icon_root/${size}x${size}/apps/system-designer.png"
done
install -Dm644 "$here/icons/scalable/system-designer.svg" "$icon_root/scalable/apps/system-designer.svg"
command -v gtk-update-icon-cache >/dev/null 2>&1 && gtk-update-icon-cache -f -t "$icon_root" 2>/dev/null || true''')
replace('README.md', '# System Designer\n', '''<p align="center">
  <img src="assets/branding/branding/system-designer-atom-icon.png" width="128" alt="System Designer atom icon">
</p>

# System Designer
''')
replace('README.md', '## What it is for', '''The approved [icon family](assets/branding/README.txt) is checked in for the
native app, installers, documentation and future website. See
[branding integration](docs/BRANDING.md) for source fidelity and platform details.

## What it is for''')
replace('CONTRIBUTING.md', 'assets/        initialization prompt, embedded with include_str!', 'assets/        embedded initialization prompt and approved branding/icon family')
replace('CONTRIBUTING.md', 'Nothing else is required: no Node, no bundler, no service to run alongside.', '''Desktop Windows builds additionally use the Windows SDK resource compiler
(installed with the C++ tools) through the build-only `embed-resource` crate.
Missing resource tooling fails the desktop build instead of silently shipping
an unbranded executable. Headless builds do not invoke the resource compiler.
No Node, bundler, image conversion tool, or runtime service is required.''')

write('packaging/verify-branding.py', '''#!/usr/bin/env python3
"""Dependency-free checks of original icon bytes and optional Windows PE resources."""
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
    assert data[pe:pe + 4] == b"PE\\0\\0", "invalid PE signature"
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
    actual = {str(p.relative_to(BRAND)).replace("\\\\", "/") for p in BRAND.rglob("*")
              if p.is_file() and p.name != "manifest.json"}
    assert actual == set(files), "brand file inventory changed"
    for relative, digest in files.items():
        assert hashlib.sha256((BRAND / relative).read_bytes()).hexdigest() == digest, relative
    for size in (16, 24, 32, 48, 64, 128, 256, 512):
        data = (BRAND / f"linux/png/system-designer-{size}.png").read_bytes()
        assert data[:8] == b"\\x89PNG\\r\\n\\x1a\\n"
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
    print(f"Verified {len(files)} original files, PNG sizes, ICO frames and ICNS structure")

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
''')
write('docs/BRANDING.md', '''# System Designer iconography

The owner-selected artwork is stored in `assets/branding/`. All 39 source files
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
''')
p = ROOT / 'docs/BRANDING.md'
p.write_text(p.read_text().replace('All 39 source files', f'All {len(manifest)} source files'), encoding='utf-8')

replace('.github/workflows/ci.yml', '      - name: Check formatting', '''      - name: Verify original branding assets
        run: python packaging/verify-branding.py
      - name: Check formatting''')
replace('.github/workflows/ci.yml', '          Copy-Item Cargo.lock dist/', '''          python packaging/verify-branding.py --pe target/release/system-designer.exe --installer dist/system-designer-setup.exe
          if ($LASTEXITCODE -ne 0) { throw 'Icon resource verification failed' }
          Copy-Item Cargo.lock dist/''')
replace('.github/workflows/ci.yml', '''          bash dist/system-designer-setup.sh --prefix "$RUNNER_TEMP/sd" --no-desktop-icon
          test -x "$RUNNER_TEMP/sd/bin/system-designer"
          bash dist/system-designer-setup.sh --prefix "$RUNNER_TEMP/sd" --uninstall
          test ! -e "$RUNNER_TEMP/sd/bin/system-designer"''', '''          export XDG_DATA_HOME="$RUNNER_TEMP/sd-data"
          icon_root="$XDG_DATA_HOME/icons/hicolor"
          mkdir -p "$icon_root/32x32/apps"
          printf 'unrelated' > "$icon_root/32x32/apps/other-app.png"
          bash dist/system-designer-setup.sh --prefix "$RUNNER_TEMP/sd" --no-desktop-icon
          test -x "$RUNNER_TEMP/sd/bin/system-designer"
          for size in 16 24 32 48 64 128 256 512; do
            cmp "assets/branding/linux/png/system-designer-$size.png" "$icon_root/${size}x${size}/apps/system-designer.png"
          done
          cmp assets/branding/linux/scalable/system-designer.svg "$icon_root/scalable/apps/system-designer.svg"
          grep -qx 'Icon=system-designer' "$XDG_DATA_HOME/applications/system-designer.desktop"
          bash dist/system-designer-setup.sh --prefix "$RUNNER_TEMP/sd" --uninstall
          test ! -e "$RUNNER_TEMP/sd/bin/system-designer"
          test -z "$(find "$icon_root" -type f -name 'system-designer.*' -print)"
          test -f "$icon_root/32x32/apps/other-app.png"''')
replace('.github/workflows/ci.yml', "          mkdir -p 'dist/System Designer.app/Contents/MacOS'", '''          mkdir -p 'dist/System Designer.app/Contents/MacOS' 'dist/System Designer.app/Contents/Resources'
          cp assets/branding/macos/system-designer.icns 'dist/System Designer.app/Contents/Resources/system-designer.icns' ''')
replace('.github/workflows/ci.yml', '          <key>CFBundlePackageType</key>', '          <key>CFBundleIconFile</key><string>system-designer.icns</string>\n          <key>CFBundlePackageType</key>')
replace('.github/workflows/ci.yml', "          hdiutil create -volname 'System Designer'", '''          cmp assets/branding/macos/system-designer.icns 'dist/System Designer.app/Contents/Resources/system-designer.icns'
          /usr/libexec/PlistBuddy -c 'Print :CFBundleIconFile' 'dist/System Designer.app/Contents/Info.plist' | grep -qx 'system-designer.icns'
          hdiutil create -volname 'System Designer' ''')

path = ROOT / 'design/system-designer.project.json'
design = json.loads(path.read_text(encoding='utf-8'))
notes = {
    'workspace.host': '\n\nBranding: src/main.rs and src/ui/branding.rs embed the approved icon and match the Linux desktop ID. build.rs links the Windows executable icon; packaging supplies native launcher/bundle resources. No project-data effect.',
    'workspace.controls': '\n\nBranding: src/ui/branding.rs displays the embedded atom icon in the toolbar and Help; cached UI resources do not alter the design model or scoped exchange.',
}
found = set()
for system in design['systems']:
    for node in system['nodes']:
        if node['id'] in notes:
            node['purpose'] += notes[node['id']]
            found.add(node['id'])
assert found == set(notes)
path.write_text(json.dumps(design, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
archive.unlink()
print(f'Imported {len(manifest)} original brand files and applied native/platform integration')
