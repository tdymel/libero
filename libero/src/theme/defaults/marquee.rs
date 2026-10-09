use crate::css::{CssDeclaration, ToCssDeclarations};
use crate::theme::{CssVar, Size};

/// One full cycle, `{n}ms`; a `duration` prop writes the `-override` twin.
pub const MARQUEE_DURATION: CssVar = CssVar::new("--lsx-marquee-duration");
/// How far the fade reaches in from each end.
pub const MARQUEE_FADE_SIZE: CssVar = CssVar::new("--lsx-marquee-fade-size");

// Per instance, from props; the shift reads both.
pub const MARQUEE_REPEAT: CssVar = CssVar::new("--lsx-marquee-repeat");
pub const MARQUEE_GAP: CssVar = CssVar::new("--lsx-marquee-gap");
/// The `transform` a cycle ends on, set per orientation, so one keyframe serves both axes.
pub const MARQUEE_SHIFT: CssVar = CssVar::new("--lsx-marquee-shift");

/// Ends one copy plus one gap along, where the next copy took the first's place:
/// the restart is invisible.
pub const MARQUEE_KEYFRAMES: &str =
    "@keyframes lsx-marquee{to{transform:var(--lsx-marquee-shift);}}";
pub const MARQUEE_ANIMATION: &str = "lsx-marquee";

/// Below two copies there is nothing to restart onto, and the loop jumps.
pub const MARQUEE_MIN_REPEAT: u8 = 2;

/// Theme defaults for `Marquee`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarqueeDefaults {
    /// Milliseconds per cycle. A duration, not a speed: a longer strip moves faster.
    pub duration: u32,
    /// Copies laid in a row. Too few leave a gap crossing the view; a debug build warns.
    pub repeat: u8,
    /// Between copies, and between the last and the first.
    pub gap: Size,
    pub pause_on_hover: bool,
    /// The built-in pause toggle, on for WCAG 2.2.2 (motion over five seconds).
    pub pause_control: bool,
    /// Off: the fade is drawn in `Paper`'s surface colour, wrong on any other.
    pub fade_edges: bool,
    /// Any CSS length or percentage of the marquee's length.
    pub fade_size: &'static str,
}

impl MarqueeDefaults {
    pub const DEFAULT: Self = Self {
        duration: 40_000,
        repeat: 4,
        gap: Size::Md,
        pause_on_hover: false,
        pause_control: true,
        fade_edges: false,
        fade_size: "5%",
    };
}

impl ToCssDeclarations for MarqueeDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            MARQUEE_DURATION.declare(format!("{}ms", self.duration)),
            MARQUEE_FADE_SIZE.declare(self.fade_size),
        ]
    }
}
