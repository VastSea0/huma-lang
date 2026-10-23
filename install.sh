#!/usr/bin/env sh
# Hüma Language Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/VastSea0/huma-lang/main/install.sh | sh

set -e

REPO="VastSea0/huma-lang"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="huma"
GITHUB_API="https://api.github.com/repos/${REPO}/releases/latest"
GITHUB_BASE="https://github.com/${REPO}/releases/latest/download"

# ── Colors ──────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()    { printf "${CYAN}[hüma]${RESET} %s\n" "$1"; }
success() { printf "${GREEN}[hüma]${RESET} ${BOLD}%s${RESET}\n" "$1"; }
error()   { printf "${RED}[error]${RESET} %s\n" "$1" >&2; exit 1; }

# ── Detect OS & Arch ────────────────────────────────────────────────────────
detect_os() {
  case "$(uname -s)" in
    Linux*)  echo "unknown-linux-gnu" ;;
    Darwin*) echo "apple-darwin" ;;
    *)       error "Unsupported OS: $(uname -s). Please build from source." ;;
  esac
}

detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64)  echo "x86_64" ;;
    aarch64|arm64) echo "aarch64" ;;
    *)             error "Unsupported architecture: $(uname -m)." ;;
  esac
}

OS=$(detect_os)
ARCH=$(detect_arch)
ASSET_NAME="huma-${ARCH}-${OS}.tar.gz"

# ── Check dependencies ───────────────────────────────────────────────────────
need_cmd() {
  if ! command -v "$1" > /dev/null 2>&1; then
    error "Required command not found: $1. Please install it and retry."
  fi
}
need_cmd curl
need_cmd tar

# ── Banner ───────────────────────────────────────────────────────────────────
printf "\n"
printf "${BOLD}  ██╗  ██╗██╗   ██╗███╗   ███╗ █████╗ ${RESET}\n"
printf "${BOLD}  ██║  ██║██║   ██║████╗ ████║██╔══██╗${RESET}\n"
printf "${BOLD}  ███████║██║   ██║██╔████╔██║███████║${RESET}\n"
printf "${BOLD}  ██╔══██║██║   ██║██║╚██╔╝██║██╔══██║${RESET}\n"
printf "${BOLD}  ██║  ██║╚██████╔╝██║ ╚═╝ ██║██║  ██║${RESET}\n"
printf "${BOLD}  ╚═╝  ╚═╝ ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝${RESET}\n"
printf "\n"
printf "  Language Installer  —  ${CYAN}github.com/${REPO}${RESET}\n\n"

info "Detected: ${OS}/${ARCH}"
info "Installing to: ${INSTALL_DIR}/${BINARY_NAME}"

# ── Download ─────────────────────────────────────────────────────────────────
DOWNLOAD_URL="${GITHUB_BASE}/${ASSET_NAME}"

info "Downloading from: ${DOWNLOAD_URL}"

TMP_DIR=$(mktemp -d)
TMP_FILE="$TMP_DIR/$ASSET_NAME"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

HTTP_CODE=$(curl -fsSL -w "%{http_code}" -o "$TMP_FILE" "$DOWNLOAD_URL" 2>&1) || true

if [ "$HTTP_CODE" != "200" ] && [ "$HTTP_CODE" != "" ]; then
  # Fallback: try to find asset from releases API
  info "Direct download failed (${HTTP_CODE}), checking GitHub API for assets..."
  if command -v python3 > /dev/null 2>&1; then
    DOWNLOAD_URL=$(curl -fsSL "$GITHUB_API" | python3 -c "
import sys, json
data = json.load(sys.stdin)
assets = data.get('assets', [])
for a in assets:
    name = a.get('name','')
    if '${OS}' in name and '${ARCH}' in name and name.endswith('.tar.gz'):
        print(a['browser_download_url'])
        break
" 2>/dev/null)
  fi

  if [ -z "$DOWNLOAD_URL" ]; then
    error "Could not find a binary for ${ARCH}-${OS}. Visit https://github.com/${REPO}/releases to download manually."
  fi

  info "Found asset: ${DOWNLOAD_URL}"
  curl -fsSL -o "$TMP_FILE" "$DOWNLOAD_URL" || error "Download failed."
fi

# ── Install ───────────────────────────────────────────────────────────────────
info "Extracting..."
tar -xzf "$TMP_FILE" -C "$TMP_DIR"
EXTRACTED_BIN="$TMP_DIR/huma"

if [ ! -f "$EXTRACTED_BIN" ]; then
  error "Extracted binary not found!"
fi

chmod +x "$EXTRACTED_BIN"

# Try to install without sudo first, fall back with sudo
if [ -w "$INSTALL_DIR" ]; then
  cp "$EXTRACTED_BIN" "${INSTALL_DIR}/${BINARY_NAME}"
else
  info "Requesting elevated permissions to install to ${INSTALL_DIR}..."
  sudo cp "$EXTRACTED_BIN" "${INSTALL_DIR}/${BINARY_NAME}"
fi

# ── Verify ────────────────────────────────────────────────────────────────────
if command -v huma > /dev/null 2>&1; then
  success "Hüma installed successfully!"
  printf "\n"
  huma --version 2>/dev/null || true
  printf "\n"
  info "Quick start:"
  printf "  ${CYAN}huma yeni merhaba && cd merhaba${RESET}\n"
  printf "  ${CYAN}huma paket run baslat${RESET}\n"
  printf "\n"
  info "Documentation: https://github.com/${REPO}#readme\n"
else
  error "Installation failed. Binary not found in PATH. Is ${INSTALL_DIR} in your PATH?"
fi
