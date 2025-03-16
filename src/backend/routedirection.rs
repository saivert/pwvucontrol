use wireplumber::spa::SpaPod;

#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum, Default)]
#[enum_type(name = "RouteDirection")]
pub enum RouteDirection {
    #[default]
    Unknown = 2,
    Input = 0,
    Output = 1,
}

impl From<u32> for RouteDirection {
    fn from(value: u32) -> Self {
        match value {
            0 => RouteDirection::Input,
            1 => RouteDirection::Output,
            _ => RouteDirection::Unknown,
        }
    }
}

impl<'a> From<&'a SpaPod> for RouteDirection {
    fn from(value: &'a SpaPod) -> Self {
        value.id().map_or(RouteDirection::Unknown, |x| x.into())
    }
}

impl From<RouteDirection> for u32 {
    fn from(value: RouteDirection) -> Self {
        value as u32
    }
}
