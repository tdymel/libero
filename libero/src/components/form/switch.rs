use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        accessibility::{VISUALLY_HIDDEN_SX, hidden_input_centred_sx},
        common::{
            HtmlTag, Input, Part, States, TOOLBAR_ITEM, ToolbarItem, base_color, contrast_color,
            disabled_look_sx, fill_color, focus_ring_sx, names_itself, ring_overlay,
            ring_overlay_sx, use_name_warning, use_toolbar_item, variables,
        },
        form::{field_parts_enum, field_props, use_bound, use_field, use_form_context},
        layout::use_box,
    },
    hooks::{use_cache, use_css, use_element, use_form_owner, use_theme},
    platform::ElementApi,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{
        ChoiceVariant, CssVar, NamedColorCss, SWITCH_RADIUS, SWITCH_THUMB, SWITCH_TRACK_H,
        SWITCH_TRACK_W, SwitchDefaults,
    },
    utils::warn,
};

const SWITCH_COLOR_VAR: CssVar = CssVar::new("--lsx-switch-color");
const SWITCH_THUMB_COLOR: CssVar = CssVar::new("--lsx-switch-thumb-color");
/// 0 or 1, multiplied by the travel distance - so the thumb offset is one
/// `calc`, and only this var changes between the two states.
const SWITCH_ON: CssVar = CssVar::new("--lsx-switch-on");

/// Gap between the thumb and the track's edge.
const INSET: &str = "2px";

static SWITCH_CONTROL_SX: StaticSx = StaticSx::new(|| {
    SwitchDefaults::theme_vars()
        .display("inline-flex")
        .align_items("center")
        // The visually hidden input is absolutely positioned; without this it
        // escapes to the nearest positioned ancestor.
        .position("relative")
        .selector("& > input", hidden_input_centred_sx())
        // On the control rather than the track, so `disabled` below reaches it.
        .cursor("pointer")
        // The ring hugs the track, drawn by the overlay: the focus is on the
        // input beside it.
        .selector("& > [data-ring]", ring_overlay_sx())
        .selector("& > input:focus-visible ~ [data-ring]", focus_ring_sx())
        // A card rings itself: the control stops being the overlay's
        // containing block, so the same overlay covers the card.
        .when("card", sx().position("static"))
        .when("disabled", disabled_look_sx("not-allowed"))
});

static SWITCH_TRACK_SX: StaticSx = StaticSx::new(|| {
    sx().position("relative")
        .flex("0 0 auto")
        .width(SWITCH_TRACK_W.value())
        .height(SWITCH_TRACK_H.value())
        .border_radius(SWITCH_RADIUS.value())
        .background(SWITCH_COLOR_VAR.value())
        // Forced colours drop the background but paint a transparent outline,
        // so the track keeps its shape there. Costs no layout.
        .outline("1px solid transparent")
        .transition("background 150ms ease")
});

static SWITCH_THUMB_SX: StaticSx = StaticSx::new(|| {
    // The travel is the track minus the thumb and both insets.
    let travel = format!(
        "({} - {} - 2 * {})",
        SWITCH_TRACK_W.value(),
        SWITCH_THUMB.value(),
        INSET,
    );
    sx().position("absolute")
        .top("50%")
        .left(INSET)
        .width(SWITCH_THUMB.value())
        .height(SWITCH_THUMB.value())
        .border_radius("50%")
        .background(SWITCH_THUMB_COLOR.value())
        .outline("1px solid transparent")
        .transform(format!(
            "translate(calc({} * ({travel})), -50%)",
            SWITCH_ON.value(),
        ))
        // Off sits at the start, so under RTL it starts on the right.
        .rtl(sx().left("auto").right(INSET).transform(format!(
            "translate(calc(-1 * {} * ({travel})), -50%)",
            SWITCH_ON.value(),
        )))
        .transition("transform 150ms ease")
});

/// Depends on `(checked, color)` alone - see the `use_cache` below.
fn switch_variables(checked: bool, base: &ThemeAwareValue) -> String {
    variables()
        .with(
            SWITCH_COLOR_VAR,
            if checked {
                fill_color(base)
            } else {
                // 3:1 against the page and against the `surface` thumb on it,
                // in both schemes (WCAG 1.4.11, todo 490).
                ThemeAwareValue::from("muted.6").resolve(None)
            },
        )
        .with(SWITCH_ON, Some(if checked { "1" } else { "0" }.to_string()))
        .with(
            SWITCH_THUMB_COLOR,
            if checked {
                contrast_color(base).and_then(|color| color.resolve(None))
            } else {
                // A `Variables` value is raw CSS, so this is the var and not
                // the colour name `sx` would have parsed (todo 385).
                Some(NamedColorCss::SURFACE.value())
            },
        )
        .render()
}

field_parts_enum! {
    /// [`Switch`]'s inner parts, for its `parts` prop: a field's, the control,
    /// its track and thumb.
    pub enum SwitchPart {
        /// Holds the hidden input and the track, beside the label.
        Control = "control" => "& > [data-slot='control']",
        /// The pill the thumb slides along.
        Track = "track" => "& > [data-slot='control'] > [data-slot='track']",
        /// The sliding knob.
        Thumb = "thumb" => "& > [data-slot='control'] > [data-slot='track'] > [data-slot='thumb']",
    }
}

field_props! {
    parts(SwitchPart);
    extends(input);
    pub struct SwitchProps {
        /// The track colour when checked.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Pair it with `onchange`; left out, the switch keeps its own state.
        #[props(default)]
        checked: Option<bool>,
        /// Called with the value `checked` should take next.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        /// Rules over `checked`, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<bool>,
        /// What the field posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<bool>,
        /// Names the switch when it has no `label`.
        #[props(default, into)]
        aria_label: Option<String>,
        /// `Card` draws a bordered surface that is all hit area.
        #[props(default, into)]
        variant: Input<ChoiceVariant>,
    }
}

/// An on/off switch with its label, description and validation message.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Switch;
/// # fn app() -> Element {
/// let mut enabled = use_signal(|| true);
/// rsx! {
///     Switch {
///         label: "Email notifications",
///         checked: enabled(),
///         onchange: move |next| enabled.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/switch>
#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let theme = use_theme();
    let color = base_color(props.color.as_ref());

    let size = props.size.copied_or(theme.switch.size);
    let radius = props.radius.copied_or(theme.switch.radius);
    let required = props.required.unwrap_or(false);
    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let checked = bound
        .value()
        .or(props.checked)
        .or(bound.entered())
        .unwrap_or(false);

    // HTML `readonly` does not apply to a checkbox: refused here, announced
    // with `aria-readonly`.
    let readonly = props.readonly.unwrap_or(false);
    let card = props.variant.copied_or(theme.switch.variant) == ChoiceVariant::Card;

    if props.checked.is_some() && props.onchange.is_none() && !bound.is_bound() {
        warn("Switch: `checked` without `onchange` can never change.");
    }
    if props.onchange.is_some() && props.checked.is_none() {
        warn("Switch: `onchange` without `checked` can never appear on.");
    }

    let onchange = bound.emit(props.onchange);
    let toggle = move || {
        if let Some(onchange) = &onchange
            && !disabled
            && !readonly
        {
            onchange(!checked);
        }
    };

    // Enter toggles outside a form (APG) and submits inside one; a raw `<form>`
    // counts, read off the DOM (todo 660).
    let in_form = use_form_context().is_some();
    let element = use_element();
    let item = use_toolbar_item();
    let owner = use_form_owner(element, !in_form);
    let in_form = in_form || owner().is_some();
    let field = use_field()
        .inline()
        .card(card)
        .activates(element, toggle.clone())
        .enter_activates(!in_form)
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&checked))
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .aria_label(props.aria_label.as_deref())
        .attributes(&props.attributes)
        .prepare();
    // A `<label for>` names it, so there is no label id to ask the field for.
    use_name_warning(
        !props.label.is_none() || props.aria_label.is_some() || names_itself(&props.attributes),
        "Switch: no `label` or `aria_label`, so it is announced as just \"switch\".",
    );

    let states: Input<States> = field
        .states()
        .as_ref()
        .cloned()
        .unwrap_or_default()
        .with("checked", checked)
        .into();

    let style = use_cache((checked, color), |(checked, color)| {
        switch_variables(*checked, color)
    });

    let control = use_box()
        .framework_sx(&SWITCH_CONTROL_SX)
        .states(&states)
        .style(Some(style).filter(|style| !style.is_empty()))
        .prepare();
    let input = use_box()
        .framework_sx(&VISUALLY_HIDDEN_SX)
        .focus_ring(false)
        .prepare();
    let track_class = use_css(Some(&SWITCH_TRACK_SX), CssLayer::Framework);
    let thumb_class = use_css(Some(&SWITCH_THUMB_SX), CssLayer::Framework);

    let input = field
        .aria(input)
        .attr("type", "checkbox")
        .attr("role", "switch")
        .attr("checked", checked)
        .attr("data-controlled", true)
        .attr("name", bound.name().map(str::to_string))
        // A toolbar keeps a disabled item focusable, in its arrow order.
        .attr("disabled", disabled && item.is_none())
        .attr(
            "aria-disabled",
            (disabled && item.is_some()).then_some("true"),
        )
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("required", required)
        .attr("aria-label", props.aria_label)
        .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
        .attr("tabindex", item.map(ToolbarItem::tabindex))
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, props.attributes, ());

    // Decoration that carries the click: a hidden input has no hit area, and a
    // `<label>` would compete for the name.
    let track = rsx! {
        span {
            class: track_class,
            "data-slot": SwitchPart::Track.slot(),
            "aria-hidden": "true",
            // A card takes the click itself. Focuses the input as a native
            // click would, so its blur shows the rules.
            onclick: move |_| {
                if !card {
                    toggle();
                    if !disabled {
                        let _ = element.focus();
                    }
                }
            },
            span { class: thumb_class, "data-slot": SwitchPart::Thumb.slot() }
        }
    };

    field.render(
        control
            .attr("data-slot", SwitchPart::Control.slot())
            .render(
                HtmlTag::Span,
                Vec::new(),
                vec![input, track, ring_overlay()],
            ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// Todo 1601: forced colours drop the fade's contrast cue, so disabled turns `GrayText`.
    #[test]
    fn a_disabled_switch_turns_gray_text_in_forced_colours() {
        let css = Stylesheet::from(&SWITCH_CONTROL_SX).as_str().to_string();
        assert!(
            css.contains("@media (forced-colors: active)") && css.contains("color:GrayText"),
            "{css}"
        );
    }
}
