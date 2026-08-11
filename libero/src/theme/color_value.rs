use crate::common::{eq, starts_with};

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

        if starts_with(value, "error-contrast") {
            return Some(Self::Contrast(Color::Error, ColorShade::parse(value, 14)));
        }

        if starts_with(value, "warning-contrast") {
            return Some(Self::Contrast(Color::Warning, ColorShade::parse(value, 16)));
        }

        if starts_with(value, "info-contrast") {
            return Some(Self::Contrast(Color::Info, ColorShade::parse(value, 13)));
        }

        if starts_with(value, "success-contrast") {
            return Some(Self::Contrast(Color::Success, ColorShade::parse(value, 16)));
        }

        if starts_with(value, "grey-contrast") {
            return Some(Self::Contrast(Color::Grey, ColorShade::parse(value, 13)));
        }

        if starts_with(value, "primary") {
            return Some(Self::Shade(Color::Primary, ColorShade::parse(value, 7)));
        }

        if starts_with(value, "secondary") {
            return Some(Self::Shade(Color::Secondary, ColorShade::parse(value, 9)));
        }

        if starts_with(value, "error") {
            return Some(Self::Shade(Color::Error, ColorShade::parse(value, 5)));
        }

        if starts_with(value, "warning") {
            return Some(Self::Shade(Color::Warning, ColorShade::parse(value, 7)));
        }

        if starts_with(value, "info") {
            return Some(Self::Shade(Color::Info, ColorShade::parse(value, 4)));
        }

        if starts_with(value, "success") {
            return Some(Self::Shade(Color::Success, ColorShade::parse(value, 7)));
        }

        if starts_with(value, "grey") {
            return Some(Self::Shade(Color::Grey, ColorShade::parse(value, 4)));
        }

        if eq(value, "black") {
            return Some(Self::Shade(Color::Black, ColorShade::S1));
        }

        if eq(value, "white") {
            return Some(Self::Shade(Color::White, ColorShade::S1));
        }

        None
    }
}
