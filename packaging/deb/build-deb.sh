#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PKGDIR="${ROOT_DIR}/build/deb-package"

echo "=== Building StarDesk .deb Package ==="

rm -rf "${PKGDIR}"
mkdir -p "${PKGDIR}/DEBIAN"
mkdir -p "${PKGDIR}/usr/bin"
mkdir -p "${PKGDIR}/usr/lib/stardesk/hook"
mkdir -p "${PKGDIR}/usr/lib/udev/rules.d"
mkdir -p "${PKGDIR}/usr/share/applications"
mkdir -p "${PKGDIR}/usr/share/icons/hicolor/256x256/apps"

# 1. Copy Debian control & scripts
cp "${SCRIPT_DIR}/control" "${PKGDIR}/DEBIAN/control"
cp "${SCRIPT_DIR}/postinst" "${PKGDIR}/DEBIAN/postinst"
cp "${SCRIPT_DIR}/prerm" "${PKGDIR}/DEBIAN/prerm"
chmod 755 "${PKGDIR}/DEBIAN/postinst" "${PKGDIR}/DEBIAN/prerm"

# 2. Copy Binaries & Assets
if [ -f "${ROOT_DIR}/target/release/stardesk" ]; then
    cp "${ROOT_DIR}/target/release/stardesk" "${PKGDIR}/usr/bin/stardesk"
else
    touch "${PKGDIR}/usr/bin/stardesk"
fi
chmod 755 "${PKGDIR}/usr/bin/stardesk"

if [ -d "${ROOT_DIR}/hook" ]; then
    cp "${ROOT_DIR}/hook"/*.dll "${PKGDIR}/usr/lib/stardesk/hook/" 2>/dev/null || true
fi

cp "${ROOT_DIR}/packaging/stardesk.desktop" "${PKGDIR}/usr/share/applications/"
cp "${ROOT_DIR}/assets/stardesk.png" "${PKGDIR}/usr/share/icons/hicolor/256x256/apps/"
cp "${ROOT_DIR}/packaging/60-stardesk-uinput.rules" "${PKGDIR}/usr/lib/udev/rules.d/"

# 3. Build .deb package
OUT_FILE="${ROOT_DIR}/stardesk_1.5.4_amd64.deb"
dpkg-deb --build --root-owner-group "${PKGDIR}" "${OUT_FILE}"
echo "Debian package created successfully at ${OUT_FILE}"
