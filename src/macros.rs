#[macro_export]
macro_rules! pwvucontrol_info {
    ($format:literal $($args:tt)*) => {
        wireplumber::log::info! { domain: crate::TOPIC, $format $($args)* }
    };
}
pub use pwvucontrol_info;

#[macro_export]
macro_rules! pwvucontrol_debug {
    ($format:literal $($args:tt)*) => {
        wireplumber::log::debug! { domain: crate::TOPIC, $format $($args)* }
    };
}
pub use pwvucontrol_debug;

#[macro_export]
macro_rules! pwvucontrol_warning {
    ($format:literal $($args:tt)*) => {
        wireplumber::log::warning! { domain: crate::TOPIC, $format $($args)* }
    };
}
pub use pwvucontrol_warning;

#[macro_export]
macro_rules! pwvucontrol_critical {
    ($format:literal $($args:tt)*) => {
        wireplumber::log::critical! { domain: crate::TOPIC, $format $($args)* }
    };
}
pub use pwvucontrol_critical;
