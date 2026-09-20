use std::time::Duration;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, base_props, variables},
        layout::use_box,
    },
    hooks::{use_presence, use_theme},
    sx::{REDUCED_MOTION, StaticSx, Sx, sx},
    theme::{COLLAPSE_DURATION, COLLAPSE_EASING, COLLAPSE_OPACITY_CLOSED},
};

/// The property `use_presence` filters `transitionend` by; a nested
/// `Collapse`'s matching event stops at its own root (todo 36d).
const EXIT_PROPERTY: &str = "grid-template-rows";

/// Each reduced-motion guard sits in its transition's `when(..)` block, or loses on specificity.
/// `visibility`, not `aria-hidden`/`inert`, drops a closed panel from focus and the a11y tree.
static COLLAPSE_BASE_SX: StaticSx = StaticSx::new(|| {
    let duration = COLLAPSE_DURATION.overridable();
    let rows = format!("{EXIT_PROPERTY} {duration} {}", COLLAPSE_EASING.value());

    sx().display("grid")
        .when(
            "open",
            sx().grid_template_rows("1fr")
                .visibility("visible")
                .transition(format!("{rows}, visibility 0s linear 0s"))
                .media(REDUCED_MOTION, sx().transition("none")),
        )
        .when(
            "closed",
            sx().grid_template_rows("0fr")
                .visibility("hidden")
                // Delayed so the panel stays announced until the close ends.
                .transition(format!("{rows}, visibility 0s linear {duration}"))
                .media(REDUCED_MOTION, sx().transition("none")),
        )
        .selector("& > div", content_sx())
});

/// `min-height: 0` lets the row reach zero; `overflow: hidden` clips and stops
/// the content's margins from holding the row open.
fn content_sx() -> Sx {
    let transition = format!(
        "opacity {} {}",
        COLLAPSE_DURATION.overridable(),
        COLLAPSE_EASING.value()
    );

    sx().min_height("0")
        .overflow("hidden")
        .when(
            "open",
            sx().opacity("1")
                .transition(transition.clone())
                .media(REDUCED_MOTION, sx().transition("none")),
        )
        .when(
            "closed",
            sx().opacity(COLLAPSE_OPACITY_CLOSED.value())
                .transition(transition)
                .media(REDUCED_MOTION, sx().transition("none")),
        )
}

/// Unset leaves the theme's var alone.
fn collapse_variables(duration: Option<u32>) -> Variables {
    variables().with(
        COLLAPSE_DURATION.override_var(),
        duration.map(|ms| format!("{ms}ms")),
    )
}

base_props! {
    pub struct CollapseProps {
        /// Whether the panel is expanded. Strictly controlled.
        open: bool,
        /// Keep `children` in the DOM while closed (the default); the root always stays.
        #[props(default)]
        keep_mounted: Option<bool>,
        /// Milliseconds, defaulting to `theme.collapse.duration`. `0` disables it.
        #[props(default)]
        duration: Option<u32>,
        children: Element,
    }
}

/// Animates its children's height open and closed.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Collapse, Text};
/// # fn app() -> Element {
/// let mut open = use_signal(|| false);
/// rsx! {
///     Button {
///         onclick: move |_| open.toggle(),
///         aria_expanded: open(),
///         aria_controls: "details",
///         "Details"
///     }
///     Collapse { id: "details", open: open(), Text { "Shipping is calculated at checkout." } }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/collapse>
#[component]
pub fn Collapse(props: CollapseProps) -> Element {
    let theme = use_theme();
    let duration = props.duration.unwrap_or(theme.collapse.duration);
    let keep_mounted = props.keep_mounted.unwrap_or(true);

    // Also with `keep_mounted`: `visible()` makes the row animate from `0fr`. The
    // duration is the fallback for an exit that never fires `transitionend`.
    let presence = use_presence(
        props.open,
        EXIT_PROPERTY,
        Some(Duration::from_millis(duration.into())),
    );
    let visible = presence.visible();
    let mounted = keep_mounted || presence.mounted();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("open", visible)
        .with("closed", !visible)
        .into();
    let variables: Input<Variables> = collapse_variables(props.duration).into();

    // Styled from the root's class, so a plain element, not a second `use_box`.
    let content_state = if visible { "open" } else { "closed" };
    let content = rsx! {
        div { "data-state": content_state,
            {mounted.then_some(props.children)}
        }
    };

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
