use wireplumber::spa::SpaPod;

#[derive(Debug, Copy, Clone, PartialEq, Eq, glib::Enum, Default)]
#[enum_type(name = "ProfileAvailability")]
pub enum ParamAvailability {
    #[default]
    Unknown,
    No,
    Yes
}

impl From<u32> for ParamAvailability {
    fn from(value: u32) -> Self {
        match value {
            1 => ParamAvailability::No,
            2 => ParamAvailability::Yes,
            _ => ParamAvailability::Unknown,
        }
    }
}


impl<'a> From<&'a SpaPod> for ParamAvailability {
    fn from(value: &'a SpaPod) -> Self {
        value.id().map_or(ParamAvailability::Unknown, |x| x.into())
    }
}
