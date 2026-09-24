use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        accessibility::VISUALLY_HIDDEN_SX,
        common::{
            HtmlTag, Input, Part, States, base_color, fill_color, focus_ring_sx, names_itself,
            ring_overlay, ring_overlay_sx, use_name_warning, variables,
        },
        form::{field_parts_enum, field_props, use_field},
        layout::use_box,
    },
    hooks::{use_cache, use_css, use_element, use_theme},
    platform::ElementApi,
    sx::{FORCED_COLORS, StaticSx, ThemeAwareValue, sx},
    theme::{ChoiceVariant, CssVar, RADIO_CIRCLE, RadioDefaults},
    utils::warn,
};

/// Ring colour of the circle; the dot reads it too, so a checked radio is one
/// colour rather than two that have to agree.
const RADIO_COLOR: CssVar = CssVar::new("--lsx-radio-color");
/// 0 or 1, scaling the dot - so only this var changes between the two states
/// and the transition is free. Same trick as `Switch`'s thumb offset.
const RADIO_ON: CssVar = CssVar::new("--lsx-radio-on");

static RADIO_CONTROL_SX: StaticSx = StaticSx::new(|| {
    RadioDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        // The visually hidden input is absolutely positioned; without this it
        // escapes to the nearest positioned ancestor.
        .position("relative")
        // On the control rather than the circle, so `disabled` below reaches it.
        .cursor("pointer")
        // Drawn by the overlay after the circle, because the focus is on the
        // input beside it - the same shape as `Checkbox`'s.
        .selector("& > [data-ring]", ring_overlay_sx())
        .selector("& > input:focus-visible ~ [data-ring]", focus_ring_sx())
        // A card rings itself: the control stops being the overlay's
        // containing block, so the same overlay covers the card.
        .when("card", sx().position("static"))
        .when("disabled", sx().opacity("0.5").cursor("not-allowed"))
});

static RADIO_CIRCLE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex("0 0 auto")
        .width(RADIO_CIRCLE.value())
        .height(RADIO_CIRCLE.value())
        // A radio is round at every size - that is what tells it apart from a
        // checkbox at a glance, so there is no `radius` prop to honour.
        .border_radius("50%")
        .border(format!("1px solid {}", RADIO_COLOR.value()))
        .transition("border-color 150ms ease")
});

static RADIO_DOT_SX: StaticSx = StaticSx::new(|| {
    sx().width("50%")
        .height("50%")
        .border_radius("50%")
        .background(RADIO_COLOR.value())
        // Forced colours would paint it `Canvas`, so a checked radio looked
        // unchecked (todo 506).
        .media(FORCED_COLORS, sx().background("CanvasText"))
        .transform(format!("scale({})", RADIO_ON.value()))
        .transition("transform 150ms ease")
});

/// Depends on `(checked, color)` alone - see the `use_cache` below.
fn radio_variables(checked: bool, base: &ThemeAwareValue) -> String {
    variables()
        .with(
            RADIO_COLOR,
            match checked {
                true => fill_color(base),
                // 3:1 on the page, as `Checkbox`'s outline (todo 490).
                false => ThemeAwareValue::from("muted.6").resolve(None),
            },
        )
        .with(RADIO_ON, Some(if checked { "1" } else { "0" }.to_string()))
        .render()
}

field_parts_enum! {
    /// [`Radio`]'s inner parts, for its `parts` prop: a field's, the control,
    /// its circle and dot.
    pub enum RadioPart {
        /// Holds the hidden input and the circle, beside the label.
        Control = "control" => "& > [data-slot='control']",
        /// The drawn ring.
        Circle = "circle" => "& > [data-slot='control'] > [data-slot='circle']",
        /// The checked mark, scaled to nothing while unchecked.
        Dot = "dot" => "& > [data-slot='control'] > [data-slot='circle'] > [data-slot='dot']",
    }
}

field_props! {
    parts(RadioPart);
    extends(input);
    without(radius);
    pub struct RadioProps {
        /// The ring and dot colour when checked.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Strictly controlled - pair it with `onselect`.
        #[props(default)]
        checked: Option<bool>,
        /// Fires when this radio is picked; never to unpick it.
        #[props(default)]
        onselect: Option<EventHandler<()>>,
        /// Shared by every radio in one group. `RadioGroup` sets it.
        #[props(default, into)]
        name: Option<String>,
        /// `RadioGroup` sets one tab stop per group.
        #[props(default, into)]
        tabindex: Option<String>,
        /// Names the radio when it has no `label`.
        #[props(default, into)]
        aria_label: Option<String>,
        /// `Card` draws a bordered surface that is all hit area.
        #[props(default, into)]
        variant: Input<ChoiceVariant>,
    }
}

/// One radio with its label. Prefer [`RadioGroup`](crate::components::RadioGroup),
/// which owns the shared `name` and the single tab stop.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Radio;
/// # fn app() -> Element {
/// let mut express = use_signal(|| false);
/// rsx! {
///     Radio {
///         label: "Express shipping",
///         name: "shipping",
///         checked: express(),
///         onselect: move |_| express.set(true),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/radio-group>
#[component]
pub fn Radio(props: RadioProps) -> Element {
    let theme = use_theme();
    let color = base_color(props.color.as_ref());

    let size = props.size.copied_or(theme.radio.size);
    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);
    let checked = props.checked.unwrap_or(false);
    // HTML's `readonly` does not apply to a radio, so the activation is refused
    // here. ARIA has no `aria-readonly` on `radio` either: the radiogroup says it.
    let readonly = props.readonly.unwrap_or(false);
    let card = props.variant.copied_or(theme.radio.variant) == ChoiceVariant::Card;

    if props.checked.is_some() && props.onselect.is_none() {
        warn("Radio: `checked` without `onselect` can never change.");
    }

    let onselect = props.onselect;
    // Picking the checked radio again is not a change.
    let select = move || {
        if let Some(onselect) = &onselect
            && !checked
            && !disabled
            && !readonly
        {
            onselect.call(());
        }
    };

    let element = use_element();
    let field = use_field()
        .inline()
        .card(card)
        .activates(element, select)
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .required(required)
        .disabled(disabled)
        .size(size)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();
    use_name_warning(
        !props.label.is_none() || props.aria_label.is_some() || names_itself(&props.attributes),
        "Radio: no `label` or `aria_label`, so it is announced as just \"radio button\".",
    );

    let states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("checked", checked)
        .into();

    let style = use_cache((checked, color), |(checked, color)| {
        radio_variables(*checked, color)
    });

    let control = use_box()
        .framework_sx(&RADIO_CONTROL_SX)
        .states(&states)
        .style(Some(style).filter(|style| !style.is_empty()))
        .prepare();
    let input = use_box()
        .framework_sx(&VISUALLY_HIDDEN_SX)
        .focus_ring(false)
        .prepare();
    let circle_class = use_css(Some(&RADIO_CIRCLE_SX), CssLayer::Framework);
    let dot_class = use_css(Some(&RADIO_DOT_SX), CssLayer::Framework);

    let input = field
        .aria(input)
        .attr("type", "radio")
        .attr("name", props.name)
        .attr("tabindex", props.tabindex)
        .attr("checked", checked)
        .attr("data-controlled", true)
        .attr("disabled", disabled)
        .attr("required", required)
        .attr("aria-label", props.aria_label)
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, props.attributes, ());

    // Decoration that carries the click: a hidden input has no hit area, and a
    // `<label>` would compete for the name. A card takes the click itself.
    let circle = rsx! {
        span {
            class: circle_class,
            "data-slot": RadioPart::Circle.slot(),
            "aria-hidden": "true",
            // Focuses the input as a native click would.
            onclick: move |_| {
                if !card {
                    select();
                    if !disabled {
                        let _ = element.focus();
                    }
                }
            },
            span { class: dot_class, "data-slot": RadioPart::Dot.slot() }
        }
    };

    field.render(control.attr("data-slot", RadioPart::Control.slot()).render(
        HtmlTag::Span,
        Vec::new(),
        vec![input, circle, ring_overlay()],
    ))
}
