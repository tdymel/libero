use dioxus::prelude::*;

use crate::{
    hooks::{LocalState, use_local_state},
    sx::{REDUCED_MOTION, Sx, sx},
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

/// A pseudo-element on the component, not a child node, which costs ~1,000 ns
/// per render even with no ripple showing.
pub(crate) fn ripple_sx(base: Sx) -> Sx {
    base.position("relative")
        .overflow("hidden")
        .selector("::after", ripple_circle_sx())
        .and(ripple_states_at("::after", RIPPLE_ANIMATION))
}

/// [`ripple_sx`] on an `aria-hidden` `span[data-ripple]` child, so the host keeps no
/// `overflow: hidden` and a `::before` may reach past it as a hit area. A transform
/// composites; the old `clip-path` circle repainted every frame (todo 2021).
pub(crate) fn child_ripple_sx(base: Sx) -> Sx {
    base.position("relative")
        .selector(
            "& > [data-ripple]",
            sx().position("absolute")
                .inset("0")
                .overflow("hidden")
                .border_radius("inherit")
                .pointer_events("none"),
        )
        .selector("& > [data-ripple]::after", ripple_circle_sx())
        .and(ripple_states_at(
            "& > [data-ripple]::after",
            RIPPLE_ANIMATION,
        ))
}

fn ripple_circle_sx() -> Sx {
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
        .pointer_events("none")
}

fn ripple_states_at(at: &str, names: [&str; 2]) -> Sx {
    sx().when(
        RIPPLE_STATE[0],
        sx().selector(at, ripple_animation_sx(names[0])),
    )
    .when(
        RIPPLE_STATE[1],
        sx().selector(at, ripple_animation_sx(names[1])),
    )
}

/// No ripple at all under reduced motion: it is decoration, the click does
/// the same without it.
fn ripple_animation_sx(name: &str) -> Sx {
    sx().animation(format!("{name} 550ms ease-out forwards"))
        .media(REDUCED_MOTION, sx().animation("none"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// Both animation names have to stop, or the second click replays one.
    #[test]
    fn reduced_motion_switches_both_ripples_off() {
        for (base, after) in [
            (ripple_sx(sx()), "::after"),
            (child_ripple_sx(sx()), " > [data-ripple]::after"),
        ] {
            assert_both_stop(&Stylesheet::from(&base), after);
        }
    }

    /// Only the ripple's span clips, so a `::before` hit area reaches past the host.
    #[test]
    fn the_child_ripple_leaves_the_host_unclipped() {
        let css = Stylesheet::from(&child_ripple_sx(sx()));
        let css = css.as_str();
        let span = css.find(" > [data-ripple]{").expect("the span's rule");
        let overflow = css.find("overflow:hidden").expect("a clip");
        assert!(
            span < overflow && css.matches("overflow").count() == 1,
            "{css}"
        );
        assert!(!css.contains("clip-path"), "{css}");
    }

    fn assert_both_stop(css: &Stylesheet, after: &str) {
        let css = css.as_str();
        let reduced = css.find(REDUCED_MOTION).expect("a reduced-motion block");

        for (state, name) in RIPPLE_STATE.iter().zip(RIPPLE_ANIMATION) {
            let running = css.find(name).expect("the ripple animation");
            let stopped = css
                .find(&format!(
                    "[data-state~=\"{state}\"]{after}{{animation:none;}}"
                ))
                .unwrap_or_else(|| panic!("{state} never stops: {css}"));
            assert!(reduced < stopped && running < stopped, "{css}");
        }
    }
}
