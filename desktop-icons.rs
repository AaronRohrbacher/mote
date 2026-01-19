use gtk::prelude::*;
use gtk::{Button, Image, Label, Window, WindowType, Box as GtkBox, Orientation, IconSize, EventBox, DrawingArea};
use gtk_layer_shell::{Edge, Layer};
use glib::{timeout_add_local, Continue};
use gtk::gdk::{EventButton, EventMask};
use std::process::Command;
use std::env;
use std::time::Duration;
use std::rc::Rc;

struct Icon {
    #[allow(dead_code)] // Must keep window alive
    window: Window,
}

struct VncOverlay {
    #[allow(dead_code)] // Must keep windows alive
    control_window: Rc<Window>,
    #[allow(dead_code)] // Must keep windows alive
    trigger_window: Rc<Window>,
}

impl Icon {
    fn new(name: &str, icon_name: &str, command: &str, x: i32, y: i32) -> Self {
        let window = Window::new(WindowType::Toplevel);
        window.set_decorated(false);
        window.set_skip_taskbar_hint(true);
        window.set_keep_above(true);
        window.set_default_size(150, 150);
        window.move_(x, y);
        window.stick();

        let button = Button::new();
        let box_ = GtkBox::new(Orientation::Vertical, 10);

        let icon = Image::from_icon_name(Some(icon_name), IconSize::Dialog);
        icon.set_pixel_size(64);

        let label_name = Label::new(None);
        label_name.set_markup(&format!("<span font='Sans Bold 16'>{}</span>", name));

        box_.pack_start(&icon, true, true, 0);
        box_.pack_start(&label_name, true, true, 0);
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
    
    if env::args().any(|a| a == "--launch-vnc") {
        log("--launch-vnc flag detected, launching VNC view");
        gtk::init().expect("Failed to initialize GTK");
        let _overlay = launch_vnc_view(); // Store overlay to keep windows alive
        gtk::main();
        return;
    }

    gtk::init().expect("Failed to initialize GTK");

    // Get the path to this executable for launching --launch-vnc
    let exe_path = env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "/home/m/desktop-icons".to_string());

    let _mote = Icon::new(
        "Mote",
        "video-display",
        &format!("{} --launch-vnc", exe_path),
        50,
        50,
    );

    let _chromium = Icon::new(
        "Chromium",
        "web-browser",
        "chromium --ozone-platform=wayland",
        230,
        50,
    );

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
    
    let dialog = gtk::MessageDialog::new(
        None::<&Window>,
        gtk::DialogFlags::MODAL,
        gtk::MessageType::Error,
        gtk::ButtonsType::Ok,
        message,
    );
    dialog.set_title(title);
    dialog.set_position(gtk::WindowPosition::Center);
    dialog.run();
    dialog.close();
}

fn launch_vnc_view() -> VncOverlay {
    log("launch_vnc_view starting");
    
    let vnc_host = env::var("VNC_HOST").unwrap_or_else(|_| "10.1.1.79".to_string());
    let vnc_port = env::var("VNC_PORT").unwrap_or_else(|_| "5900".to_string());
    let vnc_password = env::var("VNC_PASSWORD").unwrap_or_else(|_| "KonstaKANG".to_string());
    
    // Create encrypted VNC password file using vncpasswd
    let passwd_cmd = format!("echo '{}' | vncpasswd -f > /tmp/vnc_passwd", vnc_password);
    Command::new("sh").arg("-c").arg(&passwd_cmd).output().ok();
    
    // Launch VNC
    let vnc_cmd = format!(
        "ssvncviewer -fullscreen -scale 800x480 -encodings 'copyrect zrle hextile' -passwd /tmp/vnc_passwd {}:{}",
        vnc_host, vnc_port
    );
    log(&format!("Running: {}", vnc_cmd));
    
    Command::new("sh").arg("-c").arg(&vnc_cmd).spawn().ok();
    
    // Wait for VNC to appear, then create overlay
    std::thread::sleep(Duration::from_millis(1000));
    
    // Create overlay AFTER VNC launches - Layer::Overlay ensures it's on top
    log("Creating overlay windows");
    let overlay = create_control_overlay(&vnc_host);
    
    log("VNC viewer launched, overlay created");
    overlay
}

fn create_control_overlay(host: &str) -> VncOverlay {
    let window = Window::new(WindowType::Toplevel);
    window.set_decorated(false);
    window.set_default_size(280, 80);
    window.set_keep_above(true);
    window.set_skip_taskbar_hint(true);

    gtk_layer_shell::init_for_window(&window);
    gtk_layer_shell::set_layer(&window, Layer::Overlay);
    gtk_layer_shell::set_anchor(&window, Edge::Top, true);
    gtk_layer_shell::set_anchor(&window, Edge::Right, true);
    gtk_layer_shell::set_margin(&window, Edge::Top, 20);
    gtk_layer_shell::set_margin(&window, Edge::Right, 20);
    gtk_layer_shell::auto_exclusive_zone_enable(&window);

    let button_box = GtkBox::new(Orientation::Horizontal, 10);

    // Volume down - uses HTTP endpoint
    let vol_down = Button::new();
    vol_down.set_label("Vol-");
    let host_vd = host.to_string();
    vol_down.connect_clicked(move |_| {
        send_volume_http(&host_vd, false);
    });
    button_box.pack_start(&vol_down, false, false, 0);

    // Volume up - uses HTTP endpoint
    let vol_up = Button::new();
    vol_up.set_label("Vol+");
    let host_vu = host.to_string();
    vol_up.connect_clicked(move |_| {
        send_volume_http(&host_vu, true);
    });
    button_box.pack_start(&vol_up, false, false, 0);

    // Home button - exits VNC and returns to desktop
    let home_btn = Button::new();
    home_btn.set_label("Home");
    home_btn.connect_clicked(move |_| {
        // Kill VNC viewers
        Command::new("pkill").arg("-f").arg("ssvncviewer").spawn().ok();
        Command::new("pkill").arg("-f").arg("vncviewer").spawn().ok();
        Command::new("swaymsg").args(&["workspace", "1"]).spawn().ok();
        gtk::main_quit();
    });
    button_box.pack_start(&home_btn, false, false, 0);

    window.add(&button_box);
    window.show_all();
    window.present(); // Ensure window is raised and visible
    
    // Auto-hide after 5 seconds, show when top of screen is touched
    let window_rc = Rc::new(window);
    let win_hide = window_rc.clone();
    
    timeout_add_local(Duration::from_secs(5), move || {
        win_hide.hide();
        Continue(false)
    });
    
    // Create trigger zone at top of screen - return it to keep it alive
    let trigger = create_top_trigger(&window_rc);
    let trigger_rc = Rc::new(trigger);
    
    VncOverlay {
        control_window: window_rc,
        trigger_window: trigger_rc,
    }
}

fn create_top_trigger(control_window: &Rc<Window>) -> Window {
    let trigger = Window::new(WindowType::Toplevel);
    trigger.set_decorated(false);
    trigger.set_default_size(300, 48); // Narrower: only middle portion
    
    gtk_layer_shell::init_for_window(&trigger);
    gtk_layer_shell::set_layer(&trigger, Layer::Overlay);
    gtk_layer_shell::set_anchor(&trigger, Edge::Top, true);
    // Only anchor to top, not left/right - center it with margins
    gtk_layer_shell::set_margin(&trigger, Edge::Left, 250); // Center: (800-300)/2 = 250px margin on each side
    gtk_layer_shell::set_margin(&trigger, Edge::Right, 250);
    gtk_layer_shell::auto_exclusive_zone_enable(&trigger);
    
    trigger.set_opacity(0.01); // Nearly invisible but still receives events
    trigger.set_accept_focus(false);
    trigger.set_keep_above(true);
    trigger.set_events(EventMask::TOUCH_MASK | EventMask::BUTTON_PRESS_MASK | EventMask::POINTER_MOTION_MASK | EventMask::ENTER_NOTIFY_MASK);
    
    let event_box = EventBox::new();
    event_box.set_above_child(false); // Receive events on this widget
    event_box.set_visible_window(true); // Need visible window to receive events
    event_box.set_events(EventMask::TOUCH_MASK | EventMask::BUTTON_PRESS_MASK | EventMask::POINTER_MOTION_MASK | EventMask::ENTER_NOTIFY_MASK);
    
    let spacer = DrawingArea::new();
    spacer.set_size_request(300, 48); // Match the window width
    event_box.add(&spacer);
    
    // Handle both mouse and touch events
    let win = control_window.clone();
    event_box.connect_button_press_event(move |_, _| {
        log("Trigger zone: button press detected");
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

fn send_volume_http(host: &str, up: bool) {
    let endpoint = if up { "up" } else { "down" };
    let url = format!("http://{}:8080/volume/{}", host, endpoint);
    log(&format!("Volume {}: {}", endpoint, url));
    
    // Use curl in background - simple and reliable
    std::thread::spawn(move || {
        Command::new("curl")
            .args(&["-s", "-m", "2", &url])
            .output()
            .ok();
    });
}
