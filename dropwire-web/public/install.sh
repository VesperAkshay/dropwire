#!/bin/sh
set -e

echo "Downloading DropWire CLI..."

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux*)     OS_NAME=unknown-linux-gnu;;
    Darwin*)    OS_NAME=apple-darwin;;
    *)          echo "Unsupported OS: $OS"; exit 1;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)     ARCH_NAME=x86_64;;
    arm64)      ARCH_NAME=aarch64;;
    aarch64)    ARCH_NAME=aarch64;;
    *)          echo "Unsupported Architecture: $ARCH"; exit 1;;
esac

TARGET="${ARCH_NAME}-${OS_NAME}"
RELEASE_URL="https://api.github.com/repos/VesperAkshay/dropwire/releases"
# Get the latest release that starts with 'cli-v'
LATEST_TAG=$(curl -s $RELEASE_URL | grep '"tag_name": "cli-v' | head -n 1 | awk -F '"' '{print $4}')

if [ -z "$LATEST_TAG" ]; then
    echo "Could not find a valid CLI release."
    exit 1
fi

echo "Found latest version: $LATEST_TAG"

ASSET_NAME="dropwire-cli-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/VesperAkshay/dropwire/releases/download/${LATEST_TAG}/${ASSET_NAME}"

echo "Downloading from: $DOWNLOAD_URL"
curl -L -o /tmp/$ASSET_NAME $DOWNLOAD_URL

echo "Extracting..."
cd /tmp
tar -xzf $ASSET_NAME

# Install to /usr/local/bin (requires sudo)
echo "Installing to /usr/local/bin (you may be prompted for your password)"
sudo mv dropwire /usr/local/bin/
sudo chmod +x /usr/local/bin/dropwire

# Cleanup
rm /tmp/$ASSET_NAME

echo ""
echo "=================================="
echo "DropWire CLI installed successfully!"
echo "Run 'dropwire --help' to get started."
echo "=================================="
