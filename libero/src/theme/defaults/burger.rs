use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, Size, SizeCss, Sizes};

/// The glyph's width at each step - the button around it is one spacing step
/// larger, so the tap target is bigger than the bars.
pub const BURGER_SIZES: SizeCss = SizeCss::new("--lsx-burger-size-");

/// The active glyph width. `ActionIcon`'s shape: the theme picks one step off
/// the scale above, a caller's `size` writes the `-override` twin, so nothing
/// here needs a per-size rule.
pub const BURGER_SIZE: CssVar = CssVar::new("--lsx-burger-size");
/// **Deliberately never declared by the theme.** The bars read
/// `var(--lsx-burger-color, currentColor)`, so a caller who styles the button
/// with `sx().color("white")` keeps working, a caller who passes `color:`
/// sets this var, and neither silently wins over the other.
pub const BURGER_COLOR: CssVar = CssVar::new("--lsx-burger-color");
/// Bar thickness, `size / 12`. Declared by the component on the glyph itself,
/// not here: a custom property is substituted at computed-value time on the
/// element it is *declared* on, so a `:root` derivation would only ever see
/// `:root`'s (unset) size override.
pub const BURGER_LINE_SIZE: CssVar = CssVar::new("--lsx-burger-line-size");
pub const BURGER_TRANSITION_DURATION: CssVar = CssVar::new("--lsx-burger-transition-duration");
pub const BURGER_TRANSITION_TIMING: CssVar = CssVar::new("--lsx-burger-transition-timing");

/// The two words a burger announces itself with: one place to translate.
/// Split out of [`BurgerDefaults`] on purpose - the geometry beside them is
/// not locale, and a translation should be able to replace the labels without
/// restating a pixel scale. The [`DateDefaults`](crate::theme::DateDefaults)
/// precedent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurgerLabels {
    /// Names the button while the panel is closed.
    pub open: &'static str,
    /// Names the button while the panel is open.
    pub close: &'static str,
}

impl BurgerLabels {
    pub const ENGLISH: Self = Self {
        open: "Open navigation",
        close: "Close navigation",
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BurgerDefaults {
    /// The glyph's width and height, not the button's.
    pub size: Size,
    /// `xs`..`xl` are Mantine's five, adopted exactly. `xxl` is ours,
    /// continuing the ramp past the widest step Mantine offers.
    pub sizes: Sizes<u16>,
    /// Motion is a theme decision, not a per-call-site one - Mantine's
    /// `transitionDuration`/`transitionTimingFunction` props are declined.
    /// An off-scale one-off goes through `sx`.
    pub transition_duration: &'static str,
    pub transition_timing: &'static str,
    pub labels: BurgerLabels,
}

impl BurgerDefaults {
    pub const DEFAULT: Self = Self {
        size: Size::Md,
        // `xs`..`xl` are Mantine's own five, adopted exactly. `xxl`
        // continues the ramp past the widest step it offers.
        sizes: Sizes::new(12, 18, 24, 34, 42, 52),
        transition_duration: "300ms",
        transition_timing: "ease",
        labels: BurgerLabels::ENGLISH,
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
