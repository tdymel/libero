use crate::{
    common::eq,
    theme::{ColorValue, Size},
};

use super::hash::{hash_str, hash_u8};

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

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Width => "width",
            Self::Height => "height",
            Self::PaddingTop => "padding-top",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Declaration {
    pub property: DeclarationProperty,
    pub value: ThemeAwareValue,
}

impl Declaration {
    pub(crate) const fn hash(self, mut hash: u64) -> u64 {
        hash = match self.property {
            DeclarationProperty::Known(property) => hash_str(hash_u8(hash, 0), property.as_str()),
            DeclarationProperty::Raw(property) => hash_str(hash_u8(hash, 1), property),
        };

        match self.value {
            ThemeAwareValue::Raw(value) => hash_str(hash_u8(hash, 3), value),
            ThemeAwareValue::Color(color_value) => match color_value {
                ColorValue::Shade(color, shade) => hash_str(
                    hash_str(hash_u8(hash_u8(hash, 5), color as u8), shade.as_str()),
                    "",
                ),
                ColorValue::Contrast(color, shade) => hash_str(
                    hash_str(hash_u8(hash_u8(hash, 6), color as u8), shade.as_str()),
                    "",
                ),
            },
            ThemeAwareValue::Size(size) => hash_str(hash_u8(hash, 7), size.as_str()),
        }
    }
}
