use crate::css::{CssDeclaration, ToCssDeclarations};

use crate::theme::CssVar;

/// The enter/exit duration; the `duration` prop sets the `-override` twin.
pub const TRANSITION_DURATION: CssVar = CssVar::new("--lsx-transition-duration");
pub const TRANSITION_EASING: CssVar = CssVar::new("--lsx-transition-easing");
/// How far the `Fade*` kinds travel.
pub const TRANSITION_DISTANCE: CssVar = CssVar::new("--lsx-transition-distance");
/// The size the `Scale` kind grows from.
pub const TRANSITION_SCALE: CssVar = CssVar::new("--lsx-transition-scale");
/// The size the `Pop` kind grows from.
pub const TRANSITION_POP_SCALE: CssVar = CssVar::new("--lsx-transition-pop-scale");

/// Theme defaults for `Transition`, set on [`Theme`](crate::theme::Theme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransitionDefaults {
    /// Milliseconds. `0` fires no `transitionend`, so `Transition` unmounts straight from `open`.
    pub duration: u32,
    pub easing: &'static str,
    /// A CSS length.
    pub distance: &'static str,
    /// A unitless factor below `1`.
    pub scale: &'static str,
    /// A unitless factor below `scale`.
    pub pop_scale: &'static str,
}

impl TransitionDefaults {
    pub const DEFAULT: Self = Self {
        duration: 200,
        easing: "ease",
        distance: "1rem",
        scale: "0.9",
        pop_scale: "0.8",
    };
}

impl ToCssDeclarations for TransitionDefaults {
    fn to_css_declarations(&self) -> Vec<CssDeclaration> {
        vec![
            TRANSITION_DURATION.declare(format!("{}ms", self.duration)),
            TRANSITION_EASING.declare(self.easing),
            TRANSITION_DISTANCE.declare(self.distance),
            TRANSITION_SCALE.declare(self.scale),
            TRANSITION_POP_SCALE.declare(self.pop_scale),
        ]
    }
}
