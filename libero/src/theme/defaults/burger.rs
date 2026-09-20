use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

/// The glyph's width per step; the button around it is one spacing step larger.
pub const BURGER_SIZES: SizeCss = SizeCss::new("--lsx-burger-size-");

/// The active glyph width; a caller's `size` writes the `-override` twin.
pub const BURGER_SIZE: CssVar = CssVar::new("--lsx-burger-size");
/// Never declared by the theme: bars read `var(--lsx-burger-color, currentColor)`,
/// so `sx().color(..)` and the `color` prop both work.
pub const BURGER_COLOR: CssVar = CssVar::new("--lsx-burger-color");
/// Bar thickness, `size / 12`, declared on the glyph: a `:root` derivation would
/// only see `:root`'s unset size override.
pub const BURGER_LINE_SIZE: CssVar = CssVar::new("--lsx-burger-line-size");
pub const BURGER_TRANSITION_DURATION: CssVar = CssVar::new("--lsx-burger-transition-duration");
pub const BURGER_TRANSITION_TIMING: CssVar = CssVar::new("--lsx-burger-transition-timing");

/// Theme defaults for `Burger`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurgerDefaults {
    /// The glyph's width and height, not the button's.
    pub size: Size,
    /// The glyph's size per step, in px.
    pub sizes: Sizes<u16>,
    /// Theme-only, no prop; a one-off goes through `sx`.
    pub transition_duration: &'static str,
    pub transition_timing: &'static str,
}

impl BurgerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        sizes: Sizes::new(12, 18, 24, 34, 42, 52),
        transition_duration: "300ms",
        transition_timing: "ease",
    };
}

impl ToCssDeclarations for BurgerDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        let mut declarations = self.sizes.to_css_declarations(BURGER_SIZES, "px");
        declarations.push(BURGER_SIZE.declare(BURGER_SIZES.value(self.size)));
        declarations.push(BURGER_TRANSITION_DURATION.declare(self.transition_duration));
        declarations.push(BURGER_TRANSITION_TIMING.declare(self.transition_timing));
        declarations
    }
}
