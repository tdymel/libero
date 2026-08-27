use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, States,
        a11y::VISUALLY_HIDDEN_SX,
        common::{base_color, contrast_color, field_props, focus_ring_sx, variables},
        form::use_field,
        layout::use_box,
    },
    hooks::{use_cache, use_css, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CHECKBOX_BOX, CHECKBOX_RADIUS, CheckboxDefaults, CssVar},
    utils::warn,
};

/// Background and border of the box; equal when checked, so one value would
/// not do - an unchecked box is transparent with a visible outline.
const CHECKBOX_BACKGROUND: CssVar = CssVar::new("--lsx-checkbox-background");
const CHECKBOX_BORDER: CssVar = CssVar::new("--lsx-checkbox-border");
/// The check or the dash. Inherited by the `<svg>` as `currentcolor`.
const CHECKBOX_MARK: CssVar = CssVar::new("--lsx-checkbox-mark");

static CHECKBOX_CONTROL_SX: StaticSx = StaticSx::new(|| {
    CheckboxDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        // The visually hidden input is absolutely positioned; without this it
        // escapes to the nearest positioned ancestor.
        .position("relative")
        // The box is the whole control, so the ring hugs it rather than the
        // row - `:has`, because there is no `:focus-visible-within`.
        .selector("&:has(> input:focus-visible)", focus_ring_sx())
        .when("disabled", sx().opacity("0.5").cursor("not-allowed"))
});

static CHECKBOX_BOX_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex("0 0 auto")
        .width(CHECKBOX_BOX.value())
        .height(CHECKBOX_BOX.value())
        .border_radius(CHECKBOX_RADIUS.value())
        .border(format!("1px solid {}", CHECKBOX_BORDER.value()))
        .background(CHECKBOX_BACKGROUND.value())
        .color(CHECKBOX_MARK.value())
        .cursor("pointer")
        .transition("background 150ms ease, border-color 150ms ease")
});

/// Depends on `(on, color)` alone - see the `use_cache` below.
fn checkbox_variables(on: bool, base: &ThemeAwareValue) -> String {
    let off = ThemeAwareValue::from("grey.5");
    variables()
        .with(
            CHECKBOX_BACKGROUND,
            match on {
                true => base.resolve(None),
                false => Some("transparent".to_string()),
            },
        )
        .with(
            CHECKBOX_BORDER,
            match on {
                true => base.resolve(None),
                false => off.resolve(None),
            },
        )
        .with(
            CHECKBOX_MARK,
            match on {
                true => contrast_color(base).and_then(|color| color.resolve(None)),
                false => Some("transparent".to_string()),
            },
        )
        .render()
}

field_props! {
    extends(input);
    pub struct CheckboxProps {
        /// The box's colour when checked.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Strictly controlled - pair it with `onchange`.
        #[props(default)]
        checked: Option<bool>,
        /// Draws the mixed state and reads as `aria-checked="mixed"`. Outranks
        /// `checked` visually; toggling from it gives `true`.
        #[props(default)]
        indeterminate: Option<bool>,
        /// Called with the value `checked` should take next.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        /// Names the checkbox when it has no `label`; `attributes` cannot,
        /// they land on the input but a caller may not want a visible label.
        #[props(default, into)]
        aria_label: Option<String>,
    }
}

/// A checkbox, with its label beside the box and the description, helper text
/// and validation message under both.
///
/// Strictly controlled: pass `checked` and handle `onchange`. The click is
/// cancelled and the DOM re-rendered from Rust, so the property, `:checked`,
/// assistive tech and form submission never drift apart.
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let theme = use_theme();
    let color = base_color(props.color.as_ref());

    let size = props.size.copied_or(theme.checkbox.size);
    let radius = props.radius.copied_or(theme.checkbox.radius);
    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);
    let checked = props.checked.unwrap_or(false);
    let indeterminate = props.indeterminate.unwrap_or(false);

    if props.checked.is_some() && props.onchange.is_none() {
        warn("Checkbox: `checked` without `onchange` can never change.");
    }
    if props.onchange.is_some() && props.checked.is_none() {
        warn("Checkbox: `onchange` without `checked` can never appear checked.");
    }

    let field = use_field()
        .inline()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

    // The field's own states plus what only the box reads. `mixed` wins over
    // `checked`, the same way `aria-checked="mixed"` does.
    let states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("checked", checked && !indeterminate)
        .with("mixed", indeterminate)
        .into();

    let style = use_cache((checked || indeterminate, color), |(on, color)| {
        checkbox_variables(*on, color)
    });

    let control = use_box()
        .framework_sx(&CHECKBOX_CONTROL_SX)
        .states(&states)
        .style(Some(style).filter(|style| !style.is_empty()))
        .prepare();
    let input = use_box()
        .framework_sx(&VISUALLY_HIDDEN_SX)
        .focus_ring(false)
        .prepare();
    let box_class = use_css(Some(&CHECKBOX_BOX_SX), CssLayer::Framework);

    let onchange = props.onchange;
    let next = !(checked || indeterminate);
    let toggle = move || {
        if let Some(onchange) = &onchange {
            onchange.call(next);
        }
    };

    let input = field
        .aria(input)
        .attr("type", "checkbox")
        .attr("checked", checked && !indeterminate)
        .attr("aria-checked", indeterminate.then_some("mixed"))
        .attr("disabled", disabled)
        .attr("required", required)
        .attr("aria-label", props.aria_label)
        // `onclick`, not `onchange`: cancelling the click reverts the
        // browser's own flip, so Rust state stays the only source of truth.
        .event("onclick", move |event: Event<MouseData>| {
            event.prevent_default();
            toggle();
        })
        // Blitz forwards a `<label>` click to its input as a default action
        // that emits `input`, never `click`, so `onclick` alone leaves the
        // checkbox dead there. On the web a cancelled click suppresses
        // `input`, and both handlers compute the same value anyway.
        .event("oninput", move |_: FormEvent| toggle())
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, props.attributes, ());

    // The box is decoration: the input owns the name, the state and the
    // keyboard. It carries the click because a visually hidden input has no
    // hit area, and a second `<label for>` around it would compete with the
    // field's own label for the accessible name.
    let mark = rsx! {
        svg {
            width: "65%",
            height: "65%",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentcolor",
            stroke_width: "3",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            if indeterminate {
                path { d: "M6 12h12" }
            } else {
                path { d: "M5 13l4 4L19 7" }
            }
        }
    };
    let box_node = rsx! {
        span {
            class: box_class,
            "aria-hidden": "true",
            onclick: move |_| {
                if !disabled {
                    toggle();
                }
            },
            {mark}
        }
    };

    field.render(control.render(HtmlTag::Span, Vec::new(), vec![input, box_node]))
}
