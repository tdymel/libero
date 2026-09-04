use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::CssVar;

pub const POPOVER_GAP: CssVar = CssVar::new("--lsx-popover-gap");
pub const POPOVER_PADDING: CssVar = CssVar::new("--lsx-popover-padding");

/// What every popover is placed by: how far it sits off its anchor, and how
/// close to a viewport edge it may come before it flips or shifts.
///
/// Pixels, not a `Size`: `use_popover` does arithmetic with them against
/// measured rects, so a CSS length would have to be resolved back to a number.
/// The vars are published anyway, for a caller's own `sx`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopoverDefaults {
    pub gap: f64,
    pub padding: f64,
}

impl PopoverDefaults {
    pub const DEFAULT: Self = Self {
        gap: 4.0,
        padding: 8.0,
    };
}

impl ToCssDeclarations for PopoverDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            POPOVER_GAP.declare(format!("{}px", self.gap)),
            POPOVER_PADDING.declare(format!("{}px", self.padding)),
        ]
    }
}
