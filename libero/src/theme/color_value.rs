use crate::common::starts_with;

use super::{Color, ColorShade};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorValue {
    Shade(Color, ColorShade),
    Contrast(Color, ColorShade),
}

impl ColorValue {
    pub(crate) const fn parse(value: &'static str) -> Option<Self> {
        if starts_with(value, "primary-contrast") {
            return Some(Self::Contrast(Color::Primary, ColorShade::parse(value, 16)));
        }

        if starts_with(value, "secondary-contrast") {
            return Some(Self::Contrast(
                Color::Secondary,
                ColorShade::parse(value, 18),
            ));
        }

        if starts_with(value, "primary") {
            return Some(Self::Shade(Color::Primary, ColorShade::parse(value, 7)));
        }

        if starts_with(value, "secondary") {
            return Some(Self::Shade(Color::Secondary, ColorShade::parse(value, 9)));
        }

        None
    }
}
