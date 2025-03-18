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
