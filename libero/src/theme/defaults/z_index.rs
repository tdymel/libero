use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const Z_INDEX_HEADER: CssVar = CssVar::new("--lsx-z-index-header");
pub const Z_INDEX_FLOAT: CssVar = CssVar::new("--lsx-z-index-float");
pub const Z_INDEX_OVERLAY: CssVar = CssVar::new("--lsx-z-index-overlay");
pub const Z_INDEX_MODAL: CssVar = CssVar::new("--lsx-z-index-modal");

/// The library's stacking order, in one place so two components can't tie.
/// Gaps of 100 leave room to slot a layer in without renumbering.
///
/// Plain integers rather than CSS-var-only values because `ModalHost` does
/// the `modal + n * modal_step` arithmetic in Rust; the vars are published
/// too, so a caller's own `sx` can sit on the same scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZIndexDefaults {
    pub header: i32,
    pub float: i32,
    pub overlay: i32,
    /// The first modal's z-index; each further modal opened on top of it
    /// gets `modal_step` more.
    pub modal: i32,
    pub modal_step: i32,
}

impl ToCssDeclarations for ZIndexDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            Z_INDEX_HEADER.declare(self.header.to_string()),
            Z_INDEX_FLOAT.declare(self.float.to_string()),
            Z_INDEX_OVERLAY.declare(self.overlay.to_string()),
            Z_INDEX_MODAL.declare(self.modal.to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::theme::Theme;

    #[test]
    fn the_scale_is_strictly_ordered() {
        let z = Theme::DEFAULT.z_index;
        assert!(z.header < z.float, "a dropdown must clear the header");
        assert!(
            z.float < z.overlay,
            "a dimming overlay must cover dropdowns"
        );
        assert!(z.overlay < z.modal, "a modal must sit above a bare overlay");
    }
}
