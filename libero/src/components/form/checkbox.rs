use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    CssLayer,
    components::{
        accessibility::VISUALLY_HIDDEN_SX,
        common::{
            Glyph, HtmlTag, Input, Part, States, TOOLBAR_ITEM, ToolbarItem, base_color,
            contrast_color, fill_color, focus_ring_sx, names_itself, ring_overlay, ring_overlay_sx,
            use_name_warning, use_toolbar_item, variables,
        },
        form::{field_parts_enum, field_props, use_bound, use_field},
        layout::use_box,
    },
    context::IconSlot,
    hooks::{use_cache, use_css, use_element, use_theme},
    platform::ElementApi,
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
        // On the control rather than the box, so `disabled` below reaches it.
        .cursor("pointer")
        // The ring hugs the box, drawn by the overlay: the focus is on the
        // input beside it.
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
        .transition("background 150ms ease, border-color 150ms ease")
        .selector("& > svg", sx().width("65%").height("65%"))
});

/// Depends on `(on, color)` alone - see the `use_cache` below.
fn checkbox_variables(on: bool, base: &ThemeAwareValue) -> String {
    // The outline is all an unchecked box shows: muted.6 is the first step at
    // 3:1 on the page in both schemes (WCAG 1.4.11, todo 490).
    let off = ThemeAwareValue::from("muted.6");
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

field_parts_enum! {
    /// [`Checkbox`]'s inner parts, for its `parts` prop: a field's, the
    /// control and its box.
    pub enum CheckboxPart {
        /// Holds the hidden input and the box, beside the label.
        Control = "control" => "& > [data-slot='control']",
        /// The drawn square and its mark.
        Box = "box" => "& > [data-slot='control'] > [data-slot='box']",
    }
}

field_props! {
    parts(CheckboxPart);
    extends(input);
    pub struct CheckboxProps {
        /// The box's colour when checked.
        #[props(default, into)]
        color: Input<ThemeAwareValue>,
        /// Pair it with `onchange`; left out, the box keeps its own state.
        #[props(default)]
        checked: Option<bool>,
        /// Draws and announces the mixed state; toggling from it gives `true`.
        #[props(default)]
        indeterminate: Option<bool>,
        /// Called with the value `checked` should take next.
        #[props(default)]
        onchange: Option<EventHandler<bool>>,
        /// Rules over `checked`, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<bool>,
        /// What the field posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<bool>,
        /// Names the checkbox when it has no `label`.
        #[props(default, into)]
        aria_label: Option<String>,
        /// `Card` draws a bordered surface that is all hit area.
        #[props(default, into)]
        variant: Input<ChoiceVariant>,
    }
}

/// A checkbox with its label, description and validation message.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Checkbox;
/// # fn app() -> Element {
/// let mut agreed = use_signal(|| false);
/// rsx! {
///     Checkbox {
///         label: "I accept the terms",
///         checked: agreed(),
///         onchange: move |next| agreed.set(next),
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/form/checkbox>
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
    // HTML `readonly` does not apply to a checkbox: refused here, announced
    // with `aria-readonly`.
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

    let element = use_element();
    let item = use_toolbar_item();
    // A browser reads a native checkbox's mixed state from the property alone;
    // elsewhere `aria-checked="mixed"` below carries it.
    use_effect(use_reactive!(|indeterminate| {
        let _ = element.set_indeterminate(indeterminate);
    }));
    let field = use_field()
        .inline()
        .card(card)
        .activates(element, toggle.clone())
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
        .attributes(&props.attributes)
        .prepare();
    use_name_warning(
        !props.label.is_none() || props.aria_label.is_some() || names_itself(&props.attributes),
        "Checkbox: no `label` or `aria_label`, so it is announced as just \"checkbox\".",
    );

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
    // second `<label for>` would compete for the name. A card takes the click itself.
    let (slot, icon) = match indeterminate {
        true => (IconSlot::CheckboxIndeterminate, lucide::minus::outlined),
        false => (IconSlot::CheckboxCheck, lucide::check::outlined),
    };
    let box_node = rsx! {
        span {
            class: box_class,
            "data-slot": CheckboxPart::Box.slot(),
            "aria-hidden": "true",
            // Focuses the input as a native click would, so its blur shows the rules.
            onclick: move |_| {
                if !card {
                    toggle();
                    if !disabled {
                        let _ = element.focus();
                    }
                }
            },
            // Heavier than other glyphs: drawn at 65% of a small box.
            Glyph { slot, icon, stroke_width: "3" }
        }
    };

    field.render(
        control
            .attr("data-slot", CheckboxPart::Control.slot())
            .render(
                HtmlTag::Span,
                Vec::new(),
                vec![input, box_node, ring_overlay()],
            ),
    )
}
