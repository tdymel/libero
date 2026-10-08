use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{ChoiceVariant, CssVar, Size, SizeCss, Sizes};
use crate::tokens::Color;

pub const CHECKBOX_BOX_SIZE: SizeCss = SizeCss::new("--lsx-checkbox-box-size-");

/// The picked level, resolved on the root so the box and its mark inherit it.
pub const CHECKBOX_BOX: CssVar = CssVar::new("--lsx-checkbox-box");
pub const CHECKBOX_RADIUS: CssVar = CssVar::new("--lsx-checkbox-radius");

/// Theme defaults for `Checkbox`, set on [`Theme`](crate::theme::Theme).
///
/// Label and caption typography come from `FieldDefaults`; the box has its own scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckboxDefaults {
    /// The wrapper a checkbox takes when a call site names none.
    pub variant: ChoiceVariant,
    pub size: Size,
    pub radius: Size,
    /// The colour a checked box takes when a call site names none.
    pub color: Color,
    pub sizes: Sizes<&'static str>,
}

impl CheckboxDefaults {
    pub const DEFAULT: Self = Self {
        variant: ChoiceVariant::Plain,
        size: Size::Md,
        radius: Size::Sm,
        color: Color::Primary,
        sizes: Sizes::new("14px", "16px", "18px", "20px", "22px", "24px"),
    };

    fn size_sx(size: Size) -> Sx {
        sx().var(CHECKBOX_BOX, CHECKBOX_BOX_SIZE.value(size))
    }

    fn radius_sx(radius: Size) -> Sx {
        sx().var(CHECKBOX_RADIUS, SizeCss::RADIUS.value(radius))
    }

    pub fn theme_vars() -> Sx {
        sx().per_size(Self::size_sx).per_radius(Self::radius_sx)
    }
}

impl ToCssDeclarations for CheckboxDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        Size::ALL
            .into_iter()
            .map(|size| CHECKBOX_BOX_SIZE.declare(size, self.sizes.get(size)))
            .collect()
    }
}
