use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::{
        ActionIcon, HtmlTag, Input, NumberValue,
        common::{MinusIcon, PlusIcon, field_props, navigation_chord},
        form::{
            FIELD_CONTROL_SX, LiveControl, slot_icon_size, use_bound, use_field, use_field_frame,
        },
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

/// Each stepper's 24px hit area stops at the middle of the 2px gap, so the two
/// meet instead of overlapping; the spare width goes outwards.
static DECREMENT_SX: StaticSx =
    StaticSx::new(|| sx().selector("::before", sx().left("calc(100% - 23px)").right("-1px")));
static INCREMENT_SX: StaticSx =
    StaticSx::new(|| sx().selector("::before", sx().left("-1px").right("calc(100% - 23px)")));

field_props! {
    extends(input);
    pub struct NumberFieldProps<T: NumberValue> {
        /// The number in the field; strictly controlled. `None` is the empty
        /// field - nobody has typed anything yet.
        #[props(default)]
        value: Option<T>,
        /// Called with the number the caller should hold next; `None` once the
        /// field is emptied. Silent while the text is not yet a number, so
        /// `"-"` and `"1."` never reach it.
        #[props(default)]
        onchange: Option<EventHandler<Option<T>>>,
        /// Rules over the number, shown once the field loses focus or its form is
        /// submitted.
        #[props(default, into)]
        validate: crate::components::Validators<Option<T>>,
        /// Floor. Steps clamp to it; typed text below it clamps once the field
        /// is left or Enter is pressed.
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
/// value the caller's type could parse, or `None` for an emptied field. On
/// commit (leaving the field, Enter) the text becomes the value's canonical
/// form: an out-of-range number clamps, text that never parsed reverts.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::NumberField;
/// # fn app() -> Element {
/// let mut quantity = use_signal(|| Some(1u32));
/// rsx! {
///     NumberField {
///         label: "Quantity",
///         value: quantity(),
///         onchange: move |next| quantity.set(next),
///     }
/// }
/// # }
/// ```
///
/// Arrow Up and Arrow Down step the value whether or not `steppers` renders
/// the buttons, Page Up and Page Down ten steps - the keys are what
/// `role="spinbutton"` promises.
#[component]
pub fn NumberField<T: NumberValue>(props: NumberFieldProps<T>) -> Element {
    let mut props = props;
    let value = props.value.take();
    let mut live = use_signal(|| value);
    if *live.peek() != value {
        live.set(value);
    }
    rsx! { NumberFieldShell::<T> { live, field: props } }
}

/// Everything but the number, so a step skips it and redraws the control.
#[component]
fn NumberFieldShell<T: NumberValue>(
    live: Signal<Option<T>>,
    field: NumberFieldProps<T>,
) -> Element {
    let props = field;
    let theme = use_theme();

    let size = props.size.copied_or(theme.number_field.size);
    let radius = props.radius.copied_or(theme.number_field.radius);
    let required = props.required.unwrap_or(false);

    let bound = use_bound(&props.name, props.onchange.is_some());
    let disabled = bound.disabled(props.disabled);
    // The native `readonly` stops typing; the arrow keys and the steppers are
    // ours, so they are refused here.
    let readonly = props.readonly.unwrap_or(false);
    let bound_value = bound.value();
    // Not a subscription: only rules and the control read the number.
    let current = move || bound_value.unwrap_or_else(|| *live.peek());

    if props.onchange.is_none() && !bound.is_bound() {
        warn("NumberField: without `onchange` the value can never change.");
    }

    // The raw text, so that in-progress input survives a render. See `Edit`.
    let mut buffer = use_signal(|| None::<Edit<T>>);

    let min = props.min;
    let max = props.max;
    let step = props.step.unwrap_or_else(T::default_step);
    let onchange = props.onchange;
    let setter = bound.setter();

    let publish = move |next: Option<T>| {
        let next = next.map(|next| next.clamp_between(min, max));
        match (&onchange, &setter) {
            (Some(onchange), _) => onchange.call(next),
            (None, Some(setter)) => setter.set(next),
            (None, None) => {}
        }
    };
    let typed_publish = publish.clone();
    // One identity across renders, so `Steppers` can skip. Safe as a
    // `use_callback`: nothing in it focuses or clicks, so it cannot re-enter.
    let nudge = use_callback(move |steps: i32| {
        if readonly {
            return;
        }
        // An empty field steps from zero, unless the type has none.
        let Some(mut next) = current().or_else(T::zero) else {
            return;
        };
        for _ in 0..steps.unsigned_abs() {
            next = match steps > 0 {
                true => next.step_up(step),
                false => next.step_down(step),
            };
        }
        buffer.set(None);
        publish(Some(next));
    });
    let publish = typed_publish;
    let commit = publish.clone();
    // Rules read the number here, so only a validated field redraws per step.
    let rules = (!props.validate.is_empty())
        .then(|| {
            props
                .validate
                .check(&bound_value.unwrap_or_else(|| live.cloned()))
        })
        .flatten();

    let field = use_field()
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&props.status)
        .rules(rules)
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

    let trailing = props.steppers.then(|| {
        rsx! {
            Steppers {
                size,
                disabled: disabled || readonly,
                increment_label: props.increment_label.clone(),
                decrement_label: props.decrement_label.clone(),
                nudge,
            }
        }
    });
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
        .attr("aria-valuemin", min.map(|min| min.to_string()))
        .attr("aria-valuemax", max.map(|max| max.to_string()))
        .attr("name", bound.name().map(str::to_string))
        .attr("data-controlled", true)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        .event("oninput", move |event: FormEvent| {
            let text = event.value();
            let publish = publish.clone();
            let mut held = current();
            // Out of range waits for the commit: clamping the "2" of "25"
            // against a floor of 10 would make 25 untypable.
            if text.trim().is_empty() {
                held = None;
                publish(None);
            } else if let Some(parsed) = T::parse(&text)
                && parsed.clamp_between(min, max) == parsed
            {
                held = Some(parsed);
                publish(held);
            }
            buffer.set(Some(Edit { text, held }));
        })
        // Fires when the field is left or Enter commits an edit. Dropping the
        // buffer redraws the canonical text, which also rewrites the DOM.
        .event("onchange", move |event: FormEvent| {
            if let Some(parsed) = T::parse(&event.value())
                && parsed.clamp_between(min, max) != parsed
            {
                commit(Some(parsed));
            }
            buffer.set(None);
        })
        .event("onkeydown", move |event: KeyboardEvent| {
            // Ctrl+PageDown switches tabs, Ctrl+ArrowUp moves the caret.
            if navigation_chord(&event).is_some() {
                return;
            }
            let steps = match event.key() {
                Key::ArrowUp => 1,
                Key::ArrowDown => -1,
                Key::PageUp => 10,
                Key::PageDown => -10,
                _ => return,
            };
            // Otherwise the caret jumps to the end of the text as well.
            event.prevent_default();
            nudge.call(steps);
        });
    let attributes = props.attributes;
    let draw = Rc::new(move || {
        let current = bound_value.unwrap_or_else(|| live.cloned());
        let display = match &*buffer.read() {
            Some(edit) if edit.shows(current) => edit.text.clone(),
            _ => current.map(|value| value.format()).unwrap_or_default(),
        };
        input
            .clone()
            .attr("aria-valuenow", current.map(|value| value.to_string()))
            .attr("value", display)
            .render(HtmlTag::Input, attributes.clone(), ())
    });

    field.render(frame.render(rsx! { LiveControl { draw } }))
}

/// Text typed since the last commit, and the value the caller holds once it
/// has taken it in. The text shows while it parses to the caller's value, or
/// while that value has not moved since (`"-"`, an out-of-range `"5"`). So the
/// rendered `value` tracks the DOM text, and a commit that drops the buffer
/// always writes the canonical text back.
#[derive(Clone, PartialEq)]
struct Edit<T> {
    text: String,
    held: Option<T>,
}

impl<T: NumberValue> Edit<T> {
    fn shows(&self, current: Option<T>) -> bool {
        self.held == current || T::parse(&self.text) == current
    }
}

/// The two stepper buttons, a scope of their own: every prop compares equal
/// while the value moves, so a press redraws the field and not the buttons.
#[component]
fn Steppers(
    size: Size,
    disabled: bool,
    increment_label: Option<String>,
    decrement_label: Option<String>,
    nudge: Callback<i32>,
) -> Element {
    // A press keeps the focus where it was, as a native spinner does: on the
    // field, its caret and its arrow keys.
    let stepper_box = use_box()
        .framework_sx(&STEPPERS_SX)
        .prepare()
        .event("onmousedown", |event: MouseEvent| event.prevent_default());
    stepper_box.render(
        HtmlTag::Div,
        Vec::new(),
        // Side by side, minus then plus: two stacked carets in a field's line
        // box are a few pixels each, which is not a target anyone can hit.
        rsx! {
            ActionIcon {
                aria_label: decrement_label.unwrap_or_else(|| "Decrease".to_string()),
                size: ThemeAwareValue::Size(slot_icon_size(size)),
                sx: &DECREMENT_SX,
                // Not a tab stop: the field is, and the arrow keys do the same
                // job from there.
                tabindex: "-1",
                disabled,
                onclick: move |_| nudge.call(-1),
                MinusIcon {}
            }
            ActionIcon {
                aria_label: increment_label.unwrap_or_else(|| "Increase".to_string()),
                size: ThemeAwareValue::Size(slot_icon_size(size)),
                sx: &INCREMENT_SX,
                tabindex: "-1",
                disabled,
                onclick: move |_| nudge.call(1),
                PlusIcon {}
            }
        },
    )
}
