use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, States, Variables,
        common::{base_props, variables},
        layout::use_box,
    },
    hooks::{use_presence, use_theme},
    sx::{StaticSx, sx},
    theme::{COLLAPSE_DURATION, COLLAPSE_EASING, COLLAPSE_OPACITY_CLOSED},
};

/// The property the exit transition is measured on, and the one
/// [`use_presence`] filters `transitionend` by. The content's own `opacity`
/// bubbles a second event to this same root; it shares `COLLAPSE_DURATION`, so
/// the two finish together rather than one arriving early, but the filter is
/// what keeps either from standing in for the other.
///
/// The filter discriminates on the property name, **not on the target**, so a
/// nested `Collapse` inside this one's children bubbles a matching
/// `grid-template-rows` event to this root. If both are closing and the inner
/// one is faster, this one unmounts its children at the inner one's end time.
/// Unfixable here - `TransitionData` exposes no target - and only reachable
/// with `keep_mounted: false`. See todo 41.
const EXIT_PROPERTY: &str = "grid-template-rows";

const REDUCED_MOTION: &str = "(prefers-reduced-motion: reduce)";

/// `0fr` -> `1fr` rather than a measured pixel height: it needs no platform
/// capability, so web, Blitz, the WebView floor and SSR all behave alike, and
/// it re-animates for free when the content's own height changes - which a
/// measured height cannot without a resize observer we do not have.
///
/// **The reduced-motion guard lives wherever the transition lives.** Here each
/// `transition` is declared inside the `when(..)` block whose values it
/// animates, so each guard is nested inside that same block. A `media`
/// modifier adds no specificity, so a guard hoisted out beside the conditions
/// would be 0-1-0 against their 0-2-0 and the motion would still play. If a
/// transition is ever moved to the base level, its guard moves with it - the
/// invariant is the pairing, not the nesting. See `docs/public/md/styling.md`,
/// and `libero/tests/collapse.rs`, which asserts the emitted CSS text rather
/// than trusting the builder.
static COLLAPSE_BASE_SX: StaticSx = StaticSx::new(|| {
    let transition = format!(
        "{EXIT_PROPERTY} {} {}",
        COLLAPSE_DURATION.overridable(),
        COLLAPSE_EASING.value()
    );

    sx().display("grid")
        .when(
            "open",
            sx().grid_template_rows("1fr")
                .transition(transition.clone())
                .media(REDUCED_MOTION, sx().transition("none")),
        )
        .when(
            "closed",
            sx().grid_template_rows("0fr")
                .transition(transition)
                .media(REDUCED_MOTION, sx().transition("none")),
        )
});

/// `min-height: 0` lets the grid row actually reach zero, and `overflow:
/// hidden` is load-bearing twice: it clips during the transition, and it stops
/// the content's own margins collapsing out of the row and holding it open.
///
/// `visibility` is how closed content stops being a tab stop without any JS. A
/// zero-length `visibility` transition *delayed by the duration* keeps the
/// panel visible and announced for the whole close and drops it out of focus
/// order and the accessibility tree the instant the animation ends. No
/// `aria-hidden`: `visibility: hidden` already removes the subtree from the
/// accessibility tree, and it does it in step with the animation, where a
/// static `aria-hidden` would hide content that is still on screen and may
/// still hold focus. `inert` is not used either - it is unimplemented off the
/// web, `visibility` is portable.
static COLLAPSE_CONTENT_SX: StaticSx = StaticSx::new(|| {
    let duration = COLLAPSE_DURATION.overridable();
    let easing = COLLAPSE_EASING.value();

    sx().min_height("0")
        .overflow("hidden")
        .when(
            "open",
            sx().opacity("1")
                .visibility("visible")
                .transition(format!(
                    "opacity {duration} {easing}, visibility 0s linear 0s"
                ))
                .media(REDUCED_MOTION, sx().transition("none")),
        )
        .when(
            "closed",
            sx().opacity(COLLAPSE_OPACITY_CLOSED.value())
                .visibility("hidden")
                // Under reduced motion there is no animation to outlast, so
                // the delay goes with the transition and the content leaves
                // the accessibility tree at once.
                .transition(format!(
                    "opacity {duration} {easing}, visibility 0s linear {duration}"
                ))
                .media(REDUCED_MOTION, sx().transition("none")),
        )
});

/// Only when the caller set one: an absent `duration` has to leave the theme's
/// var alone rather than pinning it to the value it happens to hold.
///
/// Custom properties inherit, so setting it on the root is what reaches the
/// content element too.
fn collapse_variables(duration: Option<u32>) -> Variables {
    variables().with(
        COLLAPSE_DURATION.override_var(),
        duration.map(|ms| format!("{ms}ms")),
    )
}

base_props! {
    pub struct CollapseProps {
        /// Whether the panel is expanded. Strictly controlled - `Collapse`
        /// holds no open state of its own, like `Tabs::value`.
        open: bool,
        /// Keep `children` in the DOM while closed (the default). `false`
        /// unmounts them when the exit transition ends, which discards
        /// whatever state they held - a half-typed form does not survive being
        /// collapsed.
        ///
        /// The **root is always in the DOM** either way, so a trigger's
        /// `aria-controls` always resolves and a caller may put `id`,
        /// `role="region"` and `aria-labelledby` on it through `attributes`.
        #[props(default)]
        keep_mounted: Option<bool>,
        /// Milliseconds, defaulting to `theme.collapse.duration`. `0` disables
        /// the animation.
        #[props(default)]
        duration: Option<u32>,
        children: Element,
    }
}

/// Animates its children's height open and closed.
///
/// Renders no role and no ARIA of its own: the disclosure semantics belong to
/// whatever owns the trigger, and `Collapse` never sees it. It handles no keys
/// either, because it focuses nothing.
///
/// ```ignore
/// let mut open = use_signal(|| false);
/// rsx! {
///     Button { onclick: move |_| open.toggle(), "Details" }
///     Collapse { open: open(), Text { "Shipping is calculated at checkout." } }
/// }
/// ```
#[component]
pub fn Collapse(props: CollapseProps) -> Element {
    let theme = use_theme();
    let duration = props.duration.unwrap_or(theme.collapse.duration);
    let keep_mounted = props.keep_mounted.unwrap_or(true);

    // Unconditional, as every hook must be: the `keep_mounted: true` path
    // ignores `mounted()` but still reads `visible()`, which is what makes the
    // grid row animate from `0fr` instead of snapping.
    let presence = use_presence(props.open, EXIT_PROPERTY);
    let visible = presence.visible();
    let mounted = keep_mounted
        || match duration {
            // A zero-duration transition never runs, so no `transitionend`
            // ever arrives and the hook would hold the content mounted for
            // good. Nothing is animating, so `open` is the whole answer.
            0 => props.open,
            _ => presence.mounted(),
        };

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("open", visible)
        .with("closed", !visible)
        .into();
    let variables: Input<Variables> = collapse_variables(props.duration).into();

    // Its own states, not the caller's: the caller's tokens belong to the
    // element they were set on.
    let content_states: Input<States> = States::default()
        .with("open", visible)
        .with("closed", !visible)
        .into();
    let content = use_box()
        .framework_sx(&COLLAPSE_CONTENT_SX)
        .states(&content_states)
        .prepare()
        .render(HtmlTag::Div, Vec::new(), mounted.then_some(props.children));

    use_box()
        .framework_sx(&COLLAPSE_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .event("onmounted", move |_: Event<MountedData>| {
            presence.on_mounted()
        })
        .event("ontransitionend", move |event: Event<TransitionData>| {
            presence.on_transition_end(&event)
        })
        .render(HtmlTag::Div, props.attributes, content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_duration_sets_the_override_variable() {
        assert_eq!(
            collapse_variables(Some(350)).to_string(),
            format!("{}:350ms;", COLLAPSE_DURATION.override_var().name())
        );
    }

    /// Including `0`, which is a real value and not "unset".
    #[test]
    fn a_zero_duration_still_sets_the_variable() {
        assert_eq!(
            collapse_variables(Some(0)).to_string(),
            format!("{}:0ms;", COLLAPSE_DURATION.override_var().name())
        );
    }

    #[test]
    fn no_duration_emits_no_variable() {
        assert_eq!(collapse_variables(None).to_string(), "");
    }
}
