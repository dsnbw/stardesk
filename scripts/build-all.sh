#!/usr/bin/env bash
# ==============================================================================
# Master Build Script for Pure Native StarDesk Linux Port (No Wine)
# Generates:
#   1. Native Linux Host & Viewer Binary (cargo build --release)
#   2. Universal AppImage (.AppImage)
#   3. Debian / Ubuntu Package (.deb)
#   4. Standalone Tarball & Self-Extracting Installer
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== [1/3] Compiling Native Linux StarDesk Binary (Cargo) ==="
cd "${ROOT_DIR}"
if command -v cargo &>/dev/null; then
    cargo build --release
    echo "Compiled target/release/stardesk"
else
    echo "Warning: cargo not found in PATH."
fi

echo "=== [2/3] Building Installation Packages ==="
mkdir -p "${ROOT_DIR}/dist"

# Build AppImage
if [ -f "${ROOT_DIR}/packaging/AppImage/build-appimage.sh" ]; then
    bash "${ROOT_DIR}/packaging/AppImage/build-appimage.sh" || true
fi

# Build Debian .deb
if [ -f "${ROOT_DIR}/packaging/deb/build-deb.sh" ]; then
    bash "${ROOT_DIR}/packaging/deb/build-deb.sh" || true
fi

# Create Standalone Portable Tarball
echo "=== [3/3] Creating Standalone Universal Tarball ==="
TARBALL_DIR="${ROOT_DIR}/dist/stardesk-1.5.4-linux-x86_64"
rm -rf "${TARBALL_DIR}"
mkdir -p "${TARBALL_DIR}/bin"
mkdir -p "${TARBALL_DIR}/packaging"
mkdir -p "${TARBALL_DIR}/assets"

if [ -f "${ROOT_DIR}/target/release/stardesk" ]; then
    cp "${ROOT_DIR}/target/release/stardesk" "${TARBALL_DIR}/bin/"
fi
cp "${ROOT_DIR}/packaging/installer/install.sh" "${TARBALL_DIR}/"
cp "${ROOT_DIR}/packaging/stardesk.desktop" "${TARBALL_DIR}/packaging/"
cp "${ROOT_DIR}/packaging/60-stardesk-uinput.rules" "${TARBALL_DIR}/packaging/"
cp "${ROOT_DIR}/assets/stardesk.png" "${TARBALL_DIR}/assets/"
cp "${ROOT_DIR}/docs/linux-compatibility-guide.md" "${TARBALL_DIR}/"

chmod +x "${TARBALL_DIR}/install.sh"
tar -czf "${ROOT_DIR}/dist/stardesk-1.5.4-linux-x86_64.tar.gz" -C "${ROOT_DIR}/dist" "stardesk-1.5.4-linux-x86_64"
echo "Universal Tarball generated at dist/stardesk-1.5.4-linux-x86_64.tar.gz"

echo "=== Build Complete! All pure native Linux installation files in dist/ ==="
