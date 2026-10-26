use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        common::{
            HtmlTag, Input, Part, States, base_color_or, coarse_hit_area_sx, disabled_look_sx,
            fill_color, min_hit_area_sx, names_itself, use_name_warning, variables,
        },
        form::{choice, field_parts_enum, field_props, use_field, use_fieldset_disabled},
    },
    hooks::{use_cache, use_css, use_element, use_theme},
    sx::{FORCED_COLORS, REDUCED_MOTION, StaticSx, ThemeAwareValue, sx},
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
    choice::control_sx(RadioDefaults::theme_vars())
        .when("disabled", disabled_look_sx("not-allowed"))
});

static RADIO_CIRCLE_SX: StaticSx = StaticSx::new(|| {
    sx().display("inline-flex")
        .align_items("center")
        .justify_content("center")
        .flex("0 0 auto")
        // Carries the click, so it carries a 24px floor (WCAG 2.5.8) and the finger-sized area.
        .position("relative")
        .and(min_hit_area_sx("::before"))
        .and(coarse_hit_area_sx("::before"))
        .width(RADIO_CIRCLE.value())
        .height(RADIO_CIRCLE.value())
        // A radio is round at every size - that is what tells it apart from a
        // checkbox at a glance, so there is no `radius` prop to honour.
        .border_radius("50%")
        .border(format!("1px solid {}", RADIO_COLOR.value()))
        .transition("border-color 150ms ease")
        .media(REDUCED_MOTION, sx().transition("none"))
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
        .media(REDUCED_MOTION, sx().transition("none"))
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
/// which owns the shared `name` and the single tab stop. A `Form` does not
/// enforce `required` on a lone `Radio`; set it on the group.
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
    let color = base_color_or(props.color.as_ref(), theme.radio.color);

    let size = props.size.copied_or(theme.radio.size);
    // A disabled `Fieldset` disables the input, but not the circle and label that pick it.
    let group = use_fieldset_disabled();
    let disabled = props.disabled.unwrap_or(false) || group;
    let required = crate::components::form::fieldset_required(props.required);
    let checked = props.checked.unwrap_or(false);
    // HTML's `readonly` does not apply to a radio, so the activation is refused
    // here. ARIA has no `aria-readonly` on `radio` either: the radiogroup says it.
    let readonly = props.readonly.unwrap_or(false);
    let card = props.variant.copied_or(theme.radio.variant) == ChoiceVariant::Card;

    if props.checked.is_some() && props.onselect.is_none() {
        warn("Radio: `checked` without `onselect` can never change.");
    }
    if props.onselect.is_some() && props.checked.is_none() {
        warn("Radio: `onselect` without `checked` can never appear checked.");
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
    let aria_label = choice::aria_label(&props.aria_label);
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
        !props.label.is_none() || aria_label.is_some() || names_itself(&props.attributes),
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

    let (control, input) = choice::use_boxes(&RADIO_CONTROL_SX, &states, style);
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
        .attr("aria-label", aria_label)
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, props.attributes, ());

    // Decoration that carries the click: a hidden input has no hit area, and a
    // `<label>` would compete for the name.
    let press = choice::press(card, disabled, element, select);
    let circle = rsx! {
        span {
            class: circle_class,
            "data-slot": RadioPart::Circle.slot(),
            "aria-hidden": "true",
            onclick: move |_| press(),
            span { class: dot_class, "data-slot": RadioPart::Dot.slot() }
        }
    };

    field.render(choice::control(
        control,
        RadioPart::Control.slot(),
        input,
        circle,
    ))
}
