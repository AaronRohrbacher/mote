use gtk::prelude::*;
use gtk::{Button, Label, Window, WindowType, Box as GtkBox, Orientation};
use std::process::Command;

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
    gtk::init().expect("Failed to initialize GTK");

    // Use wrapper script with environment variable for password
    let mote_command = "VNC_PASSWORD=KonstaKANG /home/m/vnc-connect.sh &";
    
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

