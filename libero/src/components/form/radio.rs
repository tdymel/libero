use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, States,
        a11y::VISUALLY_HIDDEN_SX,
        common::{base_color, field_props, focus_ring_sx, variables},
        form::use_field,
        layout::use_box,
    },
    hooks::{use_cache, use_css, use_element, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
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
        .selector("&:has(> input:focus-visible)", focus_ring_sx())
        // A card rings itself; a second ring here would sit at another offset.
        .when(
            "card",
            sx().selector("&:has(> input:focus-visible)", sx().outline("none")),
        )
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
        .transform(format!("scale({})", RADIO_ON.value()))
        .transition("transform 150ms ease")
});

/// Depends on `(checked, color)` alone - see the `use_cache` below.
fn radio_variables(checked: bool, base: &ThemeAwareValue) -> String {
    variables()
        .with(
            RADIO_COLOR,
            match checked {
                true => base.resolve(None),
                false => ThemeAwareValue::from("grey.5").resolve(None),
            },
        )
        .with(RADIO_ON, Some(if checked { "1" } else { "0" }.to_string()))
        .render()
}

field_props! {
    extends(input);
    pub struct RadioProps {
        /// The ring and dot colour when checked.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Strictly controlled - pair it with `onselect`.
        #[props(default)]
        checked: Option<bool>,
        /// Fires when this radio is picked. Never fires to *unpick* one: a
        /// radio is turned off by another in its group being turned on.
        #[props(default)]
        onselect: Option<EventHandler<()>>,
        /// Shared by every radio in one group, which is what makes the native
        /// control exclusive. `RadioGroup` sets it.
        #[props(default, into)]
        name: Option<String>,
        /// `RadioGroup` makes exactly one radio the group's tab stop and takes
        /// the rest out of the tab order.
        #[props(default, into)]
        tabindex: Option<String>,
        /// Names the radio when it has no `label`.
        #[props(default, into)]
        aria_label: Option<String>,
        /// `Card` draws the radio as a bordered surface and makes all of it
        /// the hit area. The label and captions must not hold anything
        /// interactive of their own: a click on it would reach the card too.
        #[props(default, into)]
        variant: Input<ChoiceVariant>,
    }
}

/// One option of a radio group: a circle with its label beside it.
///
/// Prefer [`RadioGroup`](crate::components::RadioGroup) - a radio on its own
/// owns neither the shared `name` that makes the set exclusive nor the single
/// tab stop the ARIA pattern asks for. This is the escape hatch for a caller
/// laying a group out by hand.
#[component]
pub fn Radio(props: RadioProps) -> Element {
    let theme = use_theme();
    let color = base_color(props.color.as_ref());

    let size = props.size.copied_or(theme.radio.size);
    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);
    let checked = props.checked.unwrap_or(false);
    let card = props.variant.copied_or_default() == ChoiceVariant::Card;

    if props.checked.is_some() && props.onselect.is_none() {
        warn("Radio: `checked` without `onselect` can never change.");
    }

    let onselect = props.onselect;
    // Picking the checked radio again is not a change - and there is no way to
    // unpick one, so a second click reports nothing.
    let select = move || {
        if let Some(onselect) = &onselect
            && !checked
            && !disabled
        {
            onselect.call(());
        }
    };

    let field = use_field()
        .inline()
        .card(card)
        .activates(use_element(), select)
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .required(required)
        .disabled(disabled)
        .size(size)
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .attributes(&props.attributes)
        .prepare();

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

    // The circle is decoration: the input owns the name, the state and the
    // keyboard. It carries the click because a visually hidden input has no
    // hit area, and a `<label>` around it would compete with the field's own
    // label for the accessible name. A card takes the click itself.
    let circle = rsx! {
        span {
            class: circle_class,
            "aria-hidden": "true",
            onclick: move |_| {
                if !card {
                    select();
                }
            },
            span { class: dot_class }
        }
    };

    field.render(control.render(HtmlTag::Span, Vec::new(), vec![input, circle]))
}
