use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, HtmlTag, Input, NumberValue,
        common::{MinusIcon, PlusIcon, field_props},
        form::{FIELD_CONTROL_SX, use_bound, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::use_theme,
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::Size,
    utils::warn,
};

/// The steppers, side by side in the trailing slot, behind `steppers`. Two
/// `ActionIcon`s rather than the native spinner: `type="number"` hands back an
/// empty string for text a browser cannot parse, which is exactly the
/// in-progress text the edit buffer exists to keep.
static STEPPERS_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").align_items("center").gap("2px"));

/// The icon step a stepper takes for a field step. `ActionIcon`'s own scale
/// (16, 20, 24, 32, 40, 48px) climbs faster than a field's content box
/// (18, 20, 22, 24, 26, 28px), so the two cannot be the same step: at `md` an
/// `md` icon is 24px in a 22px box and the steppers would set the field's
/// height. Two field steps per icon step fits every step with room to spare.
const fn stepper_size(size: Size) -> Size {
    match size {
        Size::Xs | Size::Sm => Size::Xs,
        Size::Md | Size::Lg => Size::Sm,
        Size::Xl | Size::Xxl => Size::Md,
    }
}

field_props! {
    extends(input);
    pub struct NumberFieldProps<T: NumberValue> {
        /// The number in the field; strictly controlled. `None` is the empty
        /// field - nobody has typed anything yet.
        #[props(default)]
        value: Option<T>,
        /// Called with the number the caller should hold next. Silent while the
        /// buffer is not yet a number, so `"-"` and `"1."` never reach it.
        #[props(default)]
        onchange: Option<EventHandler<T>>,
        /// Rules over the number, shown once the field loses focus or its form is
        /// submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Option<T>>,
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
        /// What the field posts as. A path - `Signup::FIELDS.age()` - also
        /// binds it to the surrounding `Form`'s value when the field has no
        /// `onchange`.
        #[props(default, into)]
        name: crate::components::FieldName<Option<T>>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows the minus/plus buttons in the trailing slot. Off by default: a
        /// number is usually typed, the arrow keys step it either way, and two
        /// buttons are the most expensive thing a field can carry.
        #[props(default)]
        steppers: bool,
        /// Announced on the stepper that raises the value.
        #[props(default, into)]
        increment_label: Option<String>,
        /// Announced on the stepper that lowers it.
        #[props(default, into)]
        decrement_label: Option<String>,
    }
}

/// A numeric field over the caller's own number type, with optional steppers
/// in its trailing slot.
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
///
/// Arrow Up and Arrow Down step the value whether or not `steppers` renders
/// the buttons - the keys are what `role="spinbutton"` promises.
#[component]
pub fn NumberField<T: NumberValue>(props: NumberFieldProps<T>) -> Element {
    let theme = use_theme();

    let size = props.size.copied_or(theme.number_field.size);
    let radius = props.radius.copied_or(theme.number_field.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    // The native `readonly` stops typing; the arrow keys and the steppers are
    // ours, so they are refused here.
    let readonly = props.readonly.unwrap_or(false);
    let current = bound.value().unwrap_or(props.value);

    if props.onchange.is_none() && !bound.is_bound() {
        warn("NumberField: without `onchange` the value can never change.");
    }

    // The raw text, so that in-progress input survives a render. It is only
    // what the control shows while it still parses to the value the caller
    // holds - otherwise the caller's own value wins, which is what makes the
    // field controlled.
    let mut buffer = use_signal(String::new);
    let display = match T::parse(&buffer()) {
        Some(parsed) if Some(parsed) == current => buffer(),
        _ => current.map(|value| value.format()).unwrap_or_default(),
    };

    let min = props.min;
    let max = props.max;
    let step = props.step.unwrap_or_else(T::default_step);
    let value = current;
    let onchange = props.onchange;
    let setter = bound.setter();

    // Not `use_callback`: a stepper is a click handler, and a click handler
    // that writes a signal the field also reads is re-entrant.
    let publish = move |next: T| {
        let next = next.clamp_between(min, max);
        match (&onchange, &setter) {
            (Some(onchange), _) => onchange.call(next),
            (None, Some(setter)) => setter.set(Some(next)),
            (None, None) => {}
        }
    };
    let typed_publish = publish.clone();
    let nudge = move |up: bool| {
        if readonly {
            return;
        }
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
    let (mut decrement, mut increment, mut arrow) = (nudge.clone(), nudge.clone(), nudge);
    let publish = typed_publish;

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(props.validate.check(&current))
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

    let increment_label = props
        .increment_label
        .clone()
        .unwrap_or_else(|| "Increase".to_string());
    let decrement_label = props
        .decrement_label
        .clone()
        .unwrap_or_else(|| "Decrease".to_string());
    // Prepared either way - it is a hook, and `steppers` is a prop.
    let stepper_box = use_box().framework_sx(&STEPPERS_SX).prepare();
    let steppers = stepper_box.render(
        HtmlTag::Div,
        Vec::new(),
        // Side by side, minus then plus: two stacked carets in a field's line
        // box are a few pixels each, which is not a target anyone can hit.
        rsx! {
            ActionIcon {
                aria_label: decrement_label,
                size: ThemeAwareValue::Size(stepper_size(size)),
                // Not a tab stop: the field is, and the arrow keys do the same
                // job from there.
                tabindex: "-1",
                disabled: disabled || readonly,
                onclick: move |_| decrement(false),
                MinusIcon {}
            }
            ActionIcon {
                aria_label: increment_label,
                size: ThemeAwareValue::Size(stepper_size(size)),
                tabindex: "-1",
                disabled: disabled || readonly,
                onclick: move |_| increment(true),
                PlusIcon {}
            }
        },
    );
    let trailing = props.steppers.then_some(steppers);
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
        .attr("name", bound.name().map(str::to_string))
        .attr("value", display)
        .attr("data-controlled", true)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        .event("oninput", move |event: FormEvent| {
            let text = event.value();
            let publish = publish.clone();
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
            arrow(up);
        })
        .render(HtmlTag::Input, props.attributes, ());

    field.render(frame.render(input))
}
