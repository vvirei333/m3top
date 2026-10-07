#!/usr/bin/env bash
# Builds M3Top release packages: Arch (.pkg.tar.zst), Debian (.deb),
# RPM (.rpm) and a portable AppImage. Run from anywhere; output lands in
# dist/ at the repo root.
#
# One-time tool setup (not run automatically, since it touches the
# network): `cargo install cargo-deb cargo-generate-rpm`, plus
# `linuxdeploy-x86_64.AppImage` and `appimagetool-x86_64.AppImage`
# (https://github.com/linuxdeploy/linuxdeploy and
# https://github.com/AppImage/appimagetool releases) on PATH or next to
# this script.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST="$ROOT/dist"
VERSION="$(grep -m1 '^version' "$ROOT/crates/desktop/Cargo.toml" | cut -d'"' -f2)"
mkdir -p "$DIST"

find_tool() {
    local name="$1"
    if command -v "$name" >/dev/null 2>&1; then
        command -v "$name"
    elif [ -x "$ROOT/packaging/tools/$name" ]; then
        echo "$ROOT/packaging/tools/$name"
    else
        echo ""
    fi
}

echo "==> Building release binary"
(cd "$ROOT" && cargo build --release --locked -p m3top)

echo "==> Arch package (makepkg)"
if command -v makepkg >/dev/null 2>&1; then
    (cd "$ROOT/packaging/arch" && makepkg -f --noconfirm)
    cp "$ROOT"/packaging/arch/m3top-"$VERSION"-*-*.pkg.tar.* "$DIST/"
else
    echo "    makepkg not found, skipping (Arch/pacman-based systems only)"
fi

echo "==> Debian package (cargo-deb)"
if command -v cargo-deb >/dev/null 2>&1; then
    (cd "$ROOT" && cargo deb -p m3top --no-build)
    cp "$ROOT/target/debian/m3top_${VERSION}"*.deb "$DIST/"
else
    echo "    cargo-deb not found, run: cargo install cargo-deb"
fi

echo "==> RPM package (cargo-generate-rpm)"
if command -v cargo-generate-rpm >/dev/null 2>&1; then
    (cd "$ROOT" && cargo generate-rpm -p crates/desktop)
    cp "$ROOT/target/generate-rpm/m3top-${VERSION}"*.rpm "$DIST/"
else
    echo "    cargo-generate-rpm not found, run: cargo install cargo-generate-rpm"
fi

echo "==> AppImage (linuxdeploy + appimagetool)"
LINUXDEPLOY="$(find_tool linuxdeploy-x86_64.AppImage)"
if [ -z "$LINUXDEPLOY" ]; then LINUXDEPLOY="$(find_tool linuxdeploy)"; fi
if [ -n "$LINUXDEPLOY" ]; then
    APPDIR="$(mktemp -d)/M3Top.AppDir"
    mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/share/applications" \
        "$APPDIR/usr/share/icons/hicolor/256x256/apps"
    cp "$ROOT/target/release/m3top" "$APPDIR/usr/bin/m3top"
    cp "$ROOT/packaging/m3top.desktop" "$APPDIR/usr/share/applications/"
    cp "$ROOT/packaging/icons/m3top_256.png" \
        "$APPDIR/usr/share/icons/hicolor/256x256/apps/m3top.png"

    export VERSION
    "$LINUXDEPLOY" --appimage-extract-and-run \
        --appdir "$APPDIR" \
        --executable "$APPDIR/usr/bin/m3top" \
        --desktop-file "$APPDIR/usr/share/applications/m3top.desktop" \
        --icon-file "$ROOT/packaging/icons/m3top_256.png" \
        --output appimage
    mv ./M3Top-"$VERSION"-x86_64.AppImage "$DIST/"
    rm -rf "$(dirname "$APPDIR")"
else
    echo "    linuxdeploy not found; download it from"
    echo "    https://github.com/linuxdeploy/linuxdeploy/releases (continuous build)"
    echo "    and place it as packaging/tools/linuxdeploy-x86_64.AppImage"
fi

echo
echo "==> Done. Packages in $DIST:"
ls -la "$DIST"
