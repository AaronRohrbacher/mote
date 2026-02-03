# Verification: Icon transparency (no background)

## Summary

The icon implementation was checked against the actual gtk/gdk 0.17 crate sources. One API mismatch was found and fixed.

## References used (crate sources in cargo registry)

1. **gdk 0.17.1 – RGBA**
   - `gdk-0.17.1/src/rgba.rs`: `pub fn new(red: f64, green: f64, blue: f64, alpha: f64) -> RGBA`
   - Usage: `RGBA::new(0.0, 0.0, 0.0, 0.0)` ✓

2. **gdk 0.17.1 – Screen**
   - `gdk-0.17.1/src/auto/screen.rs`: `pub fn default() -> Option<Screen>` (line 104)
   - `pub fn rgba_visual(&self) -> Option<Visual>` (line 37)
   - Usage: `Screen::default()`, `screen.rgba_visual()` ✓

3. **gtk 0.17.1 – override_background_color**
   - **Not exposed** in the high-level gtk crate (no `override_background_color` on `Widget`).
   - **Present in gtk-sys 0.17.0**: `gtk-sys-0.17.0/src/lib.rs` line 26059:
     - `pub fn gtk_widget_override_background_color(widget: *mut GtkWidget, state: GtkStateFlags, color: *const gdk::GdkRGBA);`
   - **Fix**: Call `gtk::ffi::gtk_widget_override_background_color` from Rust with:
     - `widget`: from `window.to_glib_none().0` / `button.to_glib_none().0`
     - `state`: `gtk::StateFlags::NORMAL.bits()` (and PRELIGHT, ACTIVE for button)
     - `color`: `transparent.to_glib_none().0 as *const _`

4. **gtk-sys 0.17.0 – GtkStateFlags**
   - `GtkStateFlags = c_uint`; `GTK_STATE_FLAG_NORMAL = 0`, etc.
   - `gtk::StateFlags` (bitflags) has `.bits()` used for FFI ✓

## Build check

- `cargo check` cannot be run on this host (no GTK/pkg-config).
- Code is consistent with the crate APIs above.
- Recommended: run `cargo build --release` on the Pi (or any system with GTK 3.24) to confirm.

## Result

- Icons use: **RGBA visual** + **transparent background** via GTK3 C API `gtk_widget_override_background_color` (no CSS).
- Implementation is aligned with gtk 0.17 / gdk 0.17 and gtk-sys 0.17.0.
