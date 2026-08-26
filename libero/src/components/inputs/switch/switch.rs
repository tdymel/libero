use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, States,
        a11y::VISUALLY_HIDDEN_SX,
        common::{base_color, base_props, contrast_color, focus_ring_sx, variables},
        layout::use_box,
    },
    hooks::{use_cache, use_css, use_id, use_theme},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CssVar, SWITCH_RADIUS, SWITCH_THUMB, SWITCH_TRACK_H, SWITCH_TRACK_W, Size, SizeCss,
        SwitchDefaults,
    },
    utils::warn,
};

const SWITCH_COLOR_VAR: CssVar = CssVar::new("--lsx-switch-color");
const SWITCH_THUMB_COLOR: CssVar = CssVar::new("--lsx-switch-thumb-color");
/// Set on the root under `:has(> input:focus-visible)` and read by the track,
/// so the ring hugs the track instead of the whole row including the label.
const SWITCH_RING: CssVar = CssVar::new("--lsx-switch-ring");
/// 0 or 1, multiplied by the travel distance - so the thumb offset is one
/// `calc`, and only this var changes between the two states.
const SWITCH_ON: CssVar = CssVar::new("--lsx-switch-on");

/// Gap between the thumb and the track's edge.
const INSET: &str = "2px";

static SWITCH_ROOT_SX: StaticSx = StaticSx::new(|| {
    SwitchDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Xs))
        // The visually hidden input is absolutely positioned; without this it
        // escapes to the nearest positioned ancestor.
        .position("relative")
        // Pins its own size - a flex parent's `stretch` would otherwise
        // decide how wide the row is.
        .width("max-content")
        .when(
            "disabled",
            sx().opacity("0.5")
                .cursor("not-allowed")
                .pointer_events("none"),
        )
        .selector(
            "&:has(> input:focus-visible)",
            focus_ring_sx().and(sx().var(SWITCH_RING, "currentcolor")),
        )
});

static SWITCH_LABEL_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .cursor("pointer")
});

static SWITCH_TRACK_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .flex("0 0 auto")
        .width(SWITCH_TRACK_W.value())
        .height(SWITCH_TRACK_H.value())
        .border_radius(SWITCH_RADIUS.value())
        .background(SWITCH_COLOR_VAR.value())
        .transition("background 150ms ease")
});

static SWITCH_THUMB_SX: StaticSx = StaticSx::new(|| {
    sx().position("absolute")
        .top("50%")
        .left(INSET)
        .width(SWITCH_THUMB.value())
        .height(SWITCH_THUMB.value())
        .border_radius("50%")
        .background(SWITCH_THUMB_COLOR.value())
        // The travel is the track minus the thumb and both insets.
        .transform(format!(
            "translate(calc({} * (({} - {} - 2 * {}))), -50%)",
            SWITCH_ON.value(),
            SWITCH_TRACK_W.value(),
            SWITCH_THUMB.value(),
            INSET,
        ))
        .transition("transform 150ms ease")
});

/// Depends on `(checked, color)` alone - see the `use_cache` below.
fn switch_variables(checked: bool, base: &ThemeAwareValue) -> String {
    variables()
        .with(
            SWITCH_COLOR_VAR,
            if checked {
                base.resolve(None)
            } else {
                ThemeAwareValue::from("grey.3").resolve(None)
            },
        )
        .with(SWITCH_ON, Some(if checked { "1" } else { "0" }.to_string()))
        .with(
            SWITCH_THUMB_COLOR,
            if checked {
                contrast_color(base).and_then(|color| color.resolve(None))
            } else {
                Some("white".to_string())
            },
        )
        .render()
}

base_props! {
    pub struct SwitchProps {
        /// The track colour when checked.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        #[props(default, into)]
        size: Input<Size>,
        /// Track corner radius; the thumb is always a circle.
        #[props(default, into)]
        radius: Input<Size>,
        /// Strictly controlled - pair it with `onchange`.
        #[props(default)]
        checked: Option<bool>,
        #[props(default)]
        disabled: Option<bool>,
        /// Called with the value `checked` should take next.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        /// Names the switch when it has no `children`; `attributes` cannot,
        /// they land on the root rather than the input.
        #[props(default, into)]
        aria_label: Option<String>,
        /// Text and `Icon` only: a `<label>` hijacks clicks on nested controls.
        #[props(default)]
        children: Element,
    }
}

/// A checkbox styled as a track and thumb.
#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let theme = use_theme();
    let color = base_color(props.color.as_ref());
    let checked = props.checked.unwrap_or(false);
    let disabled = props.disabled.unwrap_or(false);

    let size = props.size.copied_or(theme.switch.size);
    let radius = props.radius.copied_or(theme.switch.radius);

    if props.checked.is_some() && props.onchange.is_none() {
        warn("Switch: `checked` without `onchange` can never change.");
    }
    if props.onchange.is_some() && props.checked.is_none() {
        warn("Switch: `onchange` without `checked` can never appear on.");
    }

    let style = use_cache((checked, color), |(checked, color)| {
        switch_variables(*checked, color)
    });

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(size.state_name(), true)
        .with(radius.radius_state_name(), true)
        .with("checked", checked)
        .with("disabled", disabled)
        .into();

    let id = use_id();

    let root = use_box()
        .framework_sx(&SWITCH_ROOT_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .style(Some(style).filter(|style| !style.is_empty()))
        .prepare();
    let input = use_box().framework_sx(&VISUALLY_HIDDEN_SX).prepare();
    let label = use_box().framework_sx(&SWITCH_LABEL_SX).prepare();
    let track_class = use_css(Some(&SWITCH_TRACK_SX), CssLayer::Framework);
    let thumb_class = use_css(Some(&SWITCH_THUMB_SX), CssLayer::Framework);

    let onchange = props.onchange;
    let input = input
        .attr("type", "checkbox")
        .attr("role", "switch")
        .attr("id", id())
        .attr("checked", checked)
        .attr("disabled", disabled)
        .attr("aria-label", props.aria_label)
        // `onclick`, not `onchange`: cancelling the click reverts the
        // browser's own flip, so Rust state stays the only source of truth.
        .event("onclick", move |event: Event<MouseData>| {
            event.prevent_default();
            if let Some(onchange) = &onchange {
                onchange.call(!checked);
            }
        })
        // Blitz forwards a `<label>` click to its input as a default action
        // that emits `input`, never `click`, so `onclick` alone leaves the
        // switch dead there. On the web a cancelled click suppresses `input`,
        // and both handlers compute the same `!checked` anyway, so firing
        // twice is a no-op rather than a double toggle.
        .event("oninput", move |_: FormEvent| {
            if let Some(onchange) = &onchange {
                onchange.call(!checked);
            }
        })
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, Vec::new(), ());

    let track = rsx! {
        span { class: track_class, span { class: thumb_class } }
    };

    // The track lives inside the `<label>`; as a sibling it would be dead to
    // the mouse.
    let label = label.attr("for", id()).render(
        HtmlTag::Label,
        Vec::new(),
        rsx! { {track} {props.children} },
    );

    root.attr("aria-disabled", disabled)
        .render(HtmlTag::Span, props.attributes, vec![input, label])
}
