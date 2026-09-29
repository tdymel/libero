use std::time::Duration;

use dioxus::prelude::*;

use crate::{
    components::{
        common::{HtmlTag, Input, States, Variables, base_props, variables},
        layout::use_box,
    },
    hooks::{use_presence, use_theme},
    sx::{REDUCED_MOTION, StaticSx, Sx, SxEntry, sx},
    theme::{
        CssVar, TRANSITION_APPEAR, TRANSITION_DISTANCE, TRANSITION_DURATION, TRANSITION_EASING,
        TRANSITION_POP_SCALE, TRANSITION_ROTATE, TRANSITION_SCALE, TRANSITION_SKEW,
    },
};

/// The property `use_presence` filters `transitionend` by. Every kind fades,
/// so it is the one property all of them transition.
const EXIT_PROPERTY: &str = "opacity";

/// The closed `transform`, set per instance from the [`TransitionKind`]; `TRANSITION_KEYFRAMES` reads it too.
const TRANSITION_FROM: CssVar = CssVar::new("--lsx-transition-from");
/// The `transform-origin`, per kind; outside the states so it never jumps mid-transition.
const TRANSITION_ORIGIN: CssVar = CssVar::new("--lsx-transition-origin");

/// `opacity` and `transform`, plus any `extra` properties, each over the duration.
fn motion(extra: &[&str]) -> String {
    let duration = TRANSITION_DURATION.overridable();
    let easing = TRANSITION_EASING.value();
    ["opacity", "transform"]
        .iter()
        .chain(extra)
        .map(|property| format!("{property} {duration} {easing}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Both states' `transition`; `visibility` flips at once on open, after the exit on close.
fn state_transitions(extra: &[&str]) -> Sx {
    let duration = TRANSITION_DURATION.overridable();
    let motion = motion(extra);
    sx().when(
        "open",
        sx().transition(format!("{motion}, visibility 0s linear 0s"))
            .media(REDUCED_MOTION, sx().transition("none")),
    )
    .when(
        "closed",
        sx().transition(format!("{motion}, visibility 0s linear {duration}"))
            .media(REDUCED_MOTION, sx().transition("none")),
    )
}

/// Each reduced-motion guard sits in its state's `when(..)` block, or loses on specificity.
/// `visibility`, not `aria-hidden`/`inert`, drops closed content from focus and the a11y tree.
static TRANSITION_BASE_SX: StaticSx = StaticSx::new(|| {
    let duration = TRANSITION_DURATION.overridable();
    let easing = TRANSITION_EASING.value();

    sx().transform_origin(TRANSITION_ORIGIN.value())
        .when(
            "open",
            sx().opacity("1").transform("none").visibility("visible"),
        )
        .when(
            "closed",
            sx().opacity("0")
                .transform(TRANSITION_FROM.value())
                .visibility("hidden"),
        )
        // A keyframe, not a state flip: the server markup is already open and visible.
        .when(
            "appear",
            sx().animation(format!("{TRANSITION_APPEAR} {duration} {easing} backwards"))
                .media(REDUCED_MOTION, sx().animation("none")),
        )
        .and(state_transitions(&[]))
});

/// The caller's from-state under `closed`, its top-level properties added to `transition`.
fn from_sx(from: &Sx) -> Sx {
    let extra: Vec<&str> = from
        .entries()
        .iter()
        .filter_map(|entry| match entry {
            SxEntry::Declaration { property, .. } => Some(property.as_str()),
            SxEntry::Nested { .. } => None,
        })
        .filter(|property| !matches!(*property, "opacity" | "transform" | "transition"))
        .collect();
    sx().when("closed", from.clone())
        .and(state_transitions(&extra))
}

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
    /// Grows from `theme.transition.pop_scale`, smaller than `Scale`.
    Pop,
    /// Moves left into place (physical, not logical).
    FadeLeft,
    /// Moves right into place (physical, not logical).
    FadeRight,
    /// Grows its width from zero, out of the left edge.
    ScaleX,
    /// Grows its height from zero, out of the top edge.
    ScaleY,
    /// Rises into place, leaning by `theme.transition.skew`; Mantine's `skew-up` sinks instead.
    SkewUp,
    /// Sinks into place, leaning by `theme.transition.skew`.
    SkewDown,
    /// Rises into place, turning clockwise from `theme.transition.rotate`.
    RotateLeft,
    /// Rises into place, turning counter-clockwise from `theme.transition.rotate`.
    RotateRight,
    /// Grows from zero out of the top left corner.
    PopTopLeft,
    /// Grows from zero out of the top right corner.
    PopTopRight,
    /// Grows from zero out of the bottom left corner.
    PopBottomLeft,
    /// Grows from zero out of the bottom right corner.
    PopBottomRight,
}

impl TransitionKind {
    /// The `transform` of the closed state.
    fn closed_transform(self) -> String {
        let distance = TRANSITION_DISTANCE.value();
        let rotate = TRANSITION_ROTATE.value();
        let skew = TRANSITION_SKEW.value();
        let lean = format!("skew(calc({skew} * -1), calc({skew} * -0.5))");
        match self {
            Self::Fade => "none".to_string(),
            Self::FadeUp => format!("translateY({distance})"),
            Self::FadeDown => format!("translateY(calc({distance} * -1))"),
            Self::Scale => format!("scale({})", TRANSITION_SCALE.value()),
            Self::SlideUp => "translateY(100%)".to_string(),
            Self::SlideDown => "translateY(-100%)".to_string(),
            Self::SlideLeft => "translateX(100%)".to_string(),
            Self::SlideRight => "translateX(-100%)".to_string(),
            Self::Pop => format!("scale({})", TRANSITION_POP_SCALE.value()),
            Self::FadeLeft => format!("translateX({distance})"),
            Self::FadeRight => format!("translateX(calc({distance} * -1))"),
            Self::ScaleX => "scaleX(0)".to_string(),
            Self::ScaleY => "scaleY(0)".to_string(),
            Self::SkewUp => format!("translateY({distance}) {lean}"),
            Self::SkewDown => format!("translateY(calc({distance} * -1)) {lean}"),
            Self::RotateLeft => format!("translateY({distance}) rotate(calc({rotate} * -1))"),
            Self::RotateRight => format!("translateY({distance}) rotate({rotate})"),
            Self::PopTopLeft | Self::PopTopRight | Self::PopBottomLeft | Self::PopBottomRight => {
                "scale(0)".to_string()
            }
        }
    }

    /// The `transform-origin` the kind grows, leans or turns around.
    fn origin(self) -> &'static str {
        match self {
            Self::ScaleX => "left",
            Self::ScaleY | Self::SkewDown | Self::RotateRight => "top",
            Self::SkewUp | Self::RotateLeft => "bottom",
            Self::PopTopLeft => "top left",
            Self::PopTopRight => "top right",
            Self::PopBottomLeft => "bottom left",
            Self::PopBottomRight => "bottom right",
            Self::Fade
            | Self::FadeUp
            | Self::FadeDown
            | Self::FadeLeft
            | Self::FadeRight
            | Self::Scale
            | Self::SlideUp
            | Self::SlideDown
            | Self::SlideLeft
            | Self::SlideRight
            | Self::Pop => "center",
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
        .with(TRANSITION_ORIGIN, Some(kind.origin().to_string()))
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
        /// Extra styles of the closed state, on top of `kind`'s; every property it sets animates.
        /// Needs a passed `open`: the mount entrance animates only the `kind`.
        #[props(default, into)]
        from: Option<Sx>,
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

    // A passed `open` has no enter-on-mount, like `Collapse`; an omitted one renders open
    // with the `appear` keyframe, so server and client agree and no JS is needed.
    let presence = use_presence(
        props.open.unwrap_or(true),
        EXIT_PROPERTY,
        Some(Duration::from_millis(duration.into())),
    );
    let shown = presence.visible();

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with("open", shown)
        .with("closed", !shown)
        .with("appear", props.open.is_none())
        .into();
    let variables: Input<Variables> = transition_variables(props.kind, props.duration).into();
    let sx: Input<Sx> = match &props.from {
        Some(from) => from_sx(from)
            .and(props.sx.clone().unwrap_or_default())
            .into(),
        None => props.sx.clone(),
    };
    let content = rsx! {
        {presence.mounted().then_some(props.children)}
    };

    use_box()
        .framework_sx(&TRANSITION_BASE_SX)
        .class(&props.class)
        .sx(&sx)
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
            (
                TransitionKind::Pop,
                "scale(var(--lsx-transition-pop-scale))",
            ),
            (
                TransitionKind::FadeLeft,
                "translateX(var(--lsx-transition-distance))",
            ),
            (
                TransitionKind::FadeRight,
                "translateX(calc(var(--lsx-transition-distance) * -1))",
            ),
            (TransitionKind::ScaleX, "scaleX(0)"),
            (TransitionKind::ScaleY, "scaleY(0)"),
            (
                TransitionKind::SkewUp,
                "translateY(var(--lsx-transition-distance)) skew(calc(var(--lsx-transition-skew) * -1), calc(var(--lsx-transition-skew) * -0.5))",
            ),
            (
                TransitionKind::SkewDown,
                "translateY(calc(var(--lsx-transition-distance) * -1)) skew(calc(var(--lsx-transition-skew) * -1), calc(var(--lsx-transition-skew) * -0.5))",
            ),
            (
                TransitionKind::RotateLeft,
                "translateY(var(--lsx-transition-distance)) rotate(calc(var(--lsx-transition-rotate) * -1))",
            ),
            (
                TransitionKind::RotateRight,
                "translateY(var(--lsx-transition-distance)) rotate(var(--lsx-transition-rotate))",
            ),
            (TransitionKind::PopTopLeft, "scale(0)"),
            (TransitionKind::PopTopRight, "scale(0)"),
            (TransitionKind::PopBottomLeft, "scale(0)"),
            (TransitionKind::PopBottomRight, "scale(0)"),
        ];
        for (kind, transform) in expected {
            let variables = transition_variables(kind, None).to_string();
            assert!(
                variables.starts_with(&format!("--lsx-transition-from:{transform};")),
                "{kind:?}: {variables}"
            );
        }
    }

    #[test]
    fn every_kind_sets_its_origin() {
        let expected = [
            (TransitionKind::Fade, "center"),
            (TransitionKind::Pop, "center"),
            (TransitionKind::ScaleX, "left"),
            (TransitionKind::ScaleY, "top"),
            (TransitionKind::SkewUp, "bottom"),
            (TransitionKind::SkewDown, "top"),
            (TransitionKind::RotateLeft, "bottom"),
            (TransitionKind::RotateRight, "top"),
            (TransitionKind::PopTopLeft, "top left"),
            (TransitionKind::PopTopRight, "top right"),
            (TransitionKind::PopBottomLeft, "bottom left"),
            (TransitionKind::PopBottomRight, "bottom right"),
        ];
        for (kind, origin) in expected {
            let variables = transition_variables(kind, None).to_string();
            assert!(
                variables.ends_with(&format!("--lsx-transition-origin:{origin};")),
                "{kind:?}: {variables}"
            );
        }
    }
}
