#!/bin/bash
# tg-snatch installer for macOS and Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/himanshu64/tg-snatch/main/packaging/scripts/install.sh | bash

set -euo pipefail

REPO="himanshu64/tg-snatch"
BINARY="tg-snatch"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info() { echo -e "${CYAN}${BOLD}>>>${NC} $1"; }
success() { echo -e "${GREEN}${BOLD}>>>${NC} $1"; }
error() { echo -e "${RED}${BOLD}ERROR:${NC} $1" >&2; exit 1; }

# Detect platform
detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)  os="unknown-linux-gnu" ;;
        Darwin*) os="apple-darwin" ;;
        *)       error "Unsupported OS: $(uname -s). Use Windows installer for Windows." ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)             error "Unsupported architecture: $(uname -m)" ;;
    esac

    echo "${arch}-${os}"
}

# Get latest version
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | head -1 \
        | sed 's/.*"tag_name": *"//;s/".*//'
}

main() {
    echo ""
    echo -e "${CYAN}${BOLD}  ╔════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}${BOLD}  ║  tg-snatch  ·  Installer              ║${NC}"
    echo -e "${CYAN}${BOLD}  ╚════════════════════════════════════════╝${NC}"
    echo ""

    # Check for curl
    command -v curl >/dev/null 2>&1 || error "curl is required but not installed."

    local platform
    platform=$(detect_platform)
    info "Detected platform: ${platform}"

    local version
    version=$(get_latest_version)
    if [ -z "$version" ]; then
        error "Could not determine latest version. Check https://github.com/${REPO}/releases"
    fi
    info "Latest version: ${version}"

    local url="https://github.com/${REPO}/releases/download/${version}/${BINARY}-${platform}.tar.gz"
    local tmpdir
    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    info "Downloading ${BINARY} ${version}..."
    curl -fsSL "$url" -o "${tmpdir}/${BINARY}.tar.gz" \
        || error "Download failed. Check if release exists: ${url}"

    info "Extracting..."
    tar xzf "${tmpdir}/${BINARY}.tar.gz" -C "$tmpdir"

    info "Installing to ${INSTALL_DIR}..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    else
        sudo mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    fi
    chmod +x "${INSTALL_DIR}/${BINARY}"

    success "tg-snatch ${version} installed successfully!"
    echo ""
    echo "  Run 'tg-snatch' to get started."
    echo ""
}

main "$@"
