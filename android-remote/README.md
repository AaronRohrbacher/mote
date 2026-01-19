# Mote Remote

Lightweight Android screen sharing app with HTTP-based volume control. No login required - just works on your local network.

## Features

- **MJPEG Screen Streaming**: View your Android screen from any device
- **HTTP Volume Control**: Simple REST API for volume up/down
- **Zero Config**: Auto-discovers IP, no pairing or authentication needed
- **Low Latency**: Optimized for real-time viewing

## API Endpoints

All endpoints run on port `8080`:

| Endpoint | Description |
|----------|-------------|
| `GET /stream` | MJPEG video stream |
| `GET /volume/up` | Increase volume |
| `GET /volume/down` | Decrease volume |
| `GET /volume/mute` | Toggle mute |
| `GET /volume/set/N` | Set volume to level N |
| `GET /status` | Get current volume and screen info |

## Building

### Prerequisites
- Android Studio (or command line with Android SDK)
- JDK 17+

### Build APK

```bash
cd android-remote
./gradlew assembleDebug
```

APK will be at `app/build/outputs/apk/debug/app-debug.apk`

### Install
```bash
adb install app/build/outputs/apk/debug/app-debug.apk
```

Or transfer the APK to your Android device and install manually.

## Usage

1. Launch "Mote Remote" on your Android device
2. Grant screen capture permission when prompted
3. Note the IP address shown (e.g., `http://10.1.1.79:8080`)
4. App minimizes to background with notification

### From Pi (with mpv):
```bash
# View stream
mpv --fullscreen http://10.1.1.79:8080/stream

# Volume control
curl http://10.1.1.79:8080/volume/up
curl http://10.1.1.79:8080/volume/down
```

### From Browser:
Just open `http://10.1.1.79:8080/stream` in any browser.

## Pi Integration

Set the `ANDROID_HOST` environment variable to your Android's IP:

```bash
export ANDROID_HOST=10.1.1.79
```

The `desktop-icons` app on the Pi will automatically use this for streaming and volume control.

## Requirements

- Android 10+ (API 29)
- Same WiFi network as viewing device
- Port 8080 must not be blocked by firewall
