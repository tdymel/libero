use std::{any::TypeId, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::ActionIcon,
        common::{
            Glyph, HtmlTag, Input, NumberValue, TOOLBAR_ITEM, ToolbarItem, navigation_chord,
            use_no_toolbar, use_toolbar_item,
        },
        form::{
            FIELD_CONTROL_SX, LiveControl, field_props, slot_button_sx, slot_icon_size, use_bound,
            use_field, use_field_frame,
        },
        layout::use_box,
    },
    context::IconSlot,
    hooks::{current_localization, use_formats, use_theme},
    sx::{StaticSx, ThemeAwareValue, sx},
    theme::Size,
    utils::warn,
};

/// The steppers in the trailing slot. Not the native spinner: `type="number"`
/// empties text it cannot parse, the in-progress text the edit buffer keeps.
static STEPPERS_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").align_items("center").gap("2px"));

/// Each 24px hit area stops mid-gap so the two meet; spare width goes outwards.
static DECREMENT_SX: StaticSx = StaticSx::new(|| {
    slot_button_sx()
        .selector("::before", sx().left("calc(100% - 23px)").right("-1px"))
        .rtl(sx().selector("&::before", sx().left("-1px").right("calc(100% - 23px)")))
});
static INCREMENT_SX: StaticSx = StaticSx::new(|| {
    slot_button_sx()
        .selector("::before", sx().left("-1px").right("calc(100% - 23px)"))
        .rtl(sx().selector("&::before", sx().left("calc(100% - 23px)").right("-1px")))
});

field_props! {
    extends(input);
    pub struct NumberFieldProps<T: NumberValue> {
        /// The number in the field; strictly controlled. `None` is empty.
        #[props(default)]
        value: Option<T>,
        /// Called with the next number; `None` once emptied. Silent on `"-"` or `"1."`.
        #[props(default)]
        onchange: Option<EventHandler<Option<T>>>,
        /// Rules over the number, shown on blur or submit.
        #[props(default, into)]
        validate: crate::components::form::Validators<Option<T>>,
        /// Floor. Steps clamp to it; typed text clamps on commit.
        #[props(default)]
        min: Option<T>,
        /// Ceiling, same.
        #[props(default)]
        max: Option<T>,
        /// What one stepper press moves by. Defaults to `T::default_step()`.
        #[props(default)]
        step: Option<T>,
        /// What the field posts as. A path also binds it to the surrounding `Form`.
        #[props(default, into)]
        name: crate::components::form::FieldName<Option<T>>,
        #[props(default, into)]
        placeholder: Option<String>,
        /// Shows the minus/plus buttons. The arrow keys step either way.
        #[props(default)]
        steppers: bool,
        /// Names the raising stepper. Defaults to `number_field.increase`.
        #[props(default, into)]
        increment_label: Option<String>,
        /// Names the lowering stepper. Defaults to `number_field.decrease`.
        #[props(default, into)]
        decrement_label: Option<String>,
    }
}

/// A numeric field over the caller's own number type, with optional steppers.
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
/// Docs: <https://libero-ui.dev/form/number-field>
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
    let item = use_toolbar_item();
    use_no_toolbar();
    // A toolbar keeps a disabled item focusable, in its arrow order: read-only instead.
    let soft_disabled = disabled && item.is_some();
    let readonly = props.readonly.unwrap_or(false) || soft_disabled;
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
    let separator = float_separator::<T>(use_formats().decimal_separator);
    let kind = use_hook(Keypad::<T>::of);

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
        // Typed text steps from itself, out of range too, as a native spinner
        // does. An empty field steps from zero, unless the type has none.
        let typed = buffer
            .peek()
            .as_ref()
            .filter(|edit| edit.shows(current(), separator))
            .and_then(|edit| parse::<T>(&edit.text, separator));
        let Some(mut next) = typed.or_else(current).or_else(T::zero) else {
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
        .parts(&props.parts)
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
        .placeholder(props.placeholder.as_deref())
        .prepare();

    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let input = field
        .aria(control)
        .attr_default("type", "text")
        .attr_default("inputmode", keypad(kind, min))
        // The role carries the range: `min`/`max` mean nothing on `type="text"`.
        .attr("role", "spinbutton")
        .attr("aria-valuemin", min.map(|min| min.to_string()))
        .attr("aria-valuemax", max.map(|max| max.to_string()))
        .attr("name", bound.name().map(str::to_string))
        .attr("data-controlled", true)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled && !soft_disabled)
        .attr("aria-disabled", soft_disabled.then_some("true"))
        .attr("readonly", readonly)
        .attr("required", required)
        .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
        .attr("tabindex", item.map(ToolbarItem::tabindex))
        .event("oninput", move |event: FormEvent| {
            let text = event.value();
            let publish = publish.clone();
            let mut held = current();
            // Out of range waits for the commit: clamping the "2" of "25"
            // against a floor of 10 would make 25 untypable.
            if text.trim().is_empty() {
                held = None;
                publish(None);
            } else if let Some(parsed) = parse::<T>(&text, separator)
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
            if let Some(parsed) = parse::<T>(&event.value(), separator)
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
            Some(edit) if edit.shows(current, separator) => edit.text.clone(),
            _ => current
                .map(|value| match separator {
                    Some(separator) => value.format().replacen('.', separator, 1),
                    None => value.format(),
                })
                .unwrap_or_default(),
        };
        input
            .clone()
            .attr("aria-valuenow", current.map(|value| value.to_string()))
            .attr("value", display)
            .render(HtmlTag::Input, attributes.clone(), ())
    });

    field.render(frame.render(rsx! { LiveControl { draw } }))
}

/// Text typed since the last commit, shown while it parses to the caller's
/// value or that value has not moved. Dropping it writes the canonical text.
#[derive(Clone, PartialEq)]
struct Edit<T> {
    text: String,
    held: Option<T>,
}

impl<T: NumberValue> Edit<T> {
    fn shows(&self, current: Option<T>, separator: Option<&str>) -> bool {
        self.held == current || parse::<T>(&self.text, separator) == current
    }
}

/// The formats' decimal separator where it is not `.`, for the built-in floats
/// only: a custom `NumberValue` owns its text, which may already be localized.
fn float_separator<T: 'static>(separator: &'static str) -> Option<&'static str> {
    let float =
        TypeId::of::<T>() == TypeId::of::<f64>() || TypeId::of::<T>() == TypeId::of::<f32>();
    (float && separator != ".").then_some(separator)
}

/// What `T` can hold, read once per field from its parse.
#[derive(Clone, Copy)]
struct Keypad<T> {
    signed: bool,
    fractional: bool,
    zero: Option<T>,
}

impl<T: NumberValue> Keypad<T> {
    fn of() -> Self {
        Self {
            signed: T::parse("-1").is_some(),
            fractional: T::parse("0.5").is_some(),
            zero: T::zero(),
        }
    }
}

/// The `inputmode`: iOS's `numeric` and `decimal` keypads have no minus, and `numeric` no point.
fn keypad<T: NumberValue>(kind: Keypad<T>, min: Option<T>) -> &'static str {
    let negatives = kind.signed && min.is_none_or(|min| kind.zero.is_none_or(|zero| min < zero));
    match (negatives, kind.fractional) {
        (true, _) => "text",
        (false, true) => "decimal",
        (false, false) => "numeric",
    }
}

/// `T::parse`, taking the separator as well as `.`: a German user on a US
/// layout or a numpad types `1.5`.
fn parse<T: NumberValue>(text: &str, separator: Option<&str>) -> Option<T> {
    match separator {
        Some(separator) => T::parse(&text.replacen(separator, ".", 1)),
        None => T::parse(text),
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
    let labels = current_localization().number_field;
    stepper_box.render(
        HtmlTag::Div,
        Vec::new(),
        // Side by side, minus then plus: two stacked carets in a field's line
        // box are a few pixels each, which is not a target anyone can hit.
        rsx! {
            ActionIcon {
                aria_label: decrement_label.unwrap_or_else(|| labels.decrease.to_string()),
                size: ThemeAwareValue::Size(slot_icon_size(size)),
                sx: &DECREMENT_SX,
                // Not a tab stop: the field is, and the arrow keys do the same
                // job from there.
                tabindex: "-1",
                disabled,
                onclick: move |_| nudge.call(-1),
                Glyph { slot: IconSlot::Minus, icon: lucide::minus::outlined }
            }
            ActionIcon {
                aria_label: increment_label.unwrap_or_else(|| labels.increase.to_string()),
                size: ThemeAwareValue::Size(slot_icon_size(size)),
                sx: &INCREMENT_SX,
                tabindex: "-1",
                disabled,
                onclick: move |_| nudge.call(1),
                Glyph { slot: IconSlot::Plus, icon: lucide::plus::outlined }
            }
        },
    )
}
