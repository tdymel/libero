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

/// Theme defaults for `Popover`, set on [`Theme`](crate::theme::Theme).
/// Pixels, not a `Size`: `use_popover` does arithmetic with measured rects.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopoverDefaults {
    /// The `side` and `align` a caller of `use_popover` starts from, as
    /// `PopoverOptions { side: theme.popover.side, ..PopoverOptions::new(gap, padding) }`.
    /// `PopoverOptions::new` and the built-in dropdowns keep `Bottom` and `Start`.
    pub side: Side,
    pub align: Align,
    /// Distance from the anchor.
    pub gap: f64,
    /// How close to a viewport edge the box may come before it flips or shifts.
    pub padding: f64,
}

impl PopoverDefaults {
    pub const DEFAULT: Self = Self {
        side: Side::Bottom,
        align: Align::Start,
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
