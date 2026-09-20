use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::CssVar;

pub const FORM_GAP: CssVar = CssVar::new("--lsx-form-gap");
pub const FIELDSET_GAP: CssVar = CssVar::new("--lsx-fieldset-gap");

/// Theme defaults for `Form`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormDefaults {
    /// Vertical gap between the summary and each child.
    pub gap: &'static str,
}

impl FormDefaults {
    pub const DEFAULT: Self = Self { gap: "16px" };

    pub fn theme_vars() -> Sx {
        sx().gap(FORM_GAP.value())
    }
}

impl ToCssDeclarations for FormDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![FORM_GAP.declare(self.gap)]
    }
}

/// Theme defaults for `Fieldset`, set on [`Theme`](crate::theme::Theme).
///
/// Legend and caption typography come from `FieldDefaults`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldsetDefaults {
    /// Vertical gap between the fields inside the group.
    pub gap: &'static str,
}

impl FieldsetDefaults {
    pub const DEFAULT: Self = Self { gap: "12px" };

    pub fn theme_vars() -> Sx {
        sx().gap(FIELDSET_GAP.value())
    }
}

impl ToCssDeclarations for FieldsetDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![FIELDSET_GAP.declare(self.gap)]
    }
}
