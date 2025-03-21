use gtk::gio::{self, prelude::*};

const KEY_WINDOW_WIDTH: &str = "window-width";
const KEY_WINDOW_HEIGHT: &str = "window-height";
const KEY_WINDOW_IS_MAXIMIZED: &str = "is-maximized";

pub fn remember_window_size(window: &gtk::Window, settings: &gio::Settings) {
    settings
        .bind(KEY_WINDOW_WIDTH, window, "default-width")
        .build();
    settings
        .bind(KEY_WINDOW_HEIGHT, window, "default-height")
        .build();
    settings
        .bind(KEY_WINDOW_IS_MAXIMIZED, window, "maximized")
        .build();
}
