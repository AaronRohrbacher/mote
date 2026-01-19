#!/bin/bash
# Script to properly sign and align APK for Google Drive distribution
# Usage: ./sign-apk.sh <keystore-path> <alias> <unsigned-apk-path>

set -e

if [ "$#" -ne 3 ]; then
    echo "Usage: $0 <keystore-path> <alias> <unsigned-apk-path>"
    echo "Example: $0 ~/my-keystore.jks myalias app/build/outputs/apk/release/app-release-unsigned.apk"
    exit 1
fi

KEYSTORE="$1"
ALIAS="$2"
UNSIGNED_APK="$3"
ALIGNED_APK="${UNSIGNED_APK%.apk}-aligned.apk"
SIGNED_APK="${UNSIGNED_APK%-unsigned.apk}.apk"

# Check if tools are available
if ! command -v zipalign &> /dev/null; then
    echo "Error: zipalign not found. Install Android SDK Build Tools."
    exit 1
fi

if ! command -v apksigner &> /dev/null; then
    echo "Error: apksigner not found. Install Android SDK Build Tools."
    exit 1
fi

echo "Step 1: Aligning APK..."
zipalign -f -v 4 "$UNSIGNED_APK" "$ALIGNED_APK"

echo ""
echo "Step 2: Signing APK with v1, v2, and v3 schemes..."
apksigner sign \
    --ks "$KEYSTORE" \
    --ks-key-alias "$ALIAS" \
    --v1-signing-enabled true \
    --v2-signing-enabled true \
    --v3-signing-enabled true \
    --out "$SIGNED_APK" \
    "$ALIGNED_APK"

echo ""
echo "Step 3: Verifying signature..."
apksigner verify --print-certs --verbose "$SIGNED_APK"

echo ""
echo "✅ Successfully signed APK: $SIGNED_APK"
echo "   Ready for Google Drive upload"
