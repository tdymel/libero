use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::CssVar;

/// The exit/entry duration, as a CSS time. The per-instance `duration` prop
/// sets the `-override` twin.
pub const COLLAPSE_DURATION: CssVar = CssVar::new("--lsx-collapse-duration");
pub const COLLAPSE_EASING: CssVar = CssVar::new("--lsx-collapse-easing");
/// The closed panel's opacity - `0` when the theme animates opacity, `1` when
/// it does not. A var rather than a condition, so turning the fade off is one
/// declaration on `:root` instead of a second class.
pub const COLLAPSE_OPACITY_CLOSED: CssVar = CssVar::new("--lsx-collapse-opacity-closed");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollapseDefaults {
    /// Milliseconds. `0` disables the animation - and with it the
    /// `transitionend` a `keep_mounted: false` panel unmounts on, which
    /// `Collapse` handles by unmounting straight from `open`.
    pub duration: u32,
    pub easing: &'static str,
    /// Whether a closing panel fades as well as shrinking. One app-wide design
    /// decision, so a theme field and not a prop.
    pub animate_opacity: bool,
}

impl ToCssDeclarations for CollapseDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            COLLAPSE_DURATION.declare(format!("{}ms", self.duration)),
            COLLAPSE_EASING.declare(self.easing),
            COLLAPSE_OPACITY_CLOSED.declare(if self.animate_opacity { "0" } else { "1" }),
        ]
    }
}
