# Diagnosing APK Installation Failure

To find out why the APK won't install on KonstaKANG AOSP 16, we need actual error messages.

## Method 1: Check via ADB (if available)

```bash
adb install -r app-release.apk
```

This will show the actual error code (e.g., `INSTALL_FAILED_MISSING_FEATURE`, `INSTALL_FAILED_VERIFICATION_FAILURE`, etc.)

## Method 2: Check logcat during installation attempt

```bash
adb logcat | grep -i "packageinstaller\|install\|package"
```

Then try to install the APK and watch for error messages.

## Method 3: Check merged manifest

In Android Studio:
1. Open `app/src/main/AndroidManifest.xml`
2. Click "Merged Manifest" tab at bottom
3. Look for any `<uses-feature>` entries with `android:required="true"` that might not be available on the device

## Method 4: Verify APK structure

```bash
# Check alignment
zipalign -c -v 4 app-release.apk

# Check signing
apksigner verify --print-certs --verbose app-release.apk

# Check for native libraries (should be none)
unzip -l app-release.apk | grep "\.so$"
```

## Common Error Codes:

- `INSTALL_FAILED_MISSING_FEATURE` - Device doesn't have required hardware/software feature
- `INSTALL_FAILED_VERIFICATION_FAILURE` - Developer verification failed (Android 16)
- `INSTALL_FAILED_INVALID_APK` - APK structure/signing issue
- `INSTALL_FAILED_UPDATE_INCOMPATIBLE` - Existing app with different signature
- `INSTALL_FAILED_INSUFFICIENT_STORAGE` - Not enough space

Without the actual error message, we're just guessing.
