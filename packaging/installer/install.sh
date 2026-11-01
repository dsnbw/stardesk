#!/usr/bin/env bash
# ==============================================================================
# StarDesk Pure Native Linux Installer (No Wine Required)
# Compatible with ALL Linux distributions:
# Ubuntu, Debian, Fedora, Arch Linux, Manjaro, openSUSE, CentOS/RHEL/Rocky, Alpine
# ==============================================================================
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}====================================================${NC}"
echo -e "${GREEN}      StarDesk Pure Native Linux Installer (v1.5.4)  ${NC}"
echo -e "${BLUE}        100% Native Linux - Zero Wine Required       ${NC}"
echo -e "${BLUE}====================================================${NC}"

# 1. Determine installation paths (system vs user)
if [ "$(id -u)" -eq 0 ]; then
    BIN_DIR="/usr/local/bin"
    UDEV_DIR="/etc/udev/rules.d"
    APP_DIR="/usr/share/applications"
    ICON_DIR="/usr/share/icons/hicolor/256x256/apps"
    USE_SUDO=""
else
    BIN_DIR="${HOME}/.local/bin"
    UDEV_DIR="/etc/udev/rules.d"
    APP_DIR="${HOME}/.local/share/applications"
    ICON_DIR="${HOME}/.local/share/icons/hicolor/256x256/apps"
    USE_SUDO="sudo"
fi

mkdir -p "${BIN_DIR}" "${APP_DIR}" "${ICON_DIR}"

# 2. Distribution Detection & Native Prerequisites
echo -e "\n${BLUE}[1/4] Checking Distribution & Prerequisites...${NC}"
PM=""
if command -v apt-get &>/dev/null; then
    PM="apt"
elif command -v dnf &>/dev/null; then
    PM="dnf"
elif command -v pacman &>/dev/null; then
    PM="pacman"
elif command -v zypper &>/dev/null; then
    PM="zypper"
elif command -v apk &>/dev/null; then
    PM="apk"
fi

echo "Detected package manager: ${PM:-generic}"

# 3. Configure uinput kernel permissions
echo -e "\n${BLUE}[2/4] Configuring Linux Kernel /dev/uinput Permissions...${NC}"
if [ -w /dev/uinput ]; then
    echo -e "${GREEN}✓ /dev/uinput is already writable by current user.${NC}"
else
    if command -v sudo &>/dev/null || [ "$(id -u)" -eq 0 ]; then
        echo "Installing udev rule for /dev/uinput..."
        $USE_SUDO tee "${UDEV_DIR}/60-stardesk-uinput.rules" > /dev/null << 'EOF'
KERNEL=="uinput", SUBSYSTEM=="misc", TAG+="uaccess", OPTIONS+="static_node=uinput"
KERNEL=="uinput", GROUP="input", MODE="0660"
EOF
        $USE_SUDO modprobe uinput || true
        $USE_SUDO udevadm control --reload-rules 2>/dev/null || true
        $USE_SUDO udevadm trigger 2>/dev/null || true
        $USE_SUDO usermod -aG input "${USER:-$(id -un)}" 2>/dev/null || true
        echo -e "${GREEN}✓ Udev rules installed successfully.${NC}"
    else
        echo -e "${YELLOW}Notice: Run with sudo if /dev/uinput permission errors occur.${NC}"
    fi
fi

# 4. Install Native Binary and Desktop Integration
echo -e "\n${BLUE}[3/4] Installing StarDesk Native Files...${NC}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Copy or compile stardesk native binary
if [ -f "${ROOT_DIR}/target/release/stardesk" ]; then
    cp "${ROOT_DIR}/target/release/stardesk" "${BIN_DIR}/stardesk"
elif [ -f "${SCRIPT_DIR}/stardesk" ]; then
    cp "${SCRIPT_DIR}/stardesk" "${BIN_DIR}/stardesk"
elif command -v cargo &>/dev/null && [ -f "${ROOT_DIR}/Cargo.toml" ]; then
    echo "Compiling native binary via Cargo..."
    cargo build --release --manifest-path "${ROOT_DIR}/Cargo.toml"
    cp "${ROOT_DIR}/target/release/stardesk" "${BIN_DIR}/stardesk"
else
    echo "Creating wrapper binary script in ${BIN_DIR}/stardesk..."
    cat > "${BIN_DIR}/stardesk" << 'EOF'
#!/usr/bin/env bash
exec cargo run --release --manifest-path "${HOME}/stardesk/Cargo.toml" -- "$@"
EOF
fi
chmod +x "${BIN_DIR}/stardesk"

# Copy Desktop Entry and Icons
cp "${ROOT_DIR}/packaging/stardesk.desktop" "${APP_DIR}/stardesk.desktop"
cp "${ROOT_DIR}/assets/stardesk.png" "${ICON_DIR}/stardesk.png"

# Update XDG caches if available
command -v update-desktop-database &>/dev/null && update-desktop-database "${APP_DIR}" 2>/dev/null || true
command -v gtk-update-icon-cache &>/dev/null && gtk-update-icon-cache -f -t "$(dirname "${ICON_DIR}")" 2>/dev/null || true

# 5. Verify Installation
echo -e "\n${BLUE}[4/4] Verifying Installation with 'stardesk doctor'...${NC}"
export PATH="${BIN_DIR}:${PATH}"
"${BIN_DIR}/stardesk" doctor || true

echo -e "\n${GREEN}====================================================${NC}"
echo -e "${GREEN}  StarDesk Pure Native Linux Installed Successfully!  ${NC}"
echo -e "${GREEN}====================================================${NC}"
echo "Quick Start Commands:"
echo "  stardesk host     - Start the native host (let other devices control this PC)"
echo "  stardesk login    - Log in via mobile app QR code"
echo "  stardesk viewer   - Connect and control another computer"
echo "  stardesk assist   - Get remote support code without logging in"
echo "  stardesk doctor   - Check display server, GPU, and uinput status"
