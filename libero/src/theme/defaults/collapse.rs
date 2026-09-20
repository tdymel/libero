use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::CssVar;

/// The exit/entry duration; the `duration` prop sets the `-override` twin.
pub const COLLAPSE_DURATION: CssVar = CssVar::new("--lsx-collapse-duration");
pub const COLLAPSE_EASING: CssVar = CssVar::new("--lsx-collapse-easing");
/// The closed panel's opacity: `0` when the theme fades, `1` when not.
pub const COLLAPSE_OPACITY_CLOSED: CssVar = CssVar::new("--lsx-collapse-opacity-closed");

/// Theme defaults for `Collapse`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollapseDefaults {
    /// Milliseconds. `0` fires no `transitionend`, so `Collapse` unmounts straight from `open`.
    pub duration: u32,
    pub easing: &'static str,
    /// Whether a closing panel fades as well as shrinks; theme-only, no prop.
    pub animate_opacity: bool,
}

impl CollapseDefaults {
    pub const DEFAULT: Self = Self {
        duration: 200,
        easing: "ease",
        animate_opacity: true,
    };
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
