use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const Z_INDEX_HEADER: CssVar = CssVar::new("--lsx-z-index-header");
pub const Z_INDEX_FLOAT: CssVar = CssVar::new("--lsx-z-index-float");
pub const Z_INDEX_WINDOW: CssVar = CssVar::new("--lsx-z-index-window");
pub const Z_INDEX_OVERLAY: CssVar = CssVar::new("--lsx-z-index-overlay");
pub const Z_INDEX_MODAL: CssVar = CssVar::new("--lsx-z-index-modal");
pub const Z_INDEX_POPOVER: CssVar = CssVar::new("--lsx-z-index-popover");
pub const Z_INDEX_NOTIFICATION: CssVar = CssVar::new("--lsx-z-index-notification");

/// The library's stacking order in one place, so two components can't tie.
/// Gaps of 100 leave room to slot a layer in without renumbering.
///
/// Plain integers because `ModalHost` computes `modal + n * modal_step` in
/// Rust; the vars are published too, for a caller's own `sx`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZIndexDefaults {
    pub header: i32,
    pub float: i32,
    /// The lowest floating window; each window raised above it adds
    /// `window_step`, capped below `overlay`.
    pub window: i32,
    pub window_step: i32,
    pub overlay: i32,
    /// The first modal's z-index; each one above adds `modal_step`.
    pub modal: i32,
    pub modal_step: i32,
    /// Above every modal, not below: an anchored popover is portaled to the
    /// document root, so a dropdown opened *inside* a modal no longer inherits
    /// its stacking context and would otherwise fall behind it.
    pub popover: i32,
    /// Above everything: a notification raised while a modal is open, or
    /// while a dropdown in it is, still has to be read.
    pub notification: i32,
}

impl ZIndexDefaults {
    pub const DEFAULT: Self = Self {
        header: 100,
        float: 200,
        window: 250,
        window_step: 1,
        overlay: 300,
        modal: 1000,
        modal_step: 10,
        popover: 2000,
        notification: 2100,
    };
}

impl ToCssDeclarations for ZIndexDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            Z_INDEX_HEADER.declare(self.header.to_string()),
            Z_INDEX_FLOAT.declare(self.float.to_string()),
            Z_INDEX_WINDOW.declare(self.window.to_string()),
            Z_INDEX_OVERLAY.declare(self.overlay.to_string()),
            Z_INDEX_MODAL.declare(self.modal.to_string()),
            Z_INDEX_POPOVER.declare(self.popover.to_string()),
            Z_INDEX_NOTIFICATION.declare(self.notification.to_string()),
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
            z.float < z.window,
            "a floating window must clear the dropdowns on the page"
        );
        assert!(
            z.window < z.overlay,
            "a dimming overlay, and the modal above it, must cover a window"
        );
        assert!(z.overlay < z.modal, "a modal must sit above a bare overlay");
        assert!(
            z.modal < z.popover,
            "a dropdown opened inside a modal must clear it"
        );
        assert!(
            z.popover < z.notification,
            "a notification must clear an open modal and its dropdowns"
        );
    }
}
