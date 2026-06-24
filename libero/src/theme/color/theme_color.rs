use super::{Color, ColorIndex, IntoColorIndex, Role};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeColor {
    pub palette: Color,
    pub index: ColorIndex,
    pub role: Role,
}

impl ThemeColor {
    pub const fn new(palette: Color, index: ColorIndex, role: Role) -> Self {
        Self {
            palette,
            index,
            role,
        }
    }
}

impl Color {
    pub const fn shade(self, index: impl const IntoColorIndex) -> ThemeColor {
        ThemeColor::new(self, index.into_color_index(), Role::Shade)
    }

    pub const fn at(self, index: impl const IntoColorIndex) -> ThemeColor {
        self.shade(index)
    }

    pub const fn main(self) -> ThemeColor {
        ThemeColor::new(self, ColorIndex::Main, Role::Shade)
    }

    pub const fn contrast(self, index: impl const IntoColorIndex) -> ThemeColor {
        ThemeColor::new(self, index.into_color_index(), Role::Contrast)
    }

    pub const fn contrast_main(self) -> ThemeColor {
        self.contrast(ColorIndex::Main)
    }
}

impl fmt::Display for ThemeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let palette = match self.palette {
            Color::Primary => "color-primary",
            Color::Secondary => "color-secondary",
        };

        let index = self.index.as_u8();

        match self.role {
            Role::Shade => write!(f, "var(--libero-{}-{})", palette, index),
            Role::Contrast => write!(f, "var(--libero-{}-{}-contrast)", palette, index),
        }
    }
}
