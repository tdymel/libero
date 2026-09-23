use std::time::Duration;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, base_props, variables},
        layout::use_box,
    },
    hooks::{use_presence, use_theme},
    sx::{REDUCED_MOTION, StaticSx, sx},
    theme::{
        CssVar, TRANSITION_DISTANCE, TRANSITION_DURATION, TRANSITION_EASING, TRANSITION_SCALE,
    },
};

/// The property `use_presence` filters `transitionend` by. Every kind fades,
/// so it is the one property all of them transition.
const EXIT_PROPERTY: &str = "opacity";

/// The closed `transform`, set per instance from the [`TransitionKind`].
const TRANSITION_FROM: CssVar = CssVar::new("--lsx-transition-from");

/// Each reduced-motion guard sits in its state's `when(..)` block, or loses on specificity.
/// `visibility`, not `aria-hidden`/`inert`, drops closed content from focus and the a11y tree.
static TRANSITION_BASE_SX: StaticSx = StaticSx::new(|| {
    let duration = TRANSITION_DURATION.overridable();
    let easing = TRANSITION_EASING.value();
    let motion = format!("opacity {duration} {easing}, transform {duration} {easing}");

    sx().when(
        "open",
        sx().opacity("1")
            .transform("none")
            .visibility("visible")
            .transition(format!("{motion}, visibility 0s linear 0s"))
            .media(REDUCED_MOTION, sx().transition("none")),
    )
    .when(
        "closed",
        sx().opacity("0")
            .transform(TRANSITION_FROM.value())
            .visibility("hidden")
            // Delayed so the content stays announced until the exit ends.
            .transition(format!("{motion}, visibility 0s linear {duration}"))
            .media(REDUCED_MOTION, sx().transition("none")),
    )
});

/// How `Transition` enters and exits; all of them also fade.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TransitionKind {
    /// Opacity only.
    #[default]
    Fade,
    /// Rises into place.
    FadeUp,
    /// Sinks into place.
    FadeDown,
    /// Grows from `theme.transition.scale`.
    Scale,
    /// Rises its own height into place.
    SlideUp,
    /// Sinks its own height into place.
    SlideDown,
    /// Moves left its own width into place (physical, not logical).
    SlideLeft,
    /// Moves right its own width into place (physical, not logical).
    SlideRight,
    /// Grows from a smaller size than `Scale`.
    Pop,
}

impl TransitionKind {
    /// The `transform` of the closed state.
    fn closed_transform(self) -> String {
        let distance = TRANSITION_DISTANCE.value();
        match self {
            Self::Fade => "none".to_string(),
            Self::FadeUp => format!("translateY({distance})"),
            Self::FadeDown => format!("translateY(calc({distance} * -1))"),
            Self::Scale => format!("scale({})", TRANSITION_SCALE.value()),
            Self::SlideUp => "translateY(100%)".to_string(),
            Self::SlideDown => "translateY(-100%)".to_string(),
            Self::SlideLeft => "translateX(100%)".to_string(),
            Self::SlideRight => "translateX(-100%)".to_string(),
            Self::Pop => "scale(0.8)".to_string(),
        }
    }
}

/// Unset leaves the theme's duration alone.
fn transition_variables(kind: TransitionKind, duration: Option<u32>) -> Variables {
    variables()
        .with(
            TRANSITION_DURATION.override_var(),
            duration.map(|ms| format!("{ms}ms")),
        )
        .with(TRANSITION_FROM, Some(kind.closed_transform()))
}

base_props! {
    pub struct TransitionProps {
        /// How the children move in and out.
        #[props(default)]
        kind: TransitionKind,
        /// Omitted, the children animate in once on mount and never exit. Passed, they enter
        /// and exit as it flips; the first value does not animate.
        #[props(default)]
        open: Option<bool>,
        /// Milliseconds, defaulting to `theme.transition.duration`. `0` disables it.
        #[props(default)]
        duration: Option<u32>,
        children: Element,
    }
}

/// Fades, slides or scales its children in on mount, and out when `open` turns false.
///
/// ```
/// # use dioxus::prelude::*;
/// # use libero::components::{Button, Paper, Text, Transition, TransitionKind};
/// # fn app() -> Element {
/// let mut show = use_signal(|| false);
/// rsx! {
///     Button { onclick: move |_| show.toggle(), "Toggle" }
///     Transition { kind: TransitionKind::FadeUp, open: show(),
///         Paper { Text { "Hello" } }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/layout/transition>
#[component]
pub fn Transition(props: TransitionProps) -> Element {
    let theme = use_theme();
    let duration = props.duration.unwrap_or(theme.transition.duration);

    // The first render is the closed markup, so the browser sees the from-state; a
    // passed `open` has no enter-on-mount, like `Collapse`.
    let mut entered = use_signal(|| props.open.is_some());
    let presence = use_presence(
        props.open.unwrap_or(true),
        EXIT_PROPERTY,
        Some(Duration::from_millis(duration.into())),
    );
    let shown = presence.visible() && entered();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("open", shown)
        .with("closed", !shown)
        .into();
    let variables: Input<Variables> = transition_variables(props.kind, props.duration).into();
    let content = rsx! {
        {presence.mounted().then_some(props.children)}
    };

    use_box()
        .framework_sx(&TRANSITION_BASE_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .variables(&variables)
        .prepare()
        .event("onmounted", move |_: Event<MountedData>| {
            presence.on_mounted();
            entered.set(true);
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
        let variables = transition_variables(TransitionKind::Fade, Some(350)).to_string();
        assert!(
            variables.contains("--lsx-transition-duration-override:350ms;"),
            "{variables}"
        );
    }

    /// Including `0`, which is a real value and not "unset".
    #[test]
    fn a_zero_duration_still_sets_the_variable() {
        let variables = transition_variables(TransitionKind::Fade, Some(0)).to_string();
        assert!(
            variables.contains("--lsx-transition-duration-override:0ms;"),
            "{variables}"
        );
    }

    #[test]
    fn no_duration_emits_no_override() {
        let variables = transition_variables(TransitionKind::Fade, None).to_string();
        assert!(!variables.contains("override"), "{variables}");
    }

    #[test]
    fn every_kind_sets_its_closed_transform() {
        let expected = [
            (TransitionKind::Fade, "none"),
            (
                TransitionKind::FadeUp,
                "translateY(var(--lsx-transition-distance))",
            ),
            (
                TransitionKind::FadeDown,
                "translateY(calc(var(--lsx-transition-distance) * -1))",
            ),
            (TransitionKind::Scale, "scale(var(--lsx-transition-scale))"),
            (TransitionKind::SlideUp, "translateY(100%)"),
            (TransitionKind::SlideDown, "translateY(-100%)"),
            (TransitionKind::SlideLeft, "translateX(100%)"),
            (TransitionKind::SlideRight, "translateX(-100%)"),
            (TransitionKind::Pop, "scale(0.8)"),
        ];
        for (kind, transform) in expected {
            let variables = transition_variables(kind, None).to_string();
            assert_eq!(
                variables,
                format!("--lsx-transition-from:{transform};"),
                "{kind:?}"
            );
        }
    }
}
