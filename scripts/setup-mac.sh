#!/bin/bash
# setup-mac.sh — one-time setup for blog-manager on macOS
#
# Installs: Node.js 20 (needed by generate-blog-data.js when publishing posts)
# WebKit is built into macOS — no separate install needed.
# Already-installed tools at a sufficient version are skipped.

set -e

info()  { echo "[info]  $*"; }
ok()    { echo "[ok]    $*"; }
skip()  { echo "[skip]  $*"; }
warn()  { echo "[warn]  $*"; }

# ── Homebrew ──────────────────────────────────────────────────────────────────
if ! command -v brew &>/dev/null; then
    info "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    eval "$(/opt/homebrew/bin/brew shellenv 2>/dev/null || /usr/local/bin/brew shellenv)"
    ok "Homebrew installed"
else
    skip "Homebrew already installed ($(brew --version | head -1))"
fi

# ── Node.js 20 ────────────────────────────────────────────────────────────────
if ! command -v node &>/dev/null; then
    info "Installing Node.js 20..."
    brew install node@20
    brew link --force --overwrite node@20
    ok "Node.js installed ($(node --version))"
else
    NODE_MAJOR=$(node --version | sed 's/v\([0-9]*\).*/\1/')
    if [ "$NODE_MAJOR" -lt 18 ]; then
        warn "Node.js $(node --version) is too old — upgrading to v20..."
        brew install node@20
        brew link --force --overwrite node@20
        ok "Node.js upgraded ($(node --version))"
    else
        skip "Node.js already installed ($(node --version))"
    fi
fi

# ── Verify the binary ─────────────────────────────────────────────────────────
BINARY_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "$BINARY_DIR/blog-manager" ]]; then
    ok "blog-manager binary found: $BINARY_DIR/blog-manager"
else
    warn "blog-manager binary not found in $BINARY_DIR — move it here before running."
fi

echo ""
echo "Setup complete."
echo "Run: $BINARY_DIR/blog-manager"
