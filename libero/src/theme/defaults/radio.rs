use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{ChoiceVariant, CssVar, Size, SizeCss, Sizes};

pub const RADIO_CIRCLE_SIZE: SizeCss = SizeCss::new("--lsx-radio-circle-size-");

/// The picked level, resolved on the control so the circle and the dot inside
/// it - which carry no `data-state` of their own - can inherit it.
pub const RADIO_CIRCLE: CssVar = CssVar::new("--lsx-radio-circle");

/// What `Radio` does not share with every other field. No `radius`: a radio is
/// a circle at every size, which is the one thing that tells it apart from a
/// `Checkbox` at a glance.
///
/// The circle uses the same scale as the checkbox box, so a form mixing the
/// two lines up by construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadioDefaults {
    /// The wrapper a radio, and every radio in a `RadioGroup`, takes when a
    /// call site names none.
    pub variant: ChoiceVariant,
    pub size: Size,
    pub sizes: Sizes<&'static str>,
}

impl RadioDefaults {
    pub const DEFAULT: Self = Self {
        variant: ChoiceVariant::Plain,
        size: Size::Md,
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
