use crate::{components::Input, str_enum::str_enum};

str_enum! {
    /// An item's width, in twelfths of its zone.
    #[state_prefix = "span"]
    pub enum GridSpan {
        #[default]
        Full = "full",
        ThreeQuarters = "three-quarters",
        TwoThirds = "two-thirds",
        Half = "half",
        Third = "third",
        Quarter = "quarter",
        Sixth = "sixth",
        Twelfth = "twelfth",
    }
}

crate::components::common::input_from_str!(GridSpan);

impl From<GridSpan> for Input<GridSpan> {
    fn from(value: GridSpan) -> Self {
        Input::Value(value)
    }
}

impl GridSpan {
    /// Columns of the zone's twelve.
    pub const fn columns(self) -> u8 {
        match self {
            Self::Full => 12,
            Self::ThreeQuarters => 9,
            Self::TwoThirds => 8,
            Self::Half => 6,
            Self::Third => 4,
            Self::Quarter => 3,
            Self::Sixth => 2,
            Self::Twelfth => 1,
        }
    }
}
