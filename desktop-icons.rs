use gtk::prelude::*;
use gtk::{Button, Label, Window, WindowType, Box as GtkBox, Orientation};
use gtk_layer_shell::{Edge, Layer};
use std::process::Command;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

struct Icon {
    window: Window,
    command: String,
}

impl Icon {
    fn new(name: &str, emoji: &str, command: &str, x: i32, y: i32) -> Self {
        let window = Window::new(WindowType::Toplevel);
        window.set_decorated(false);
        window.set_skip_taskbar_hint(true);
        window.set_keep_above(true);
        window.set_default_size(150, 150);
        window.move_(x, y);
        window.stick();

        let button = Button::new();
        let box_ = GtkBox::new(Orientation::Vertical, 10);

        let label_emoji = Label::new(None);
        label_emoji.set_markup(&format!("<span font='Sans 72'>{}</span>", emoji));

        let label_name = Label::new(None);
        label_name.set_markup(&format!("<span font='Sans Bold 16'>{}</span>", name));

        box_.pack_start(&label_emoji, true, true, 0);
        box_.pack_start(&label_name, true, true, 0);
        button.add(&box_);
        window.add(&button);

        let command_clone = command.to_string();
        button.connect_clicked(move |_| {
            Command::new("sh")
                .arg("-c")
                .arg(&command_clone)
                .spawn()
                .ok();
        });

        window.show_all();

        Self {
            window,
            command: command.to_string(),
        }
    }
}

fn main() {
    if env::args().any(|a| a == "--launch-vnc") {
        gtk::init().expect("Failed to initialize GTK");
        launch_vnc();
        gtk::main();
        return;
    }

    gtk::init().expect("Failed to initialize GTK");

    let mote_command = "/home/m/desktop-icons --launch-vnc";
    
    let _mote = Icon::new(
        "Mote",
        "🖥️",
        &mote_command,
        50,
        50,
    );

    let _chromium = Icon::new(
        "Chromium",
        "🌐",
        "cromium &",
        230,
        50,
    );

    gtk::main();
}

fn launch_vnc() {
    let vnc_password = "KonstaKANG";
    let vnc_host = "10.1.1.79";
    let passwd_file = format!("/tmp/mote_vnc_passwd_{}", std::process::id());

    if let Ok(mut child) = Command::new("vncpasswd")
        .arg("-f")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(vnc_password.as_bytes());
            let _ = stdin.flush();
        }
        if let Ok(output) = child.wait_with_output() {
            if output.status.success() {
                if let Ok(mut file) = fs::File::create(&passwd_file) {
                    let _ = std::io::Write::write_all(&mut file, &output.stdout);
                    let mut perms = fs::metadata(&passwd_file).unwrap().permissions();
                    perms.set_mode(0o600);
                    let _ = fs::set_permissions(&passwd_file, perms);
                }
            }
        }
    }

    Command::new("ssvncviewer")
        .args(&["-quality", "9", "-compresslevel", "0", "-fullscreen", "-scale", "800x480", "-passwd", &passwd_file, vnc_host])
        .spawn()
        .ok();

    let passwd_file_clone = passwd_file.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(5));
        let _ = fs::remove_file(&passwd_file_clone);
    });

    std::thread::sleep(std::time::Duration::from_millis(1000));
    
    create_home_button_overlay();
}

fn create_home_button_overlay() {
    let window = Window::new(WindowType::Toplevel);
    window.set_decorated(false);
    window.set_default_size(80, 80);
    
    gtk_layer_shell::init_for_window(&window);
    gtk_layer_shell::set_layer(&window, Layer::Overlay);
    gtk_layer_shell::set_anchor(&window, Edge::Top, true);
    gtk_layer_shell::set_anchor(&window, Edge::Right, true);
    gtk_layer_shell::set_margin(&window, Edge::Top, 20);
    gtk_layer_shell::set_margin(&window, Edge::Right, 20);
    gtk_layer_shell::auto_exclusive_zone_enable(&window);
    
    let button = Button::new();
    let label = Label::new(None);
    label.set_markup("<span font='Sans 48'>🏠</span>");
    button.add(&label);
    window.add(&button);
    
    button.connect_clicked(move |_| {
        Command::new("pkill")
            .arg("-f")
            .arg("ssvncviewer.*10.1.1.79")
            .spawn()
            .ok();
        Command::new("swaymsg")
            .arg("workspace")
            .arg("1")
            .spawn()
            .ok();
        gtk::main_quit();
    });
    
    window.show_all();
}

