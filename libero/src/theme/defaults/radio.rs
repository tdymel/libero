use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{ChoiceVariant, CssVar, Size, SizeCss, Sizes};
use crate::tokens::Color;

pub const RADIO_CIRCLE_SIZE: SizeCss = SizeCss::new("--lsx-radio-circle-size-");

/// Resolved on the control; the circle and dot inherit it.
pub const RADIO_CIRCLE: CssVar = CssVar::new("--lsx-radio-circle");

/// Theme defaults for `Radio`, set on [`Theme`](crate::theme::Theme).
/// No `radius`: the circle is what tells a radio from a `Checkbox`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadioDefaults {
    /// The wrapper a radio, or a `RadioGroup`'s radios, take when a call site names none.
    pub variant: ChoiceVariant,
    pub size: Size,
    /// The ring and dot colour a checked radio takes when a call site names none.
    pub color: Color,
    pub sizes: Sizes<&'static str>,
}

impl RadioDefaults {
    pub const DEFAULT: Self = Self {
        variant: ChoiceVariant::Plain,
        size: Size::Md,
        color: Color::Primary,
        sizes: Sizes::new("14px", "16px", "18px", "20px", "22px", "24px"),
    };

    fn size_sx(size: Size) -> Sx {
        sx().var(RADIO_CIRCLE, RADIO_CIRCLE_SIZE.value(size))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx)
    }
}

impl ToCssDeclarations for RadioDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        Size::ALL
            .into_iter()
            .map(|size| RADIO_CIRCLE_SIZE.declare(size, self.sizes.get(size)))
            .collect()
    }
}
