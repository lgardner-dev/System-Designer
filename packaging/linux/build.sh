#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo generate-lockfile
cargo fmt --all
cargo test --locked --all-targets
cargo build --locked --release --bin system-designer
# One self-extracting file, matching system-designer-setup.exe on Windows.
bash packaging/linux/make-installer.sh
cp Cargo.lock dist/
