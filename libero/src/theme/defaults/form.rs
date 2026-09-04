use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::sx::{Sx, sx};
use crate::theme::CssVar;

pub const FORM_GAP: CssVar = CssVar::new("--lsx-form-gap");
pub const FIELDSET_GAP: CssVar = CssVar::new("--lsx-fieldset-gap");

/// What `Form` lays out: the space between its error summary and its fields.
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

/// What `Fieldset` lays out. The legend and caption typography is
/// `FieldDefaults`', so a group reads at the size of its fields.
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
