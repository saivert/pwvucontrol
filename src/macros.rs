#[macro_export]
macro_rules! pwvucontrol_info {
    ($format:literal $($args:tt)*) => {
        wireplumber::log::info! { domain: "pwvucontrol", $format $($args)* }
    };
}
pub use pwvucontrol_info;

#[macro_export]
macro_rules! pwvucontrol_debug {
    ($format:literal $($args:tt)*) => {
        wireplumber::log::debug! { domain: "pwvucontrol", $format $($args)* }
    };
}
pub use pwvucontrol_debug;

#[macro_export]
macro_rules! pwvucontrol_warning {
    ($format:literal $($args:tt)*) => {
        wireplumber::log::warning! { domain: "pwvucontrol", $format $($args)* }
    };
}
pub use pwvucontrol_warning;

#[macro_export]
macro_rules! pwvucontrol_critical {
    ($format:literal $($args:tt)*) => {
        wireplumber::log::critical! { domain: "pwvucontrol", $format $($args)* }
    };
}
pub use pwvucontrol_critical;

#[macro_export]
macro_rules! pwvucontrol_hex_to_rgba {
    ($r:literal $g:literal $b:literal) => {
        gtk::gdk::RGBA::new($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0, 1.0)
    };
}
pub use pwvucontrol_hex_to_rgba;
