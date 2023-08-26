#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Direction {
    Unspecified = 0,
    Asc = 1,
    Desc = 2,
}
impl Direction {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Direction::Unspecified => "DIRECTION_UNSPECIFIED",
            Direction::Asc => "DIRECTION_ASC",
            Direction::Desc => "DIRECTION_DESC",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DIRECTION_UNSPECIFIED" => Some(Self::Unspecified),
            "DIRECTION_ASC" => Some(Self::Asc),
            "DIRECTION_DESC" => Some(Self::Desc),
            _ => None,
        }
    }
}
