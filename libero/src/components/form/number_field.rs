use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, Caption, ClassList, HtmlTag, Input, NumberValue, States,
        form::{FIELD_CONTROL_SX, FieldStatus, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, Sx, sx},
    theme::Size,
    utils::warn,
};

/// The steppers, stacked in the trailing slot. Two `ActionIcon`s rather than
/// the native spinner: `type="number"` hands back an empty string for text a
/// browser cannot parse, which is exactly the in-progress text the edit buffer
/// exists to keep.
static STEPPERS_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").align_items("center").gap("2px"));

/// What an `ActionIcon` needs to sit inside a field. Its own size comes off
/// the icon scale, which is taller than a field's text at every step - so two
/// of them would decide the field's height. `1.5em` is the control's line box,
/// so they scale with `size` and never outgrow it.
static STEPPER_SX: StaticSx = StaticSx::new(|| sx().width("1.5em").height("1.5em"));

// Hand-written rather than `field_props!`, which is not generic - as
// `SelectProps` is.
#[derive(Props, Clone, PartialEq)]
pub struct NumberFieldProps<T: NumberValue> {
    /// The number in the field; strictly controlled. `None` is the empty
    /// field - nobody has typed anything yet.
    #[props(default)]
    value: Option<T>,
    /// Called with the number the caller should hold next. Silent while the
    /// buffer is not yet a number, so `"-"` and `"1."` never reach it.
    #[props(default)]
    onchange: Option<EventHandler<T>>,
    /// Floor, enforced on typing and on the steppers alike.
    #[props(default)]
    min: Option<T>,
    /// Ceiling, same.
    #[props(default)]
    max: Option<T>,
    /// What one press of a stepper moves by. Defaults to
    /// `T::default_step()` - `1` for an integer, `1.0` for a float.
    #[props(default)]
    step: Option<T>,
    #[props(default, into)]
    placeholder: Option<String>,
    /// Announced on the stepper that raises the value.
    #[props(default, into)]
    increment_label: Option<String>,
    /// Announced on the stepper that lowers it.
    #[props(default, into)]
    decrement_label: Option<String>,
    /// The field's caption, above the control.
    #[props(default, into)]
    label: Caption,
    /// Between the label and the control. What to enter.
    #[props(default, into)]
    description: Caption,
    /// Under the control. Units, ranges, what the number means.
    #[props(default, into)]
    helper: Caption,
    /// Validation state, under the helper. A bare `&str` is an error.
    #[props(default, into)]
    status: Input<FieldStatus>,
    #[props(default, into)]
    size: Input<Size>,
    /// Corner radius, independent of `size`.
    #[props(default, into)]
    radius: Input<Size>,
    /// `None` is "not stated" - what a `Fieldset` will cascade into later.
    #[props(default)]
    disabled: Option<bool>,
    #[props(default)]
    required: Option<bool>,
    #[props(extends = GlobalAttributes, extends = input)]
    attributes: Vec<Attribute>,
    #[props(default, into)]
    class: Input<ClassList>,
    #[props(default, into)]
    sx: Input<Sx>,
    #[props(default, into)]
    states: Input<States>,
}

/// A numeric field over the caller's own number type, with steppers in its
/// trailing slot.
///
/// Every primitive number is a `NumberValue`, so `T` is normally inferred from
/// `value` and there is nothing to implement. Controlled: it renders `value`
/// and asks for a new one through `onchange`.
///
/// The control is a `<text>` input with a numeric `inputmode`, not
/// `type="number"`: a number input reports an empty string for anything a
/// browser cannot parse, which would erase `"-"` and `"1."` as they are typed.
/// The field keeps the raw text in an edit buffer instead and only publishes a
/// value the caller's type could parse.
#[component]
pub fn NumberField<T: NumberValue>(props: NumberFieldProps<T>) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.number_field.size);
    let radius = props.radius.copied_or(theme.number_field.radius);
    let disabled = props.disabled.unwrap_or(false);
    let required = props.required.unwrap_or(false);

    if props.onchange.is_none() {
        warn("NumberField: without `onchange` the value can never change.");
    }

    // The raw text, so that in-progress input survives a render. It is only
    // what the control shows while it still parses to the value the caller
    // holds - otherwise the caller's own value wins, which is what makes the
    // field controlled.
    let mut buffer = use_signal(String::new);
    let display = match T::parse(&buffer()) {
        Some(parsed) if Some(parsed) == props.value => buffer(),
        _ => props.value.map(|value| value.format()).unwrap_or_default(),
    };

    let min = props.min;
    let max = props.max;
    let step = props.step.unwrap_or_else(T::default_step);
    let value = props.value;
    let onchange = props.onchange;

    // Not `use_callback`: a stepper is a click handler, and a click handler
    // that writes a signal the field also reads is re-entrant.
    let publish = move |next: T| {
        if let Some(onchange) = &onchange {
            onchange.call(next.clamp_to(min, max));
        }
    };
    let mut nudge = move |up: bool| {
        // An empty field steps from zero, unless the type has none.
        let Some(from) = value.or_else(T::zero) else {
            return;
        };
        let next = match up {
            true => from.step_up(step),
            false => from.step_down(step),
        };
        buffer.set(String::new());
        publish(next);
    };

    let field = use_field()
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

    let increment_label = props
        .increment_label
        .clone()
        .unwrap_or_else(|| "Increase".to_string());
    let decrement_label = props
        .decrement_label
        .clone()
        .unwrap_or_else(|| "Decrease".to_string());
    let steppers = use_box().framework_sx(&STEPPERS_SX).prepare();
    let steppers = steppers.render(
        HtmlTag::Div,
        Vec::new(),
        // Side by side, minus then plus: two stacked carets in a field's line
        // box are a few pixels each, which is not a target anyone can hit.
        rsx! {
            ActionIcon {
                aria_label: decrement_label,
                sx: STEPPER_SX.clone(),
                // Not a tab stop: the field is, and the arrow keys do the same
                // job from there.
                tabindex: "-1",
                disabled,
                onclick: move |_| nudge(false),
                MinusIcon {}
            }
            ActionIcon {
                aria_label: increment_label,
                sx: STEPPER_SX.clone(),
                tabindex: "-1",
                disabled,
                onclick: move |_| nudge(true),
                PlusIcon {}
            }
        },
    );
    let trailing = Some(steppers);
    let frame = use_field_frame()
        .trailing(&trailing)
        .states(field.states())
        .prepare();

    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let input = field
        .aria(control)
        .attr_default("type", "text")
        .attr_default("inputmode", "decimal")
        // A text input driving a value in a range is a spinbutton, and that is
        // what carries the range to assistive technology - `min`/`max` on a
        // `type="text"` input mean nothing.
        .attr("role", "spinbutton")
        .attr("aria-valuenow", value.map(|value| value.to_string()))
        .attr("aria-valuemin", min.map(|min| min.to_string()))
        .attr("aria-valuemax", max.map(|max| max.to_string()))
        .attr("value", display)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("required", required)
        .event("oninput", move |event: FormEvent| {
            let text = event.value();
            if let Some(parsed) = T::parse(&text) {
                publish(parsed);
            }
            buffer.set(text);
        })
        .event("onkeydown", move |event: KeyboardEvent| {
            let up = match event.key() {
                Key::ArrowUp => true,
                Key::ArrowDown => false,
                _ => return,
            };
            // Otherwise the caret jumps to the end of the text as well.
            event.prevent_default();
            nudge(up);
        })
        .render(HtmlTag::Input, props.attributes, ());

    field.render(frame.render(input))
}

/// libero ships no icon set; a stepper with no glyph is a blank button. Kept
/// private, the same call the reveal toggle's eye makes.
#[component]
fn MinusIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "M6 12h12" }
        }
    }
}

#[component]
fn PlusIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "M12 6v12" }
            path { d: "M6 12h12" }
        }
    }
}
