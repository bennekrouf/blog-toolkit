#!/usr/bin/env bash
# setup-linux.sh — one-time setup for blog-manager on Linux (Debian/Ubuntu/Fedora/Arch)
#
# Installs: WebKitGTK runtime (required by Dioxus desktop), Node.js 20
# Already-installed tools are skipped automatically.
#
# Usage (from the extracted release archive):
#   chmod +x setup-linux.sh && sudo ./setup-linux.sh

set -e

info()  { echo -e "\033[34m[info]\033[0m  $*"; }
ok()    { echo -e "\033[32m[ok]\033[0m    $*"; }
skip()  { echo -e "\033[33m[skip]\033[0m  $*"; }
warn()  { echo -e "\033[33m[warn]\033[0m  $*"; }
err()   { echo -e "\033[31m[error]\033[0m $*"; exit 1; }

# ── Detect distro ─────────────────────────────────────────────────────────────
if   command -v apt-get &>/dev/null; then DISTRO=debian
elif command -v dnf     &>/dev/null; then DISTRO=fedora
elif command -v pacman  &>/dev/null; then DISTRO=arch
else err "Unsupported distro — install libwebkit2gtk-4.1 and Node.js 20 manually"
fi

info "Detected distro family: $DISTRO"

# ── WebKitGTK runtime (required by the Dioxus desktop shell) ──────────────────
case "$DISTRO" in
  debian)
    if ! dpkg -l libwebkit2gtk-4.1-0 &>/dev/null && ! dpkg -l libwebkit2gtk-4.0-0 &>/dev/null; then
      info "Installing libwebkit2gtk..."
      apt-get update -qq
      apt-get install -y libwebkit2gtk-4.1-0 2>/dev/null || apt-get install -y libwebkit2gtk-4.0-0
      ok "WebKitGTK installed"
    else
      skip "WebKitGTK already installed"
    fi
    ;;
  fedora)
    if ! rpm -q webkit2gtk4.1 &>/dev/null && ! rpm -q webkit2gtk3 &>/dev/null; then
      info "Installing WebKitGTK..."
      dnf install -y webkit2gtk4.1 2>/dev/null || dnf install -y webkit2gtk3
      ok "WebKitGTK installed"
    else
      skip "WebKitGTK already installed"
    fi
    ;;
  arch)
    if ! pacman -Qi webkit2gtk-4.1 &>/dev/null && ! pacman -Qi webkit2gtk &>/dev/null; then
      info "Installing WebKitGTK..."
      pacman -S --noconfirm webkit2gtk-4.1 2>/dev/null || pacman -S --noconfirm webkit2gtk
      ok "WebKitGTK installed"
    else
      skip "WebKitGTK already installed"
    fi
    ;;
esac

# ── Node.js 20 (needed by generate-blog-data.js when publishing posts) ────────
install_node() {
  case "$DISTRO" in
    debian)
      info "Installing Node.js 20 via NodeSource..."
      curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
      apt-get install -y nodejs
      ;;
    fedora)
      info "Installing Node.js 20 via NodeSource..."
      curl -fsSL https://rpm.nodesource.com/setup_20.x | bash -
      dnf install -y nodejs
      ;;
    arch)
      info "Installing Node.js..."
      pacman -S --noconfirm nodejs npm
      ;;
  esac
}

if ! command -v node &>/dev/null; then
  install_node
  ok "Node.js installed ($(node --version))"
else
  NODE_MAJOR=$(node --version | sed 's/v\([0-9]*\).*/\1/')
  if [ "$NODE_MAJOR" -lt 18 ]; then
    warn "Node.js $(node --version) is too old — upgrading to v20..."
    install_node
    ok "Node.js upgraded ($(node --version))"
  else
    skip "Node.js already installed ($(node --version))"
  fi
fi

# ── Desktop shortcut ──────────────────────────────────────────────────────────
BINARY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DESKTOP_FILE="/usr/share/applications/blog-manager.desktop"
if [[ -f "$BINARY_DIR/blog-manager" && ! -f "$DESKTOP_FILE" ]]; then
  info "Creating .desktop launcher..."
  cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Name=Blog Manager
Comment=Blog post queue manager for Cvenom
Exec=$BINARY_DIR/blog-manager
Icon=text-editor
Terminal=false
Type=Application
Categories=Office;
EOF
  ok "Desktop shortcut created"
fi

echo ""
echo "Setup complete."
echo "Run: $BINARY_DIR/blog-manager"
