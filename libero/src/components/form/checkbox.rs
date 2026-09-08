use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        HtmlTag, Input, States,
        a11y::VISUALLY_HIDDEN_SX,
        common::{
            CheckboxMarkIcon, base_color, contrast_color, field_props, fill_color, focus_ring_sx,
            ring_overlay, ring_overlay_sx, variables,
        },
        form::{use_bound, use_field},
        layout::use_box,
    },
    hooks::{use_cache, use_css, use_element, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::{CHECKBOX_BOX, CHECKBOX_RADIUS, CheckboxDefaults, ChoiceVariant, CssVar},
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
        // row. Drawn by the overlay after the box, because the focus is on
        // the input beside it.
        .selector("& > [data-ring]", ring_overlay_sx())
        .selector("& > input:focus-visible ~ [data-ring]", focus_ring_sx())
        // A card rings itself: the control stops being the overlay's
        // containing block, so the same overlay covers the card.
        .when("card", sx().position("static"))
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
        .selector("& > svg", sx().width("65%").height("65%"))
});

/// Depends on `(on, color)` alone - see the `use_cache` below.
fn checkbox_variables(on: bool, base: &ThemeAwareValue) -> String {
    let off = ThemeAwareValue::from("muted.5");
    variables()
        .with(
            CHECKBOX_BACKGROUND,
            match on {
                true => fill_color(base),
                false => Some("transparent".to_string()),
            },
        )
        .with(
            CHECKBOX_BORDER,
            match on {
                true => fill_color(base),
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
        /// Pair it with `onchange`. Left out, the box keeps its own state
        /// unless a `name` binds it to the form around it.
        #[props(default)]
        checked: Option<bool>,
        /// Draws the mixed state and reads as `aria-checked="mixed"`. Outranks
        /// `checked` visually; toggling from it gives `true`.
        #[props(default)]
        indeterminate: Option<bool>,
        /// Called with the value `checked` should take next.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        /// Rules over `checked`, shown once the checkbox loses focus or its
        /// form is submitted.
        #[props(default, into)]
        validate: crate::components::Validators<bool>,
        /// Names the checkbox when it has no `label`; `attributes` cannot,
        /// they land on the input but a caller may not want a visible label.
        /// What the field posts as. A path - `Signup::FIELDS.terms()` - also
        /// binds it to the surrounding `Form`'s value when the field has no
        /// `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<bool>,
        #[props(default, into)]
        aria_label: Option<String>,
        /// `Card` draws the checkbox as a bordered surface and makes all of
        /// it the hit area - pair it with a `description`. On the web a link
        /// or button in the label or captions keeps its own click; natively
        /// the card cannot tell, and toggles.
        #[props(default, into)]
        variant: Input<ChoiceVariant>,
    }
}

/// A checkbox, with its label beside the box and the description, helper text
/// and validation message under both.
///
/// The browser never toggles the input itself - a label click and Space are
/// taken in Rust - so the property, `:checked`, assistive tech and form
/// submission never drift apart. Pass `checked` and handle `onchange` to own
/// the state; with neither, and outside a form binding, the box keeps its own.
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let theme = use_theme();
    let color = base_color(props.color.as_ref());

    let size = props.size.copied_or(theme.checkbox.size);
    let radius = props.radius.copied_or(theme.checkbox.radius);
    let required = props.required.unwrap_or(false);
    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    let checked = bound
        .value()
        .or(props.checked)
        .or(bound.entered())
        .unwrap_or(false);
    let indeterminate = props.indeterminate.unwrap_or(false);
    // HTML's `readonly` does not apply to a checkbox, and the activation is
    // ours anyway - so it is refused here and said with `aria-readonly`,
    // which WAI-ARIA 1.2 supports on the `checkbox` role.
    let readonly = props.readonly.unwrap_or(false);
    let card = props.variant.copied_or(theme.checkbox.variant) == ChoiceVariant::Card;

    if props.checked.is_some() && props.onchange.is_none() && !bound.is_bound() {
        warn("Checkbox: `checked` without `onchange` can never change.");
    }
    if props.onchange.is_some() && props.checked.is_none() {
        warn("Checkbox: `onchange` without `checked` can never appear checked.");
    }

    let onchange = bound.emit(props.onchange);
    let next = indeterminate || !checked;
    let toggle = move || {
        if let Some(onchange) = &onchange
            && !disabled
            && !readonly
        {
            onchange(next);
        }
    };

    let field = use_field()
        .inline()
        .card(card)
        .activates(use_element(), toggle.clone())
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

    let input = field
        .aria(input)
        .attr("type", "checkbox")
        .attr("checked", checked && !indeterminate)
        .attr("data-controlled", true)
        .attr("aria-checked", indeterminate.then_some("mixed"))
        .attr("name", bound.name().map(str::to_string))
        .attr("disabled", disabled)
        .attr("aria-readonly", readonly.then_some("true"))
        .attr("required", required)
        .attr("aria-label", props.aria_label)
        // Void element - `()` costs no dynamic node.
        .render(HtmlTag::Input, props.attributes, ());

    // The box is decoration: the input owns the name, the state and the
    // keyboard. It carries the click because a visually hidden input has no
    // hit area, and a second `<label for>` around it would compete with the
    // field's own label for the accessible name. A card takes the click
    // itself, so the box leaves it to the card rather than toggling twice.
    let box_node = rsx! {
        span {
            class: box_class,
            "aria-hidden": "true",
            onclick: move |_| {
                if !card {
                    toggle();
                }
            },
            CheckboxMarkIcon { indeterminate }
        }
    };

    field.render(control.render(
        HtmlTag::Span,
        Vec::new(),
        vec![input, box_node, ring_overlay()],
    ))
}
