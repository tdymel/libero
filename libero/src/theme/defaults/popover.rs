use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::str_enum::str_enum;
use crate::theme::CssVar;

pub const POPOVER_GAP: CssVar = CssVar::new("--lsx-popover-gap");
pub const POPOVER_PADDING: CssVar = CssVar::new("--lsx-popover-padding");

str_enum! {
    /// Which side of the anchor the floating box sits on. `Start`/`End` are
    /// logical: `Start` is the left under `dir="ltr"`, the right under `rtl`.
    ///
    /// ```rust
    /// use libero::theme::Side;
    /// assert_eq!(Side::Start.as_str(), "start");
    /// ```
    #[state_prefix = "side"]
    pub enum Side {
        Top = "top",
        #[default]
        Bottom = "bottom",
        Start = "start",
        End = "end",
    }
}

impl Side {
    pub(crate) fn opposite(self) -> Self {
        match self {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Start => Side::End,
            Side::End => Side::Start,
        }
    }

    /// Top and bottom stack along `y`, so their cross axis is `x`.
    pub(crate) fn is_vertical(self) -> bool {
        matches!(self, Side::Top | Side::Bottom)
    }
}

str_enum! {
    /// Where the floating box lines up along the side's cross axis. Across a
    /// `Top`/`Bottom` side `Start` follows the direction, as `Side::Start` does.
    #[state_prefix = "align"]
    pub enum Align {
        #[default]
        Start = "start",
        Center = "center",
        End = "end",
    }
}

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
