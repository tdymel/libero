use dioxus::prelude::*;

use crate::{
    hooks::{LocalState, use_local_state},
    sx::{Sx, sx},
    theme::{CssVar, RIPPLE_ANIMATION, RIPPLE_STATE},
};

const RIPPLE_X_VAR: CssVar = CssVar::new("--lsx-ripple-x");
const RIPPLE_Y_VAR: CssVar = CssVar::new("--lsx-ripple-y");

/// The ripple currently showing: where it started, and which of the two
/// animation names runs it - see [`RIPPLE_ANIMATION`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Ripple {
    which: usize,
    x: f64,
    y: f64,
}

impl Ripple {
    /// The `data-state` that runs this one.
    pub(crate) fn state(&self) -> &'static str {
        RIPPLE_STATE[self.which]
    }

    /// The click point, appended to the already-rendered variables. Only a
    /// component that has been clicked pays for this.
    pub(crate) fn with_point(&self, mut style: String) -> String {
        for (name, value) in [(RIPPLE_X_VAR, self.x), (RIPPLE_Y_VAR, self.y)] {
            style.push_str(name.name());
            style.push(':');
            style.push_str(&value.to_string());
            style.push_str("px;");
        }
        style
    }
}

/// One ripple at a time; a new click overrides the last.
pub(crate) fn use_ripple() -> RippleState {
    RippleState(use_local_state(|| None))
}

/// `Clone`, not `Copy` - a component that both reads it and moves it into a
/// click handler reads first.
#[derive(Clone)]
pub(crate) struct RippleState(LocalState<Option<Ripple>>);

impl RippleState {
    pub(crate) fn showing(&self) -> Option<Ripple> {
        self.0.get()
    }

    /// `which` alternates so the animation name changes and the browser
    /// replays it.
    pub(crate) fn press(&self, event: &Event<MouseData>) {
        let point = event.element_coordinates();
        let which = self.0.get().map_or(0, |last| 1 - last.which);
        self.0.set(Some(Ripple {
            which,
            x: point.x,
            y: point.y,
        }));
    }
}

/// A pseudo-element on the component, not a child node. A rendered child
/// costs ~1,000 ns per component per render even while no ripple is showing -
/// see the render-cost notes.
pub(crate) fn ripple_sx(base: Sx) -> Sx {
    base.position("relative")
        .overflow("hidden")
        .selector(
            "::after",
            sx().content("\"\"")
                .position("absolute")
                .left(RIPPLE_X_VAR.value_or("50%"))
                .top(RIPPLE_Y_VAR.value_or("50%"))
                .width("300%")
                .height("300%")
                .border_radius("50%")
                .background("currentColor")
                .opacity("0")
                .transform("translate(-50%, -50%) scale(0)")
                .pointer_events("none"),
        )
        .when(
            RIPPLE_STATE[0],
            sx().selector("::after", ripple_animation_sx(0)),
        )
        .when(
            RIPPLE_STATE[1],
            sx().selector("::after", ripple_animation_sx(1)),
        )
}

fn ripple_animation_sx(which: usize) -> Sx {
    sx().animation(format!(
        "{} 550ms ease-out forwards",
        RIPPLE_ANIMATION[which]
    ))
}
