use crate::theme::{ColorValue, Size};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeAwareValue {
    Raw(&'static str),
    Color(ColorValue),
    Spacing(Size),
}

impl ThemeAwareValue {
    pub const fn parse(value: &'static str) -> Self {
        if let Some(color_value) = ColorValue::parse(value) {
            return Self::Color(color_value);
        }

        if let Some(size) = Size::parse(value) {
            return Self::Spacing(size);
        }

        Self::Raw(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Declaration {
    pub property: &'static str,
    pub value: ThemeAwareValue,
}
