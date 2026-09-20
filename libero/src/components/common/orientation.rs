use super::Input;
use crate::components::common::input_from_str;
use crate::str_enum::str_enum;

str_enum! {
    /// The axis a component lays out along.
    pub enum Orientation {
        #[default]
        Vertical = "vertical",
        Horizontal = "horizontal",
    }
}

input_from_str!(Orientation);

impl From<Orientation> for Input<Orientation> {
    fn from(value: Orientation) -> Self {
        Input::Value(value)
    }
}
