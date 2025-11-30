# Mote Compositor

A minimal Wayland compositor in Rust with built-in desktop icons.

## Features

- Native Wayland compositor (no Sway needed)
- Built-in desktop icons (Mote and Chromium)
- Direct Wayland protocol handling
- Touchscreen-friendly interface

## Building

```bash
cd compositor
cargo build --release
```

## Running

```bash
# Set up Wayland display
export WAYLAND_DISPLAY=wayland-1

# Run the compositor
./target/release/mote-compositor
```

## Architecture

This is a minimal compositor using Smithay. It:
- Handles Wayland protocol directly
- Manages outputs and displays
- Renders desktop icons natively
- Launches applications
- Manages windows

## Status

This is a foundation/skeleton. Full implementation requires:
- DRM backend integration
- Input handling (touchscreen, keyboard)
- Surface rendering with Cairo/Pango
- Icon click detection
- Window management
- Home button overlay

## Dependencies

- Rust
- Smithay (Wayland compositor framework)
- DRM/libinput (for hardware access)
- Cairo/Pango (for rendering)


