use glib::translate::ToGlibPtr;
use gtk::prelude::*;
use gtk::{Button, Image, Label, Window, WindowType, Box as GtkBox, Orientation, IconSize, EventBox, DrawingArea};
use gtk::gdk::{Screen, RGBA};
use gtk_layer_shell::{Edge, Layer};
use glib::{timeout_add_local, MainLoop, Continue};
use gtk::gdk::EventMask;
use std::process::Command;
use std::env;
use std::time::{Duration, Instant};
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;

const MOTE_ACTIVE_FLAG: &str = "/tmp/mote-active";

struct Icon {
    window: Window,
}

impl Icon {
    fn show(&self) {
        self.window.show_all();
    }
    
    fn hide(&self) {
        self.window.hide();
    }
}

struct MoteOverlay {
    #[allow(dead_code)] // Must keep windows alive
    control_window: Rc<Window>,
    #[allow(dead_code)] // Must keep windows alive
    trigger_window: Rc<Window>,
    #[allow(dead_code)] // Must keep windows alive  
    wake_overlay: Rc<Window>,
}

impl Icon {
    fn new(name: &str, icon_name: &str, command: &str, margin_left: i32) -> Self {
        let window = Window::new(WindowType::Toplevel);
        window.set_decorated(false);
        window.set_skip_taskbar_hint(true);
        window.set_keep_above(true);
        // Fixed small size like real desktop icons (icon + label)
        window.set_default_size(72, 72);
        window.set_size_request(72, 72);

        gtk_layer_shell::init_for_window(&window);
        gtk_layer_shell::set_layer(&window, Layer::Top);
        gtk_layer_shell::set_anchor(&window, Edge::Top, true);
        gtk_layer_shell::set_anchor(&window, Edge::Left, true);
        gtk_layer_shell::set_margin(&window, Edge::Top, 16);
        gtk_layer_shell::set_margin(&window, Edge::Left, margin_left);
        gtk_layer_shell::auto_exclusive_zone_enable(&window);

        // Transparent icon window: RGBA visual, no background (icons only).
        // gtk-rs 0.17 does not expose override_background_color; use FFI (GTK3 C API).
        if let Some(screen) = Screen::default() {
            if let Some(visual) = screen.rgba_visual() {
                window.set_visual(Some(&visual));
            }
        }
        let transparent = RGBA::new(0.0, 0.0, 0.0, 0.0);
        unsafe {
            gtk::ffi::gtk_widget_override_background_color(
                window.upcast_ref::<gtk::Widget>().to_glib_none().0,
                gtk::StateFlags::NORMAL.bits(),
                transparent.to_glib_none().0 as *const _,
            );
        }

        let button = Button::new();
        unsafe {
            gtk::ffi::gtk_widget_override_background_color(
                button.upcast_ref::<gtk::Widget>().to_glib_none().0,
                gtk::StateFlags::NORMAL.bits(),
                transparent.to_glib_none().0 as *const _,
            );
            gtk::ffi::gtk_widget_override_background_color(
                button.upcast_ref::<gtk::Widget>().to_glib_none().0,
                gtk::StateFlags::PRELIGHT.bits(),
                transparent.to_glib_none().0 as *const _,
            );
            gtk::ffi::gtk_widget_override_background_color(
                button.upcast_ref::<gtk::Widget>().to_glib_none().0,
                gtk::StateFlags::ACTIVE.bits(),
                transparent.to_glib_none().0 as *const _,
            );
        }
        let box_ = GtkBox::new(Orientation::Vertical, 2);
        box_.set_size_request(72, 72);

        let icon = Image::from_icon_name(Some(icon_name), IconSize::Dialog);
        icon.set_pixel_size(32);

        let label_name = Label::new(None);
        label_name.set_markup(&format!("<span font='Sans 9'>{}</span>", name));
        label_name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label_name.set_max_width_chars(8);

        // Don't expand: keep icon + label compact
        box_.pack_start(&icon, false, false, 0);
        box_.pack_start(&label_name, false, false, 0);
        button.add(&box_);
        window.add(&button);

        let command_clone = command.to_string();
        let name_clone = name.to_string();
        button.connect_clicked(move |_| {
            log(&format!("Launching: {}", command_clone));
            match Command::new("sh")
                .arg("-c")
                .arg(&command_clone)
                .spawn()
            {
                Ok(_) => {}
                Err(e) => {
                    log(&format!("Failed: {}", e));
                    let dialog = gtk::MessageDialog::new(
                        None::<&Window>,
                        gtk::DialogFlags::MODAL,
                        gtk::MessageType::Error,
                        gtk::ButtonsType::Ok,
                        &format!("Failed to launch {}: {}", name_clone, e),
                    );
                    dialog.set_title("Launch Failed");
                    dialog.run();
                    dialog.close();
                }
            }
        });

        window.show_all();

        Self { window }
    }
}

fn main() {
    log(&format!("desktop-icons starting, args: {:?}", env::args().collect::<Vec<_>>()));
    
    if env::args().any(|a| a == "--mote-view") {
        log("--mote-view flag detected, launching Mote (scrcpy) view");
        // Create flag file to signal main process to hide icons
        std::fs::write(MOTE_ACTIVE_FLAG, "").ok();
        gtk::init().expect("Failed to initialize GTK");
        let _overlay = launch_mote_view();
        gtk::main();
        // Remove flag file on exit so icons reappear
        std::fs::remove_file(MOTE_ACTIVE_FLAG).ok();
        log("Mote view exited, removed mote-active flag");
        return;
    }

    gtk::init().expect("Failed to initialize GTK");

    // Get the path to this executable for launching Mote view
    let exe_path = env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "/home/m/desktop-icons".to_string());

    let mote = Rc::new(Icon::new(
        "Mote",
        "video-display",
        &format!("{} --mote-view", exe_path),
        16,
    ));

    let chromium = Rc::new(Icon::new(
        "Chromium",
        "web-browser",
        "chromium --ozone-platform=wayland",
        16 + 72 + 12,
    ));

    let shutdown = Rc::new(Icon::new(
        "Shutdown",
        "system-shutdown",
        "sudo shutdown -h now",
        16 + (72 + 12) * 2,
    ));

    let reboot = Rc::new(Icon::new(
        "Reboot",
        "system-reboot",
        "sudo reboot",
        16 + (72 + 12) * 3,
    ));

    // Poll for mote-active flag to hide/show icons
    let icons: Vec<Rc<Icon>> = vec![mote, chromium, shutdown, reboot];
    let icons_hidden = Rc::new(RefCell::new(false));
    
    timeout_add_local(Duration::from_millis(500), move || {
        let mote_active = Path::new(MOTE_ACTIVE_FLAG).exists();
        let currently_hidden = *icons_hidden.borrow();
        
        if mote_active && !currently_hidden {
            log("Mote active detected, hiding desktop icons");
            for icon in &icons {
                icon.hide();
            }
            *icons_hidden.borrow_mut() = true;
        } else if !mote_active && currently_hidden {
            log("Mote inactive, showing desktop icons");
            for icon in &icons {
                icon.show();
            }
            *icons_hidden.borrow_mut() = false;
        }
        Continue(true)
    });

    gtk::main();
}

fn log(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true)
        .open("/tmp/desktop-icons.log")
    {
        let _ = writeln!(f, "{}", msg);
    }
}

fn show_error(title: &str, message: &str) {
    log(&format!("ERROR: {} - {}", title, message));
    show_error_overlay(title, message);
}

/// Show error in a layer-shell overlay so it is always on top and visible (not hidden behind other windows).
fn show_error_overlay(title: &str, message: &str) {
    let window = Window::new(WindowType::Toplevel);
    window.set_title(title);
    window.set_decorated(false);
    window.set_default_size(400, 180);
    window.set_keep_above(true);
    window.set_modal(true);

    gtk_layer_shell::init_for_window(&window);
    gtk_layer_shell::set_layer(&window, Layer::Overlay);
    gtk_layer_shell::set_anchor(&window, Edge::Top, true);
    gtk_layer_shell::set_anchor(&window, Edge::Bottom, true);
    gtk_layer_shell::set_anchor(&window, Edge::Left, true);
    gtk_layer_shell::set_anchor(&window, Edge::Right, true);
    gtk_layer_shell::set_margin(&window, Edge::Top, 80);
    gtk_layer_shell::set_margin(&window, Edge::Bottom, 80);
    gtk_layer_shell::set_margin(&window, Edge::Left, 160);
    gtk_layer_shell::set_margin(&window, Edge::Right, 160);
    gtk_layer_shell::auto_exclusive_zone_enable(&window);

    let main_loop = MainLoop::new(None, false);
    let main_loop_clone = main_loop.clone();
    window.connect_destroy(move |_| {
        main_loop_clone.quit();
    });

    let box_ = GtkBox::new(Orientation::Vertical, 12);
    box_.set_margin_top(24);
    box_.set_margin_bottom(24);
    box_.set_margin_start(24);
    box_.set_margin_end(24);

    let title_label = Label::new(None);
    title_label.set_markup(&format!("<b>{}</b>", title));
    title_label.set_line_wrap(true);
    title_label.set_selectable(true);
    box_.pack_start(&title_label, false, false, 0);

    let msg_label = Label::new(None);
    msg_label.set_text(message);
    msg_label.set_line_wrap(true);
    msg_label.set_selectable(true);
    msg_label.set_max_width_chars(50);
    box_.pack_start(&msg_label, true, true, 0);

    let ok_btn = Button::with_label("OK");
    ok_btn.set_margin_top(12);
    let window_clone = window.clone();
    ok_btn.connect_clicked(move |_| {
        window_clone.close();
    });
    box_.pack_end(&ok_btn, false, false, 0);

    window.add(&box_);
    window.show_all();
    window.present();

    main_loop.run();
}

fn launch_mote_view() -> MoteOverlay {
    log("launch_mote_view starting");

    // No desktop icons in Mote view - only scrcpy + control overlay

    // Android device IP (reserved)
    let android_host = env::var("ANDROID_HOST").unwrap_or_else(|_| "10.1.1.3".to_string());
    let adb_port = env::var("ADB_PORT").unwrap_or_else(|_| "5555".to_string());
    let serial = format!("{}:{}", android_host, adb_port);

    // Scrcpy: fullscreen, optimized for 800x480 display
    // -m 480: match display height, greatly improves performance (per scrcpy docs)
    // -f: fullscreen (no window chrome/borders)
    // --no-audio: reduce CPU usage
    let scrcpy_cmd = format!(
        "adb connect {} && scrcpy -s {} -m 480 -f --no-audio",
        serial, serial
    );
    log(&format!("Running: {}", scrcpy_cmd));

    let mut child = match Command::new("sh").arg("-c").arg(&scrcpy_cmd).spawn() {
        Ok(c) => c,
        Err(e) => {
            log(&format!("Failed to start scrcpy: {}", e));
            show_error(
                "Mote failed",
                &format!("Could not start scrcpy: {}. Is adb and scrcpy installed?", e),
            );
            return create_control_overlay(&serial);
        }
    };

    let start = Instant::now();
    std::thread::spawn(move || {
        let status = child.wait();
        let elapsed = start.elapsed();
        if elapsed < Duration::from_secs(10) {
            if let Ok(exit_status) = status {
                if !exit_status.success() {
                    glib::idle_add_local(move || {
                        show_error(
                            "Connection failed",
                            "Could not connect to the Android device. Check that the device is on, USB debugging over network is enabled, and it is reachable at the configured address.",
                        );
                        Continue(false)
                    });
                }
            }
        }
    });

    std::thread::sleep(Duration::from_millis(1000));

    log("Creating overlay windows");
    let overlay = create_control_overlay(&serial);

    let screen_off_delay = env::var("SCREEN_OFF_DELAY")
        .unwrap_or_else(|_| "120".to_string())
        .parse::<u64>()
        .unwrap_or(120);

    // When screen turns off, show the wake overlay so taps can wake it
    let wake_overlay = overlay.wake_overlay.clone();
    timeout_add_local(Duration::from_secs(screen_off_delay), move || {
        log("Screen off timeout fired");
        turn_screen_off();
        wake_overlay.show_all();
        log("Wake overlay shown");
        Continue(false)
    });

    log("Scrcpy launched, overlay created");
    overlay
}

fn create_control_overlay(serial: &str) -> MoteOverlay {
    let window = Window::new(WindowType::Toplevel);
    window.set_decorated(false);
    
    // Control panel: Vol−, Vol+, Home buttons
    const CONTROL_WIDTH: i32 = 240;
    const CONTROL_HEIGHT: i32 = 56;
    const SCREEN_WIDTH: i32 = 800;
    let margin_left = (SCREEN_WIDTH - CONTROL_WIDTH) / 2;
    
    window.set_default_size(CONTROL_WIDTH, CONTROL_HEIGHT);
    window.set_size_request(CONTROL_WIDTH, CONTROL_HEIGHT);
    window.set_keep_above(true);
    window.set_skip_taskbar_hint(true);

    gtk_layer_shell::init_for_window(&window);
    gtk_layer_shell::set_layer(&window, Layer::Overlay);
    // Anchor top-left only so we can position in center via margin
    gtk_layer_shell::set_anchor(&window, Edge::Top, true);
    gtk_layer_shell::set_anchor(&window, Edge::Left, true);
    gtk_layer_shell::set_anchor(&window, Edge::Right, false);
    gtk_layer_shell::set_anchor(&window, Edge::Bottom, false);
    gtk_layer_shell::set_margin(&window, Edge::Top, 8);
    gtk_layer_shell::set_margin(&window, Edge::Left, margin_left);
    gtk_layer_shell::set_exclusive_zone(&window, 0);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk::Align::Center);

    let serial = serial.to_string();

    // Volume Down button
    let vol_down_btn = Button::new();
    vol_down_btn.set_label("Vol −");
    vol_down_btn.set_size_request(64, 48);
    let s = serial.clone();
    vol_down_btn.connect_clicked(move |_| {
        log("Vol− pressed");
        Command::new("adb").args(["-s", &s, "shell", "input", "keyevent", "KEYCODE_VOLUME_DOWN"]).spawn().ok();
    });
    button_box.pack_start(&vol_down_btn, false, false, 0);

    // Volume Up button
    let vol_up_btn = Button::new();
    vol_up_btn.set_label("Vol +");
    vol_up_btn.set_size_request(64, 48);
    let s = serial.clone();
    vol_up_btn.connect_clicked(move |_| {
        log("Vol+ pressed");
        Command::new("adb").args(["-s", &s, "shell", "input", "keyevent", "KEYCODE_VOLUME_UP"]).spawn().ok();
    });
    button_box.pack_start(&vol_up_btn, false, false, 0);

    // Home button - exits Mote view
    let home_btn = Button::new();
    home_btn.set_label("Home");
    home_btn.set_size_request(64, 48);
    home_btn.connect_clicked(move |_| {
        log("Home pressed - exiting Mote view");
        Command::new("pkill").args(["-f", "scrcpy"]).spawn().ok();
        Command::new("swaymsg").args(["workspace", "1"]).spawn().ok();
        gtk::main_quit();
    });
    button_box.pack_start(&home_btn, false, false, 0);

    window.add(&button_box);
    window.show_all();
    window.present();
    
    // Auto-hide after 5 seconds, show when top of screen is touched
    let window_rc = Rc::new(window);
    let win_hide = window_rc.clone();
    
    timeout_add_local(Duration::from_secs(5), move || {
        win_hide.hide();
        Continue(false)
    });
    
    // Create trigger zone at top of screen
    let trigger = create_top_trigger(&window_rc);
    let trigger_rc = Rc::new(trigger);
    
    // Create wake overlay (starts HIDDEN, shown only when screen is off)
    let wake = create_wake_overlay(&window_rc);
    let wake_rc = Rc::new(wake);
    
    MoteOverlay {
        control_window: window_rc,
        trigger_window: trigger_rc,
        wake_overlay: wake_rc,
    }
}

/// Fullscreen wake overlay - starts HIDDEN, shown only when screen is off
/// Tap anywhere to wake screen, then overlay hides itself
fn create_wake_overlay(control_window: &Rc<Window>) -> Window {
    let wake = Window::new(WindowType::Toplevel);
    wake.set_decorated(false);
    
    gtk_layer_shell::init_for_window(&wake);
    gtk_layer_shell::set_layer(&wake, Layer::Overlay);
    gtk_layer_shell::set_anchor(&wake, Edge::Top, true);
    gtk_layer_shell::set_anchor(&wake, Edge::Bottom, true);
    gtk_layer_shell::set_anchor(&wake, Edge::Left, true);
    gtk_layer_shell::set_anchor(&wake, Edge::Right, true);
    gtk_layer_shell::set_exclusive_zone(&wake, -1);
    
    wake.set_opacity(0.01); // Nearly invisible
    wake.set_accept_focus(false);
    wake.set_events(EventMask::TOUCH_MASK | EventMask::BUTTON_PRESS_MASK);
    
    let event_box = EventBox::new();
    event_box.set_above_child(false);
    event_box.set_visible_window(true);
    event_box.set_events(EventMask::TOUCH_MASK | EventMask::BUTTON_PRESS_MASK);
    
    let spacer = DrawingArea::new();
    event_box.add(&spacer);
    
    let win = control_window.clone();
    let wake_ref = wake.clone();
    event_box.connect_button_press_event(move |_, _| {
        log("Wake overlay tapped - waking screen and hiding overlay");
        wake_screen();
        wake_ref.hide(); // Hide wake overlay so scrcpy gets input again
        win.show();
        win.present();
        let win_hide = win.clone();
        timeout_add_local(Duration::from_secs(5), move || {
            win_hide.hide();
            Continue(false)
        });
        gtk::Inhibit(true) // Consume the event
    });
    
    wake.add(&event_box);
    // Realize the window (must show once), then hide immediately
    wake.show_all();
    wake.hide();
    wake
}

fn create_top_trigger(control_window: &Rc<Window>) -> Window {
    let trigger = Window::new(WindowType::Toplevel);
    trigger.set_decorated(false);
    
    // Trigger: centered rectangle at top of screen (not full width!)
    // Size: 160x32 - small enough not to interfere, big enough to tap
    const TRIGGER_WIDTH: i32 = 160;
    const TRIGGER_HEIGHT: i32 = 32;
    const SCREEN_WIDTH: i32 = 800;
    let margin_left = (SCREEN_WIDTH - TRIGGER_WIDTH) / 2; // Center horizontally
    
    trigger.set_default_size(TRIGGER_WIDTH, TRIGGER_HEIGHT);
    trigger.set_size_request(TRIGGER_WIDTH, TRIGGER_HEIGHT);
    
    gtk_layer_shell::init_for_window(&trigger);
    gtk_layer_shell::set_layer(&trigger, Layer::Overlay);
    // Anchor only Top+Left so compositor respects our size (not full width)
    gtk_layer_shell::set_anchor(&trigger, Edge::Top, true);
    gtk_layer_shell::set_anchor(&trigger, Edge::Left, true);
    gtk_layer_shell::set_anchor(&trigger, Edge::Right, false);
    gtk_layer_shell::set_anchor(&trigger, Edge::Bottom, false);
    gtk_layer_shell::set_margin(&trigger, Edge::Top, 0); // Flush with top
    gtk_layer_shell::set_margin(&trigger, Edge::Left, margin_left);
    gtk_layer_shell::set_exclusive_zone(&trigger, 0); // Don't reserve space

    // Nearly invisible trigger zone
    trigger.set_opacity(0.01);
    trigger.set_accept_focus(false);
    trigger.set_keep_above(true);
    trigger.set_events(EventMask::TOUCH_MASK | EventMask::BUTTON_PRESS_MASK | EventMask::POINTER_MOTION_MASK | EventMask::ENTER_NOTIFY_MASK);
    
    let event_box = EventBox::new();
    event_box.set_above_child(false);
    event_box.set_visible_window(true);
    event_box.set_events(EventMask::TOUCH_MASK | EventMask::BUTTON_PRESS_MASK | EventMask::POINTER_MOTION_MASK | EventMask::ENTER_NOTIFY_MASK);
    
    let spacer = DrawingArea::new();
    spacer.set_size_request(TRIGGER_WIDTH, TRIGGER_HEIGHT);
    event_box.add(&spacer);
    
    // Handle both mouse and touch events - wake screen on touch
    let win = control_window.clone();
    event_box.connect_button_press_event(move |_, _| {
        log("Trigger zone: button press detected");
        wake_screen();
        win.show();
        win.present(); // Ensure it's raised
        let win_hide = win.clone();
        timeout_add_local(Duration::from_secs(5), move || {
            win_hide.hide();
            Continue(false)
        });
        gtk::Inhibit(false)
    });
    
    // Also connect to motion/enter events as fallback for touch
    let win2 = control_window.clone();
    event_box.connect_enter_notify_event(move |_, _| {
        log("Trigger zone: enter event detected");
        wake_screen();
        win2.show();
        win2.present();
        let win_hide = win2.clone();
        timeout_add_local(Duration::from_secs(5), move || {
            win_hide.hide();
            Continue(false)
        });
        gtk::Inhibit(false)
    });
    
    // Also handle touch events via motion notify
    let win3 = control_window.clone();
    event_box.connect_motion_notify_event(move |_, _| {
        log("Trigger zone: motion/touch detected");
        wake_screen();
        win3.show();
        win3.present();
        let win_hide = win3.clone();
        timeout_add_local(Duration::from_secs(5), move || {
            win_hide.hide();
            Continue(false)
        });
        gtk::Inhibit(false)
    });
    
    trigger.add(&event_box);
    trigger.show_all();
    trigger.present(); // Ensure trigger window is raised
    trigger
}

fn find_sway_socket() -> Option<String> {
    // Try environment variable first
    if let Ok(sock) = env::var("SWAYSOCK") {
        return Some(sock);
    }
    // Try XDG_RUNTIME_DIR
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("sway-ipc.") && name.ends_with(".sock") {
                    return Some(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    // Try common locations for user 1000
    for uid in [1000, 0] {
        let runtime_dir = format!("/run/user/{}", uid);
        if let Ok(entries) = std::fs::read_dir(&runtime_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("sway-ipc.") && name.ends_with(".sock") {
                    return Some(entry.path().to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

fn run_swaymsg(args: &[&str]) -> bool {
    let socket = find_sway_socket();
    let mut cmd = Command::new("swaymsg");
    if let Some(ref sock) = socket {
        cmd.arg("-s").arg(sock);
    }
    cmd.args(args);
    
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                true
            } else {
                log(&format!("swaymsg {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr)));
                false
            }
        }
        Err(e) => {
            log(&format!("swaymsg not found: {}", e));
            false
        }
    }
}

fn find_backlight() -> Option<String> {
    if let Ok(entries) = std::fs::read_dir("/sys/class/backlight") {
        for entry in entries.flatten() {
            let path = entry.path().join("brightness");
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

fn turn_screen_off() {
    log("Turning screen off");
    // Try backlight control first (works on Pi DSI displays)
    if let Some(backlight) = find_backlight() {
        if std::fs::write(&backlight, "0").is_ok() {
            log("Screen turned off via backlight");
            return;
        }
    }
    // Fallback to swaymsg DPMS
    if run_swaymsg(&["output", "*", "dpms", "off"]) {
        log("Screen turned off via swaymsg");
        return;
    }
    // Fallback to xset (X11)
    if let Ok(output) = Command::new("xset").args(["dpms", "force", "off"]).output() {
        if output.status.success() {
            log("Screen turned off via xset");
            return;
        }
    }
    log("WARNING: Failed to turn screen off");
}

fn wake_screen() {
    log("Waking screen");
    // Try backlight control first (works on Pi DSI displays)
    if let Some(backlight) = find_backlight() {
        if std::fs::write(&backlight, "255").is_ok() {
            log("Screen woken via backlight");
            return;
        }
    }
    // Fallback to swaymsg DPMS
    if run_swaymsg(&["output", "*", "dpms", "on"]) {
        log("Screen woken via swaymsg");
        return;
    }
    // Fallback to xset (X11)
    if let Ok(output) = Command::new("xset").args(["dpms", "force", "on"]).output() {
        if output.status.success() {
            log("Screen woken via xset");
            return;
        }
    }
    // Fallback: simulate a key press to wake
    Command::new("xdotool").args(["key", "Shift"]).output().ok();
    log("Screen wake attempted via xdotool");
}

