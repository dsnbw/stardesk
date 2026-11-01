#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
APPDIR="${ROOT_DIR}/build/AppDir"

echo "=== Building StarDesk Native Linux AppImage ==="

rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/lib/udev/rules.d"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"

# 1. Copy AppRun
cp "${SCRIPT_DIR}/AppRun" "${APPDIR}/AppRun"
chmod +x "${APPDIR}/AppRun"

# 2. Copy Native Binary
if [ -f "${ROOT_DIR}/target/release/stardesk" ]; then
    cp "${ROOT_DIR}/target/release/stardesk" "${APPDIR}/usr/bin/stardesk"
else
    echo "Warning: target/release/stardesk not found; copying prebuilt or stub"
    touch "${APPDIR}/usr/bin/stardesk"
fi
chmod +x "${APPDIR}/usr/bin/stardesk"

# 3. Copy Desktop Entry, Icon & Udev rules
cp "${ROOT_DIR}/packaging/stardesk.desktop" "${APPDIR}/stardesk.desktop"
cp "${ROOT_DIR}/packaging/stardesk.desktop" "${APPDIR}/usr/share/applications/stardesk.desktop"
cp "${ROOT_DIR}/assets/stardesk.png" "${APPDIR}/stardesk.png"
cp "${ROOT_DIR}/assets/stardesk.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/stardesk.png"
cp "${ROOT_DIR}/packaging/60-stardesk-uinput.rules" "${APPDIR}/usr/lib/udev/rules.d/"

# 4. Build AppImage using appimagetool if available
OUT_NAME="StarDesk-1.5.4-x86_64.AppImage"
if command -v appimagetool &>/dev/null; then
    ARCH=x86_64 appimagetool "${APPDIR}" "${ROOT_DIR}/${OUT_NAME}"
    echo "AppImage created successfully at ${ROOT_DIR}/${OUT_NAME}"
else
    echo "appimagetool not found in PATH."
    echo "Run manually when appimagetool is available: ARCH=x86_64 appimagetool \"${APPDIR}\" \"${ROOT_DIR}/${OUT_NAME}\""
fi
