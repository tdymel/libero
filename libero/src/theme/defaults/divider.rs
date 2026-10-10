use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};

use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const DIVIDER_SPACING: CssVar = CssVar::new("--lsx-divider-spacing");
pub const DIVIDER_THICKNESS: SizeCss = SizeCss::new("--lsx-divider-thickness-");
pub const DIVIDER_LINE: CssVar = CssVar::new("--lsx-divider-line");

str_enum! {
    /// Where a `Divider`'s label sits along the line.
    pub enum LabelPosition {
        Start = "start",
        #[default]
        Center = "center",
        End = "end",
    }
}

/// Theme defaults for `Divider`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DividerDefaults {
    /// The line's thickness step when `size` is omitted.
    pub size: Size,
    /// Where the label sits when `label_position` is omitted.
    pub label_position: LabelPosition,
    pub spacing: Option<Size>,
    pub thicknesses: Sizes<u16>,
}

impl DividerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Xs,
        label_position: LabelPosition::Center,
        spacing: None,
        thicknesses: Sizes::new(1, 2, 3, 4, 5, 6),
    };

    pub fn size_sx(size: Size) -> Sx {
        sx().var(DIVIDER_LINE, DIVIDER_THICKNESS.value(size))
    }

    pub fn horizontal_sx() -> Sx {
        sx().margin_top(DIVIDER_SPACING.value())
            .margin_bottom(DIVIDER_SPACING.value())
    }

    pub fn vertical_sx() -> Sx {
        sx().margin_left(DIVIDER_SPACING.value())
            .margin_right(DIVIDER_SPACING.value())
    }
}

impl ToCssDeclarations for DividerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let spacing = match self.spacing {
            Some(size) => SizeCss::SPACING.value(size),
            None => "0".to_string(),
        };
        let mut declarations = self
            .thicknesses
            .to_css_declarations(DIVIDER_THICKNESS, "px");
        declarations.push(DIVIDER_SPACING.declare(spacing));
        declarations
    }
}
