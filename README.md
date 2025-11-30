# Sway Desktop Icons Setup

**For Raspberry Pi 4 running Raspberry Pi OS Lite with Wayland/Sway**

## Files
- `desktop-icons.rs` - Rust program that creates desktop icons and VNC overlay
- `Cargo.toml` - Rust project configuration
- `Makefile` - Build script
- `sway-config-snippet` - Add this to your `~/.config/sway/config`

## Building Options

### Option 1: Cross-compile using Podman (Recommended)

Build on a more powerful machine using Podman to avoid installing 1GB+ of dependencies:

```bash
# On your build machine (Mac/Linux with Podman):
# Make sure Podman is installed and running

# Build for Pi (64-bit ARM)
make pi4

# This will:
# 1. Build a container with ARM64 cross-compilation tools
# 2. Compile the binary inside the container
# 3. Extract the binary to ./desktop-icons

# Copy the binary to your Pi home directory
scp desktop-icons pi@your-pi:~/
```

**Note:** This builds for 64-bit ARM (aarch64). If you're running 32-bit Raspberry Pi OS, you'll need to modify the Dockerfile to use `armv7-unknown-linux-gnueabihf` instead.

**Requirements:** Podman must be installed on your build machine.

### Option 2: Build on Pi (Requires ~1GB dependencies)

If you must build on the Pi:

1. **Install dependencies:**
   ```bash
   sudo apt update && sudo apt install rustc cargo libgtk-3-dev libgtk-layer-shell-dev pkg-config libcairo2-dev libpango1.0-dev libgdk-pixbuf2.0-dev libglib2.0-dev
   ```
   **Warning:** This installs ~150 packages (~1GB). Consider cross-compiling instead.

2. **Build:**
   ```bash
   cd /path/to/mote
   make
   ```

## Runtime Dependencies (Much Lighter)

Once you have the binary, you only need runtime libraries on the Pi:
- **Raspberry Pi OS:** `sudo apt install libgtk-3-0 libgtk-layer-shell0 libcairo2 libpango-1.0-0 libgdk-pixbuf-2.0-0 libglib2.0-0 tigervnc-standalone-server`
- These are much smaller (~50MB total)
- `libgtk-layer-shell0` is required for the home button overlay
- `tigervnc-standalone-server` provides `vncpasswd` utility for creating VNC password files

**If you get "cannot execute: required file not found":**
1. Check the binary architecture: `file /home/m/desktop-icons` (should show ARM)
2. Check for missing libraries: `ldd /home/m/desktop-icons` (shows which libraries are missing)
3. Install any missing libraries shown by `ldd`

## Setup

1. **Get the binary** (either cross-compile or build on Pi)

2. **Install runtime dependencies:**
   ```bash
   sudo apt install libgtk-3-0 libgtk-layer-shell0 libcairo2 libpango-1.0-0 libgdk-pixbuf-2.0-0 libglib2.0-0 tigervnc-standalone-server
   ```

3. **Add to your Sway config:**
   Copy the contents of `sway-config-snippet` into `~/.config/sway/config`
   
   **Important:** Update the path in the config snippet if your username is not `m` (e.g., `/home/pi/desktop-icons`)
   
   The binary should be in your home directory: `desktop-icons`

4. **Reload Sway:**
   `$mod+Shift+c` or `swaymsg reload`

## How it works

- Two desktop icons appear on startup (Mote and Chromium)
- Click an icon to launch the app
- Icons stay visible after clicking
- When Mote is running, a home button appears in the upper right corner
- Home button closes VNC and returns to desktop

## Customization

- Icon positions: Edit `desktop-icons.rs` (x, y coordinates in `main()`)
- Icon size: Edit `desktop-icons.rs` (default size in `set_default_size()`)
- Commands: Edit the command strings in `desktop-icons.rs`
- VNC password: Edit `desktop-icons.rs` (line 88) and replace `"KonstaKANG"` with your actual VNC password, then rebuild: `make pi4`
