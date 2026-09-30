use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::sx::{Sx, sx};
use crate::theme::{Color, ColorShade, ColorValue, CssVar, Size, TextDefaults};

pub const ANCHOR_COLOR: CssVar = CssVar::new("--lsx-anchor-color");
/// The link colour on inline `Code`'s fill, which the stylesheet rebases (todo 899).
pub(crate) const ANCHOR_CODE_COLOR: CssVar = CssVar::new("--lsx-anchor-code-color");

str_enum! {
    /// When an `Anchor` draws its underline.
    #[state_prefix = "underline"]
    pub enum AnchorUnderline {
        Always = "always",
        #[default]
        Hover = "hover",
        Never = "never",
    }
}

/// Theme defaults for `Anchor`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorDefaults {
    pub size: Size,
    pub underline: AnchorUnderline,
    pub color: Color,
}

impl AnchorDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        underline: AnchorUnderline::Hover,
        color: Color::Primary,
    };

    /// `Text`'s own sizing plus the link colour.
    pub fn theme_vars() -> Sx {
        TextDefaults::theme_vars()
            .color(ANCHOR_COLOR.value())
            .selector("& code", sx().color(ANCHOR_CODE_COLOR.value()))
    }

    /// For a fill whose links take its text colour: an `underline: hover` link is
    /// underlined at rest, or only its position tells it from text (1.4.1, todo 1647).
    pub(crate) fn underline_at_rest() -> Sx {
        sx().selector(
            format!(
                "& a[data-state~=\"{}\"]",
                AnchorUnderline::Hover.state_name()
            ),
            sx().text_decoration("underline"),
        )
    }
}

impl ToCssDeclarations for AnchorDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        // Text ramp: `blue.6` is only 3.56:1 on paper (todo 239).
        let color = ColorValue::Text(self.color, ColorShade::DEFAULT).value();
        vec![
            ANCHOR_COLOR.declare(color.clone()),
            ANCHOR_CODE_COLOR.declare(color),
        ]
    }
}
