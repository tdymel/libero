use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

pub const CHECKBOX_BOX_SIZE: SizeCss = SizeCss::new("--lsx-checkbox-box-size-");

/// The picked level, resolved on the root so the box and the mark inside it -
/// which carry no `data-state` of their own - can inherit it.
pub const CHECKBOX_BOX: CssVar = CssVar::new("--lsx-checkbox-box");
pub const CHECKBOX_RADIUS: CssVar = CssVar::new("--lsx-checkbox-radius");

/// What `Checkbox` does not share with every other field. The label and
/// caption typography come from `FieldDefaults`, so a checkbox and a
/// `TextField` in one form read at the same scale by construction.
///
/// The box has a scale of its own rather than the field's content box: it is
/// not an affordance inside a frame, it *is* the control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckboxDefaults {
    pub size: Size,
    pub radius: Size,
    pub sizes: Sizes<&'static str>,
}

impl CheckboxDefaults {
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
