#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Primary,
    Secondary,
    Black,
    White,
}

impl Color {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Black => "black",
            Self::White => "white",
        }
    }
}
