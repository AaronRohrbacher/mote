#!/bin/bash
# Build for Raspberry Pi 4 using Podman

set -e

echo "Building for Raspberry Pi 4 using Podman..."

# Build the container image and extract the binary
podman build -t mote-builder .
podman create --name mote-temp mote-builder
podman cp mote-temp:/build/target/aarch64-unknown-linux-gnu/release/desktop-icons ./desktop-icons
podman rm mote-temp

echo "Binary created: ./desktop-icons"
echo "Copy to your Pi with: scp desktop-icons pi@your-pi:/path/to/mote/"

