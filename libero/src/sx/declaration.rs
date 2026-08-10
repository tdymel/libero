use crate::{
    common::eq,
    theme::{ColorValue, Size},
};

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Property {
    Background,
    Width,
    Height,
    PaddingTop,
}

impl Property {
    pub const fn parse(property: &'static str) -> Option<Self> {
        if eq(property, "background") {
            return Some(Self::Background);
        }

        if eq(property, "width") {
            return Some(Self::Width);
        }

        if eq(property, "height") {
            return Some(Self::Height);
        }

        if eq(property, "padding-top") {
            return Some(Self::PaddingTop);
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationProperty {
    Known(Property),
    Raw(&'static str),
}

impl DeclarationProperty {
    pub const fn parse(property: &'static str) -> Self {
        match Property::parse(property) {
            Some(property) => Self::Known(property),
            None => Self::Raw(property),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeAwareValue {
    Raw(&'static str),
    Color(ColorValue),
    Size(Size),
}

impl ThemeAwareValue {
    pub const fn parse(value: &'static str) -> Self {
        if let Some(color_value) = ColorValue::parse(value) {
            return Self::Color(color_value);
        }

        if let Some(size) = Size::parse(value) {
            return Self::Size(size);
        }

        Self::Raw(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Declaration {
    pub property: DeclarationProperty,
    pub value: ThemeAwareValue,
}
