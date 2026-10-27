mod spinner;

use std::{any::TypeId, borrow::Cow, rc::Rc};

use dioxus::{dioxus_core::AttributeValue, prelude::*};
use pictogram_icons_lucide as lucide;

use spinner::{Look, NumberSpinner, Spin};

use crate::{
    components::{
        accessibility::use_announcer,
        buttons::{ACTION_ICON_COMPACT, ActionIcon},
        common::{
            COARSE_POINTER, Glyph, HtmlTag, Input, NumberValue, ScaleOrCss, TOOLBAR_ITEM,
            ToolbarItem, aria_label_of, navigation_chord, use_no_toolbar, use_toolbar_item,
        },
        form::{
            FieldError, FieldStatus, FormScope, LiveControl, LiveSlot, Parse, Typed,
            field_control_sx, field_props, slot_button_sx, slot_icon_size, use_bound, use_field,
            use_field_frame, use_live_slot, use_prefill, word,
        },
        layout::use_box,
    },
    context::IconSlot,
    hooks::{
        current_localization, use_element, use_form_owner, use_formats, use_silent_focus_in,
        use_silent_focus_out_of, use_theme,
    },
    localization::{Formats, Localization},
    platform::{ElementApi, next_task},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::Size,
    utils::{ascii_digit, ungrouped, warn},
};

/// The steppers in the trailing slot. Not the native spinner: `type="number"`
/// empties text it cannot parse, the in-progress text the edit buffer keeps.
static STEPPERS_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").align_items("center").gap("2px"));

/// The input is `dir="ltr"`; its text still sits at the page's start.
static NUMBER_CONTROL_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
        .width("100%")
        .text_align("start")
        .rtl(sx().text_align("right"))
});

/// A stepper's hit area, `side` px wide, stops mid-gap so the two meet; spare width goes
/// outwards. `towards_start` reaches left in LTR.
fn stepper_hit_area_sx(side: u32, towards_start: bool, rtl: bool) -> Sx {
    let reach = format!("calc(100% - {}px)", side - 1);
    let (left, right) = match towards_start != rtl {
        true => (reach, "-1px".to_string()),
        false => ("-1px".to_string(), reach),
    };
    sx().left(left).right(right)
}

fn stepper_sx(towards_start: bool) -> Sx {
    let area = |side, rtl| {
        sx().selector(
            if rtl { "&::before" } else { "::before" },
            stepper_hit_area_sx(side, towards_start, rtl),
        )
    };
    slot_button_sx()
        .and(area(24, false))
        .rtl(area(24, true))
        .when(
            ACTION_ICON_COMPACT,
            sx().media(COARSE_POINTER, area(44, false).rtl(area(44, true))),
        )
}

static DECREMENT_SX: StaticSx = StaticSx::new(|| stepper_sx(true));
static INCREMENT_SX: StaticSx = StaticSx::new(|| stepper_sx(false));

field_props! {
    extends(input);
    pub struct NumberFieldProps<T: NumberValue> {
        /// The number in the field; strictly controlled. `None` is empty.
        #[props(default)]
        value: Option<T>,
        /// Called with the next number; `None` once emptied. Silent on `"-"`; a float `"1."` arrives as `1`.
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
        /// Writes the thousands grouped (`12,500`) while the field is not focused; typing
        /// sees the bare number. Built-in number types only: a custom one writes itself.
        /// A WebView cannot read or set the caret: focusing the field may send it to the end.
        #[props(default)]
        group_digits: bool,
        /// How many digits the fraction keeps: typed ones past it are cut, a step or a
        /// caller's value is rounded. `f32` and `f64` only.
        #[props(default)]
        decimals: Option<Decimals>,
        /// Inside the frame, before the control: a currency sign.
        #[props(default, into)]
        leading: Option<Element>,
        /// Inside the frame, after the control and before the steppers: a unit or `%`.
        #[props(default, into)]
        trailing: Option<Element>,
        /// `leading` is text that describes the value (`aria-describedby`).
        #[props(default)]
        describe_leading: bool,
        /// `trailing` is text that describes the value, e.g. a unit.
        #[props(default)]
        describe_trailing: bool,
        /// Names the raising stepper. Defaults to `number_field.increase`.
        #[props(default, into)]
        increment_label: Option<String>,
        /// Names the lowering stepper. Defaults to `number_field.decrease`.
        #[props(default, into)]
        decrement_label: Option<String>,
        /// What a tap under a finger opens: the soft keyboard, or a spinner that steps by `step`.
        #[props(default)]
        touch_input: TouchInput,
    }
}

/// What a tap on a [`NumberField`] opens under a finger. A mouse always types.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{NumberField, TouchInput};
/// # fn app() -> Element {
/// let mut minutes = use_signal(|| Some(30u32));
/// rsx! {
///     // A tap spins in steps of 5; the spinner's keyboard button types instead.
///     NumberField {
///         label: "Break",
///         trailing: rsx! { "min" },
///         describe_trailing: true,
///         step: 5,
///         touch_input: TouchInput::Spin,
///         value: minutes(),
///         onchange: move |next| minutes.set(next),
///     }
/// }
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TouchInput {
    /// The soft keyboard, as any text field.
    #[default]
    Keyboard,
    /// A spin column, as a digital clock's: below `sm` in a sheet, a popover from `sm` up.
    /// Every step is sent at once; Done, Escape, Back and a tap outside only close it.
    Spin,
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
/// A price and a share, the sign and `%` outside the typed text:
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Decimals, NumberField};
/// # fn app() -> Element {
/// let mut price = use_signal(|| Some(12.5));
/// let mut share = use_signal(|| Some(40.0));
/// rsx! {
///     NumberField {
///         label: "Price",
///         value: price(),
///         onchange: move |next| price.set(next),
///         leading: rsx! { "$" },
///         describe_leading: true,
///         decimals: Decimals::Exactly(2),
///         group_digits: true,
///     }
///     NumberField {
///         label: "Share",
///         value: share(),
///         onchange: move |next| share.set(next),
///         trailing: rsx! { "%" },
///         describe_trailing: true,
///         decimals: Decimals::AtMost(1),
///         min: 0.0,
///         max: 100.0,
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
    let leading = use_live_slot(props.leading.take());
    let trailing = use_live_slot(props.trailing.take());
    rsx! { NumberFieldShell::<T> { live, leading, trailing, field: props } }
}

/// How many digits a [`NumberField`]'s fraction keeps.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{Decimals, NumberField};
/// # fn app() -> Element {
/// let mut total = use_signal(|| Some(1_250.5f64));
/// let mut share = use_signal(|| Some(12.5f64));
/// rsx! {
///     // Shows `1,250.50` while not typing.
///     NumberField {
///         label: "Total",
///         leading: rsx! { "$" },
///         decimals: Decimals::Exactly(2),
///         group_digits: true,
///         value: total(),
///         onchange: move |next| total.set(next),
///     }
///     NumberField {
///         label: "Discount",
///         trailing: rsx! { "%" },
///         decimals: Decimals::AtMost(1),
///         min: 0.0,
///         max: 100.0,
///         value: share(),
///         onchange: move |next| share.set(next),
///     }
/// }
/// # }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decimals {
    /// Up to this many: `12.5` stays `12.5`.
    AtMost(u8),
    /// Always this many, padded while not typing: `12.5` shows `12.50`.
    Exactly(u8),
}

impl Decimals {
    fn places(self) -> usize {
        match self {
            Self::AtMost(places) | Self::Exactly(places) => places.into(),
        }
    }

    /// Rust's text of a number (`12.345`) rounded to these places; other text stays.
    fn write(self, text: &str) -> String {
        let Ok(number) = text.parse::<f64>() else {
            return text.to_string();
        };
        let fixed = format!("{number:.*}", self.places());
        match self {
            Self::Exactly(_) => fixed,
            Self::AtMost(_) if fixed.contains('.') => fixed
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_string(),
            Self::AtMost(_) => fixed,
        }
    }

    /// A bound's text cut to these places toward the range: down for `max`, up for `min`.
    fn toward(self, text: &str, up: bool) -> Option<String> {
        let scale = 10f64.powi(self.places() as i32);
        let scaled = text.parse::<f64>().ok()? * scale;
        let cut = if up { scaled.ceil() } else { scaled.floor() };
        Some(format!("{:.*}", self.places(), cut / scale))
    }

    /// A step these places cannot show, as text: rounded to them, at least one unit.
    fn coarse_step(self, text: &str) -> Option<String> {
        let step = text.parse::<f64>().ok()?;
        let places = self.places();
        let unit = 10f64.powi(-(places as i32));
        let shown = format!("{step:.places$}").parse::<f64>().ok()?;
        (shown != step).then(|| {
            let units = (step.abs() / unit).round().max(1.0);
            format!("{:.places$}", units * unit)
        })
    }

    /// Typed text cut after these places, where it has more; `mark` is the decimal mark.
    /// A `.` stands in for it unless `.` groups, where `1.234` is a whole number (4358).
    fn cut(self, text: &str, mark: &str, group: Option<&str>) -> Option<String> {
        let (at, mark) = match text.find(mark) {
            Some(at) => (at, mark),
            None if group == Some(".") => return None,
            None => (text.find('.')?, "."),
        };
        let (whole, fraction) = (&text[..at], &text[at + mark.len()..]);
        let digits = fraction
            .chars()
            .take_while(|c| ascii_digit(*c).is_some())
            .count();
        let places = self.places();
        (digits > places).then(|| {
            let kept: String = fraction.chars().take(places).collect();
            let rest: String = fraction.chars().skip(digits).collect();
            match places {
                0 => format!("{whole}{rest}"),
                _ => format!("{whole}{mark}{kept}{rest}"),
            }
        })
    }
}

/// Everything but the number and the slots' content, so a step skips it and redraws the control.
#[component]
fn NumberFieldShell<T: NumberValue>(
    live: Signal<Option<T>>,
    leading: Option<Signal<Option<Element>>>,
    trailing: Option<Signal<Option<Element>>>,
    field: NumberFieldProps<T>,
) -> Element {
    let props = field;
    let unit = trailing;
    let leading = leading.map(|content| rsx! { LiveSlot { content } });
    let caller_trailing = trailing.map(|content| rsx! { LiveSlot { content } });
    let theme = use_theme();

    let size = props.size.copied_or(theme.number_field.size);
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.number_field.radius);
    let required = crate::components::form::fieldset_required(props.required);

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

    // Text that is no number, kept by the form over an unmount, as `DateField` keeps its own (4359).
    let scope = try_use_context::<FormScope>();
    let kept = use_hook(|| {
        let full = bound.name()?;
        scope?.typed(full).filter(|_| current().is_none())
    });
    // The raw text, so that in-progress input survives a render. See `Edit`.
    let mut buffer = use_signal(|| {
        kept.as_ref().map(|typed| Edit {
            text: typed.text.clone(),
            held: None,
        })
    });
    // Why the last commit was refused; cleared by the next keystroke, step, reset or outside value.
    let mut rejected = use_signal(|| kept.map(|typed| typed.error));
    // A form reset drops the typed text, which `-` or `.` would outlive on a `None` value.
    let anchor = use_element();
    let owner = use_form_owner(anchor, scope.is_none());
    let resets = scope.map_or(0, |form| form.resets()) + owner().unwrap_or(0);
    // Not on mount, which may restore text the form kept.
    let seen = use_hook(|| Rc::new(std::cell::Cell::new(resets)));
    use_effect(use_reactive!(|resets| {
        if seen.replace(resets) == resets {
            return;
        }
        if buffer.peek().is_some() {
            buffer.set(None);
        }
        if rejected.peek().is_some() {
            rejected.set(None);
        }
    }));

    let min = props.min;
    let max = props.max;
    let step = props.step.unwrap_or_else(T::default_step);
    let onchange = props.onchange;
    let setter = bound.setter();
    let formats = use_formats();
    let separator = Separators::of::<T>(formats);
    let grouped = props.group_digits && built_in::<T>();
    let decimals = props.decimals.filter(|_| float::<T>());
    if props.decimals.is_some() && decimals.is_none() {
        warn("NumberField: `decimals` takes `f32` or `f64` only.");
    }
    // A step finer than the places would round back to where it started: 2 + 0.5 shows 2.
    let coarse = decimals.and_then(|decimals| decimals.coarse_step(&step.format()));
    use_hook(|| {
        if coarse.is_some() {
            warn("NumberField: `step` is finer than `decimals`; it steps by one shown place.");
        }
    });
    let step = coarse.and_then(|text| T::parse(&text)).unwrap_or(step);
    let rounded = move |value: T| rounded_within(value, decimals, min, max);
    let mut focused = use_signal(|| false);
    // Blitz's Tab and libero's `focus()` fire no `focusin`/`focusout`.
    let focus = move |now: bool| {
        let mut focused = focused;
        if *focused.peek() != now {
            focused.set(now);
        }
    };
    use_silent_focus_in(anchor, move || focus(grouped));
    use_silent_focus_out_of(anchor, move || focus(false));
    let kind = use_hook(Keypad::<T>::of);
    let mut spin = use_signal(Spin::default);
    let spins = props.touch_input == TouchInput::Spin;
    let spin_opens = spins && !disabled && !readonly;
    // A field turned disabled or read-only while open (a form submitting) closes it.
    use_effect(use_reactive!(|spin_opens| {
        if !spin_opens && spin.peek().open {
            spin.write().open = false;
        }
    }));

    // Refused text resets a bound value, as a mapped field's does; the form keeps the text.
    let full = bound.name().map(str::to_string);
    let resets_value = setter.is_some() && onchange.is_none();
    let keep = move |typed: Option<Typed>| {
        let (Some(mut scope), Some(full)) = (scope, &full) else {
            return;
        };
        match typed {
            Some(typed) if resets_value => scope.keep_typed(full, typed),
            _ => scope.drop_typed(full),
        }
    };
    // A value set from outside replaces the refused text, and its error goes with it (4382).
    // The text goes too, or a later outside `None` would draw it again unflagged (4387).
    let outside_keep = keep.clone();
    use_effect(use_reactive!(|bound_value| {
        let value = bound_value.unwrap_or_else(|| live.cloned());
        let shown = buffer
            .peek()
            .as_ref()
            .is_some_and(|edit| edit.shows(value, separator));
        if rejected.peek().is_some() && !shown {
            rejected.set(None);
            buffer.set(None);
            outside_keep(None);
        }
    }));
    let unrefuse = {
        let keep = keep.clone();
        move || {
            let mut rejected = rejected;
            if rejected.peek().is_some() {
                rejected.set(None);
                keep(None);
            }
        }
    };
    let refuse_setter = setter.clone();
    // Returns the error to say; the text stays.
    let refuse = {
        let keep = keep.clone();
        move |text: String| {
            let error =
                Rc::new(|locale: &Localization| locale.number_field.invalid_number.to_string())
                    as Rc<dyn FieldError>;
            let said = word(&*error);
            let mut rejected = rejected;
            rejected.set(Some(error.clone()));
            keep(Some(Typed { text, error }));
            if let (true, Some(setter)) = (resets_value && current().is_some(), &refuse_setter) {
                setter.set(None);
            }
            said
        }
    };
    let publish = move |next: Option<T>| {
        let next = next.map(|next| rounded(next.clamp_between(min, max)));
        match (&onchange, &setter) {
            (Some(onchange), _) => onchange.call(next),
            (None, Some(setter)) => setter.set(next),
            (None, None) => {}
        }
    };
    let typed_publish = publish.clone();
    let spin_publish = publish.clone();
    // A spinner's step: the column sends the number it shows next.
    let spin_unrefuse = unrefuse.clone();
    let spin_to = use_callback(move |next: T| {
        if !spin_opens {
            return;
        }
        buffer.set(None);
        spin_unrefuse();
        spin_publish(Some(next));
    });
    // One identity across renders, so `Steppers` can skip. Safe as a
    // `use_callback`: nothing in it focuses or clicks, so it cannot re-enter.
    let nudge_unrefuse = unrefuse.clone();
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
        nudge_unrefuse();
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

    // Refused text outranks the caller's status: the field cannot hold what it shows.
    let status: Input<FieldStatus> = match rejected() {
        Some(error) => Input::Value(FieldStatus::Error(word(&*error))),
        None => props.status.clone(),
    };
    let field = use_field();
    let field = match rejected() {
        Some(error) => field.parse(Parse::Invalid(error)),
        None => field,
    };
    let field = field
        .label(&props.label)
        .description(&props.description)
        .helper(&props.helper)
        .status(&status)
        .rules(rules)
        .bound(&bound)
        .required(required)
        .empty(bound.is_empty(
            required,
            || Some(bound_value.unwrap_or_else(|| live.cloned())),
            Option::is_none,
        ))
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius.clone())
        .class(&props.class)
        .sx(&props.sx)
        .parts(&props.parts)
        .states(&props.states)
        .attributes(&props.attributes)
        .text_slots(
            props.describe_leading && leading.is_some(),
            props.describe_trailing && caller_trailing.is_some(),
        )
        .prepare();

    // The steppers sit last. The id goes on the caller's part alone, so their names
    // do not describe the input.
    let [leading_id, trailing_id] = field.slot_ids();
    let unit_id = trailing_id.clone();
    // The memo alone reads the number, so a step redraws the buttons only at a bound.
    // Typed text steps from itself (see `nudge`), so it counts, not the caller's value.
    let reach =
        use_memo(use_reactive!(|bound_value,
                                min,
                                max,
                                separator,
                                decimals| {
            let value = bound_value.unwrap_or_else(|| live.cloned());
            // A bound the places cannot show is reached at its rounded value: 9.99 under a 9.999 max (4360).
            let bound =
                |bound: Option<T>| bound.map(|bound| rounded_within(bound, decimals, min, max));
            let value = buffer
                .read()
                .as_ref()
                .filter(|edit| edit.shows(value, separator))
                .and_then(|edit| parse::<T>(&edit.text, separator))
                .or(value);
            Reach {
                at_min: value.is_some() && value == bound(min),
                at_max: value.is_some() && value == bound(max),
            }
        }));
    let steppers = props.steppers.then(|| {
        rsx! {
            Steppers {
                size,
                disabled: disabled || readonly,
                reach,
                increment_label: props.increment_label.clone(),
                decrement_label: props.decrement_label.clone(),
                nudge,
            }
        }
    });
    let trailing = (caller_trailing.is_some() || steppers.is_some()).then(|| {
        let caller = match (caller_trailing, trailing_id) {
            (Some(caller), Some(id)) => rsx! { span { id, {caller} } },
            (caller, _) => caller.unwrap_or_else(|| rsx! {}),
        };
        rsx! {
            {caller}
            {steppers}
        }
    });
    let frame = use_field_frame()
        .leading(&leading)
        .trailing(&trailing)
        .ids([leading_id, None])
        .states(field.states())
        .placeholder(props.placeholder.as_deref())
        .prepare();

    let control = use_box()
        .framework_sx(&NUMBER_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let typed_unrefuse = unrefuse.clone();
    let announcer = use_announcer();
    let on_text = move |mut text: String| {
        let mut buffer = buffer;
        typed_unrefuse();
        announcer.clear();
        // Written back at once, as a native field drops a char past `maxlength`: a cut that
        // equals the last render diffs to nothing.
        if let Some(cut) = decimals.and_then(|decimals| {
            decimals.cut(&text, separator.decimal.unwrap_or("."), separator.group)
        }) {
            let _ = anchor.set_value(&cut);
            text = cut;
        }
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
    };
    let prefilled = on_text.clone();
    let (blur_refuse, enter_refuse) = (refuse.clone(), refuse);
    use_prefill(
        anchor,
        None,
        Some(move |text: String| {
            if parse::<T>(&text, separator) != current() {
                prefilled(text);
            }
        }),
        disabled || readonly,
    );
    let input = field
        .aria(control)
        .element(&anchor)
        .attr_default("type", "text")
        // A leading minus takes the page direction, so `-5` drew as `5-` under `rtl`.
        .attr_default("dir", "ltr")
        // The role carries the range: `min`/`max` mean nothing on `type="text"`.
        .attr("role", "spinbutton")
        .attr("aria-valuemin", aria_number(min))
        .attr("aria-valuemax", aria_number(max))
        .attr("name", bound.name().map(str::to_string))
        .attr("data-controlled", true)
        .attr("placeholder", props.placeholder)
        .attr("disabled", disabled && !soft_disabled)
        .attr("aria-disabled", soft_disabled.then_some("true"))
        .attr("readonly", readonly)
        .attr("required", required)
        .attr(TOOLBAR_ITEM, item.map(ToolbarItem::key))
        .attr("tabindex", item.map(ToolbarItem::tabindex))
        .event("oninput", move |event: FormEvent| on_text(event.value()))
        // Fires when the field is left or Enter commits an edit. Dropping the
        // buffer redraws the canonical text, which also rewrites the DOM.
        // Text that is no number stays, with the field's error (4359).
        .event("onchange", move |event: FormEvent| {
            let text = event.value();
            match parse::<T>(&text, separator) {
                None if !text.trim().is_empty() => {
                    blur_refuse(text);
                    return;
                }
                Some(parsed) if parsed.clamp_between(min, max) != parsed => commit(Some(parsed)),
                _ => {}
            }
            keep(None);
            buffer.set(None);
        })
        .event("onkeydown", move |event: KeyboardEvent| {
            // A composing arrow picks an IME candidate; Ctrl+PageDown switches tabs, Ctrl+ArrowUp moves the caret.
            if event.is_composing() || navigation_chord(&event).is_some() {
                return;
            }
            let steps = match event.key() {
                // A refused text holds the form's implicit submit, so the error stays seen.
                Key::Named(NamedKey::Enter) => {
                    let text = buffer.peek().as_ref().map(|edit| edit.text.clone());
                    if let Some(text) = text.filter(|text| {
                        !text.trim().is_empty() && parse::<T>(text, separator).is_none()
                    }) {
                        event.prevent_default();
                        announcer.say(enter_refuse(text));
                    }
                    return;
                }
                Key::Named(NamedKey::ArrowUp) => 1,
                Key::Named(NamedKey::ArrowDown) => -1,
                Key::Named(NamedKey::PageUp) => 10,
                Key::Named(NamedKey::PageDown) => -10,
                _ => return,
            };
            // Otherwise the caret jumps to the end of the text as well.
            event.prevent_default();
            nudge.call(steps);
        });
    // The tap's native focus raised no keyboard; its click opens the spinner.
    let input = match spin_opens {
        true => input.event("onclick", move |_: MouseEvent| {
            let now = *spin.peek();
            if now.keyboardless() && !now.open {
                spin.write().open = true;
            }
        }),
        false => input,
    };
    // `focusin`/`focusout`: a caller's own `onfocus` and `onblur` stay theirs.
    let input = match grouped {
        true => input
            .event("onfocusin", move |_: FocusEvent| focused.set(true))
            .event("onfocusout", move |_: FocusEvent| focused.set(false)),
        false => input,
    };
    // The spinner's dialog and column take the field's name, a spread one too.
    // Not `field.label_id()`: that is set only for a `labelled_by` control, and the input has
    // `<label for>`; the label always carries this id (4310).
    let own_label = (!props.label.is_none()).then(|| format!("{}-label", field.id()));
    let labelled_by = own_label.or_else(|| {
        props
            .attributes
            .iter()
            .find_map(|attribute| match &attribute.value {
                AttributeValue::Text(id) if attribute.name == "aria-labelledby" => Some(id.clone()),
                _ => None,
            })
    });
    let aria_label = aria_label_of(&props.attributes);
    let attributes = props.attributes;
    // The caret of the grouped text a focus is about to strip, put right after the redraw.
    let mut stripped = use_hook(|| CopyValue::new(None::<Stripped>));
    let grouped_shown = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        focused();
        if let Some(stripped) = stripped.write().take() {
            spawn(async move {
                next_task().await;
                let after = anchor.selection_start();
                if after.is_some_and(|after| stripped.reselects(after)) {
                    let _ = anchor.select();
                } else if let Some(at) = after.and_then(|after| stripped.fix(after)) {
                    let _ = anchor.set_caret(at);
                }
            });
        }
    });
    let draw = Rc::new(move || {
        let current = bound_value.unwrap_or_else(|| live.cloned());
        let (mut stripped, mut grouped_shown) = (stripped, grouped_shown);
        let at_rest = grouped && !focused();
        if grouped_shown.replace(at_rest) && !at_rest {
            let text = current.map(|value| formats.write_number(&value.format()));
            let caret = anchor.selection_start().zip(text).zip(separator.group);
            let selected_to = anchor.selection_end();
            stripped
                .set(caret.map(|((at, text), group)| Stripped::of(&text, at, selected_to, group)));
        }
        let display = match &*buffer.read() {
            Some(edit) if edit.shows(current, separator) => edit.text.clone(),
            _ => current
                .map(|value| written(value, decimals, at_rest, separator, formats))
                .unwrap_or_default(),
        };
        // Read only by a spinner field, so a plain one never redraws for it.
        let keyboardless = spins && spin().keyboardless();
        // What the role reads besides the number: a group, a padded place or a comma.
        let now = aria_number(current);
        let value_text = current
            .zip(now.as_ref())
            .map(|(value, _)| written(value, decimals, grouped, separator, formats))
            .filter(|text| Some(text) != now.as_ref());
        input
            .clone()
            .attr_default(
                "inputmode",
                if keyboardless {
                    "none"
                } else {
                    keypad(kind, min)
                },
            )
            .attr("aria-valuenow", now)
            .attr("aria-valuetext", value_text)
            .attr("value", display)
            .render(HtmlTag::Input, attributes.clone(), ())
    });

    let control = frame.render(rsx! { LiveControl { draw } });
    let control = match spins {
        true => rsx! {
            NumberSpinner::<T> {
                control,
                spin,
                live,
                bound: bound_value,
                field_id: field.id().to_string(),
                labelled_by,
                aria_label,
                field_root: field.root(),
                look: Look { decimals, grouped, separator },
                step,
                min,
                max,
                size,
                unit: unit.filter(|_| props.describe_trailing),
                unit_id,
                spin_to,
            }
        },
        false => control,
    };
    field.render(rsx! {
        {control}
        {announcer.render()}
    })
}

/// Text typed since the last commit, shown while it parses to the caller's
/// value or that value has not moved. Dropping it writes the canonical text.
#[derive(Clone, PartialEq)]
struct Edit<T> {
    text: String,
    held: Option<T>,
}

impl<T: NumberValue> Edit<T> {
    fn shows(&self, current: Option<T>, separator: Separators) -> bool {
        self.held == current || parse::<T>(&self.text, separator) == current
    }
}

/// A clamped `value` rounded through the text, so `0.1 + 0.2` publishes `0.3`. One that
/// rounds past a bound (`max: 9.999` to 10.00) takes the bound cut toward the range.
fn rounded_within<T: NumberValue>(
    value: T,
    decimals: Option<Decimals>,
    min: Option<T>,
    max: Option<T>,
) -> T {
    let Some(decimals) = decimals else {
        return value;
    };
    let next = T::parse(&decimals.write(&value.format())).unwrap_or(value);
    let inside = next.clamp_between(min, max);
    if inside == next {
        return next;
    }
    decimals
        .toward(&inside.format(), Some(inside) == min)
        .and_then(|text| T::parse(&text))
        .filter(|cut| cut.clamp_between(min, max) == *cut)
        .unwrap_or(inside)
}

/// The text a field shows for `value` when no typing is held; `grouped` writes the groups.
fn written<T: NumberValue>(
    value: T,
    decimals: Option<Decimals>,
    grouped: bool,
    separator: Separators,
    formats: &Formats,
) -> String {
    let text = match decimals {
        Some(decimals) => decimals.write(&value.format()),
        None => value.format(),
    };
    match (grouped, separator.decimal) {
        (true, _) => formats.write_number(&text),
        (false, Some(decimal)) => text.replacen('.', decimal, 1),
        (false, None) => text,
    }
}

/// The formats' separators a field reads besides Rust's own text. Built-in numbers only:
/// a custom `NumberValue` owns its text, which may already be localized.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Separators {
    /// The decimal separator where it is not `.`, floats only.
    decimal: Option<&'static str>,
    /// The group separator, dropped where it splits the whole into threes.
    group: Option<&'static str>,
}

/// `f32` or `f64`, whose fraction a field may round.
fn float<T: 'static>() -> bool {
    TypeId::of::<T>() == TypeId::of::<f64>() || TypeId::of::<T>() == TypeId::of::<f32>()
}

impl Separators {
    fn of<T: 'static>(formats: &Formats) -> Self {
        let float = float::<T>();
        let group = formats.group_separator;
        Self {
            decimal: (float && formats.decimal_separator != ".")
                .then_some(formats.decimal_separator),
            group: (built_in::<T>() && !group.is_empty()).then_some(group),
        }
    }
}

/// One of Rust's own number types, whose text `NumberField` may write in the locale.
fn built_in<T: 'static>() -> bool {
    [
        TypeId::of::<f32>(),
        TypeId::of::<f64>(),
        TypeId::of::<i8>(),
        TypeId::of::<i16>(),
        TypeId::of::<i32>(),
        TypeId::of::<i64>(),
        TypeId::of::<i128>(),
        TypeId::of::<isize>(),
        TypeId::of::<u8>(),
        TypeId::of::<u16>(),
        TypeId::of::<u32>(),
        TypeId::of::<u64>(),
        TypeId::of::<u128>(),
        TypeId::of::<usize>(),
    ]
    .contains(&TypeId::of::<T>())
}

/// The caret of grouped text (UTF-16 units) and where it lands once the separators go.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Stripped {
    before: u32,
    mapped: u32,
    end: u32,
    /// The whole grouped text was selected, as a Tab in selects it (4184).
    whole: bool,
}

impl Stripped {
    fn of(text: &str, before: u32, selected_to: Option<u32>, group: &str) -> Self {
        let length = text.encode_utf16().count() as u32;
        Self {
            before,
            mapped: caret_without_groups(text, before, group),
            end: caret_without_groups(text, u32::MAX, group),
            whole: before == 0 && length > 0 && selected_to == Some(length),
        }
    }

    /// Whether to select the stripped text again: the rewrite collapsed the selection.
    fn reselects(self, after: u32) -> bool {
        self.whole && (after == 0 || after == self.end)
    }

    /// The caret to put once the rewrite left it at `after`: a browser keeps the index or
    /// sends it to the end, while a press that came later placed its own.
    fn fix(self, after: u32) -> Option<u32> {
        let moved = self.before > 0 && (after == self.before || after == self.end);
        (moved && after != self.mapped).then_some(self.mapped)
    }
}

/// `caret` (UTF-16 units) in `text` moved to the same place once its group separators are gone.
fn caret_without_groups(text: &str, caret: u32, group: &str) -> u32 {
    let mut units = 0;
    let before: String = text
        .chars()
        .take_while(|char| {
            units += char.len_utf16() as u32;
            units <= caret
        })
        .collect();
    before.replace(group, "").encode_utf16().count() as u32
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

/// The number as `aria-value*` text; `None` for `NaN` and `inf`, which ARIA cannot read.
fn aria_number<T: NumberValue>(number: Option<T>) -> Option<String> {
    let text = number?.to_string();
    // A custom type's text that is no float stays.
    let finite = text
        .parse::<f64>()
        .map_or(true, |number| number.is_finite());
    finite.then_some(text)
}

/// `T::parse`, taking the separator as well as `.`. Grouped text (`12,500`) parses too; where
/// `.` groups (German) one outside the threes is no number, not `1.2345` read as 1234.5 (4267).
fn parse<T: NumberValue>(text: &str, separator: Separators) -> Option<T> {
    let text = ascii_digits(text);
    let text = match separator.group.and_then(|group| ungrouped(&text, group)) {
        Some(bare) => Cow::Owned(bare),
        None if separator.group == Some(".") && text.contains('.') => return None,
        None => text,
    };
    match separator.decimal {
        Some(decimal) => T::parse(&text.replacen(decimal, ".", 1)),
        None => T::parse(&text),
    }
}

/// Digits of another script and the full-width or true minus as ASCII: the
/// default of a Persian or Arabic keyboard and a CJK IME in full-width mode.
fn ascii_digits(text: &str) -> Cow<'_, str> {
    let fold = |c: char| match c {
        '\u{FF0D}' | '\u{2212}' => Some('-'),
        '\u{066B}' | '\u{FF0E}' => Some('.'),
        _ => ascii_digit(c).filter(|digit| *digit != c),
    };
    match text.chars().any(|c| fold(c).is_some()) {
        true => Cow::Owned(text.chars().map(|c| fold(c).unwrap_or(c)).collect()),
        false => Cow::Borrowed(text),
    }
}

/// Whether the number sits on a bound, where the matching stepper has nothing to do.
#[derive(Clone, Copy, PartialEq)]
struct Reach {
    at_min: bool,
    at_max: bool,
}

/// The two stepper buttons, a scope of their own: every prop compares equal
/// while the value moves, so a press redraws the field and not the buttons.
#[component]
fn Steppers(
    size: Size,
    disabled: bool,
    reach: Memo<Reach>,
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
    let reach = reach();
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
                disabled: disabled || reach.at_min,
                onclick: move |_| nudge.call(-1),
                Glyph { slot: IconSlot::Minus, icon: lucide::minus::outlined }
            }
            ActionIcon {
                aria_label: increment_label.unwrap_or_else(|| labels.increase.to_string()),
                size: ThemeAwareValue::Size(slot_icon_size(size)),
                sx: &INCREMENT_SX,
                tabindex: "-1",
                disabled: disabled || reach.at_max,
                onclick: move |_| nudge.call(1),
                Glyph { slot: IconSlot::Plus, icon: lucide::plus::outlined }
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::{
        DECREMENT_SX, Decimals, Separators, Stripped, aria_number, caret_without_groups, parse,
        rounded_within,
    };
    use crate::{css::Stylesheet, localization::Formats};

    /// Todo 3589: under a finger a stepper's box reaches 44px, still stopping mid-gap.
    #[test]
    fn a_stepper_box_grows_to_44px_under_a_coarse_pointer() {
        let css = Stylesheet::from(&DECREMENT_SX);
        let css = css.as_str();
        let media = css
            .find("@media (pointer: coarse)")
            .unwrap_or_else(|| panic!("no coarse query: {css}"));
        assert!(css[media..].contains("calc(100% - 43px)"), "{css}");
        assert!(css[..media].contains("calc(100% - 23px)"), "{css}");
    }

    /// Todo 3430: the caret keeps its digit when focus strips the separators.
    #[test]
    fn the_caret_keeps_its_digit_when_the_separators_go() {
        assert_eq!(caret_without_groups("1,234,567.5", 4, ","), 3);
        assert_eq!(caret_without_groups("1,234,567.5", 2, ","), 1);
        assert_eq!(caret_without_groups("1,234,567.5", 0, ","), 0);
        assert_eq!(caret_without_groups("1.234.567,5", u32::MAX, "."), 9);
    }

    /// A press that placed its own caret after the rewrite is left alone.
    #[test]
    fn only_a_caret_the_rewrite_moved_is_put_back() {
        let stripped = Stripped::of("1,234,567.5", 4, Some(4), ",");
        assert_eq!(stripped.fix(4), Some(3), "the index kept");
        assert_eq!(stripped.fix(9), Some(3), "sent to the end");
        assert_eq!(stripped.fix(6), None, "a press placed it");
        assert_eq!(stripped.fix(3), None, "already there");
        assert!(!stripped.reselects(9), "a caret is not a selection");
        assert_eq!(
            Stripped::of("1,234,567.5", 0, Some(0), ",").fix(9),
            None,
            "clicked at the start"
        );
    }

    /// Todo 4184: a Tab in selects the whole grouped text, so the stripped text is selected again.
    #[test]
    fn a_whole_selection_is_selected_again() {
        let tabbed = Stripped::of("1,234", 0, Some(5), ",");
        assert!(tabbed.reselects(4), "sent to the end");
        assert!(tabbed.reselects(0), "the index kept");
        assert!(!tabbed.reselects(2), "a press placed it");
        assert!(!Stripped::of("1,234", 0, Some(0), ",").reselects(4));
        assert!(!Stripped::of("1,234", 0, Some(3), ",").reselects(4));
        assert!(
            !Stripped::of("1,234", 0, None, ",").reselects(4),
            "no end read"
        );
    }

    const NONE: Separators = Separators {
        decimal: None,
        group: None,
    };
    const COMMA: Separators = Separators {
        decimal: Some(","),
        group: None,
    };

    /// Todo 2436: `aria-valuenow="NaN"` and `aria-valuemax="inf"` are no numbers.
    #[test]
    fn a_non_finite_number_is_left_out_of_aria() {
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(aria_number(Some(number)), None, "{number}");
        }
        assert_eq!(aria_number(Some(2.5)), Some("2.5".into()));
        assert_eq!(aria_number(Some(-3_i32)), Some("-3".into()));
        assert_eq!(aria_number::<f64>(None), None);
    }

    /// Todo 2646: the Arabic and full-width decimal separators fold to `.`.
    #[test]
    fn non_ascii_decimal_separators_parse() {
        assert_eq!(parse::<f64>("1\u{066B}5", NONE), Some(1.5));
        assert_eq!(parse::<f64>("\u{FF11}\u{FF0E}\u{FF15}", NONE), Some(1.5));
        assert_eq!(parse::<f64>("1\u{066B}5", COMMA), Some(1.5));
    }

    /// Todos 3390, 4267: text grouped in the locale parses; a `.` outside the threes is no number.
    #[test]
    fn grouped_text_parses_in_the_locale() {
        let german = Separators::of::<f64>(&Formats::GERMAN);
        assert_eq!(parse::<f64>("12.500,5", german), Some(12_500.5));
        assert_eq!(parse::<f64>("-1.234.567", german), Some(-1_234_567.0));
        assert_eq!(parse::<f64>("1.234", german), Some(1_234.0));
        for text in ["1.5", "1.2345", "1.23", "12.5,5", "1.234.5"] {
            assert_eq!(parse::<f64>(text, german), None, "{text}");
        }
        assert_eq!(parse::<f64>("1,2345", german), Some(1.2345));
        assert_eq!(
            parse::<i32>("2.026", Separators::of::<i32>(&Formats::GERMAN)),
            Some(2026)
        );

        let american = Separators::of::<f64>(&Formats::AMERICAN);
        assert_eq!(parse::<f64>("12,500.5", american), Some(12_500.5));
        assert_eq!(parse::<f64>("1,5", american), None);
        assert_eq!(parse::<f64>("12,50", american), None);
        assert_eq!(
            parse::<u32>("1234", Separators::of::<u32>(&Formats::AMERICAN)),
            Some(1234)
        );
    }

    /// Todo 3390: `group_digits` writes the shown number grouped in the locale.
    #[test]
    fn group_digits_writes_the_number_grouped() {
        use super::NumberField;
        use crate::LiberoProvider;
        use dioxus::prelude::*;

        fn render(app: fn() -> Element) -> String {
            let mut dom = VirtualDom::new(app);
            dom.rebuild_in_place();
            dioxus_ssr::render(&dom)
        }
        let grouped = render(|| {
            rsx! {
                LiberoProvider { formats: &Formats::GERMAN,
                    NumberField::<f64> { label: "Amount", value: Some(12_500.5), group_digits: true, onchange: |_| {} }
                    NumberField::<u32> { label: "Year", value: Some(2026), onchange: |_| {} }
                }
            }
        });
        assert!(grouped.contains("value=\"12.500,5\""), "{grouped}");
        assert!(grouped.contains("value=\"2026\""), "{grouped}");
    }

    /// Todo 4273: the role reads the shown text beside the number, only where they differ.
    #[test]
    fn the_spinbutton_reads_the_shown_text() {
        use super::{Decimals, NumberField};
        use crate::LiberoProvider;
        use dioxus::prelude::*;

        let mut dom = VirtualDom::new(|| {
            rsx! {
                LiberoProvider { formats: &Formats::GERMAN,
                    NumberField::<f64> { label: "Amount", value: Some(12_500.5), group_digits: true, decimals: Decimals::Exactly(2), onchange: |_| {} }
                    NumberField::<u32> { label: "Year", value: Some(2026), onchange: |_| {} }
                    NumberField::<f64> { label: "Empty", value: None, group_digits: true, onchange: |_| {} }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("aria-valuenow=\"12500.5\""), "{html}");
        assert!(html.contains("aria-valuetext=\"12.500,50\""), "{html}");
        assert_eq!(html.matches("aria-valuetext").count(), 1, "{html}");
    }

    /// Todo 4271: a stepper at its bound is disabled; the other one and a field inside are not.
    #[test]
    fn a_stepper_at_its_bound_is_disabled() {
        use super::NumberField;
        use crate::LiberoProvider;
        use dioxus::prelude::*;

        #[component]
        fn App(value: f64) -> Element {
            rsx! {
                LiberoProvider {
                    NumberField::<f64> { label: "A", value: Some(value), min: 0.0, max: 10.0, steppers: true, onchange: |_| {} }
                }
            }
        }
        // The buttons, minus then plus, as `(decrease, increase)` disabled flags.
        let stepper_state = |value: f64| {
            let mut dom = VirtualDom::new_with_props(App, AppProps { value });
            dom.rebuild_in_place();
            let html = dioxus_ssr::render(&dom);
            let buttons = html.split("<button").skip(1).map(|button| {
                let tag = &button[..button.find('>').unwrap_or(button.len())];
                tag.contains("disabled")
            });
            buttons.collect::<Vec<_>>()
        };
        assert_eq!(stepper_state(0.0), [true, false]);
        assert_eq!(stepper_state(10.0), [false, true]);
        assert_eq!(stepper_state(5.0), [false, false]);
    }

    /// Todo 3962: `decimals` rounds Rust's text and pads it only when exact.
    #[test]
    fn decimals_round_and_pad() {
        use super::Decimals::{AtMost, Exactly};
        assert_eq!(Exactly(2).write("12.5"), "12.50");
        assert_eq!(Exactly(2).write("0.30000000000000004"), "0.30");
        assert_eq!(AtMost(2).write("0.30000000000000004"), "0.3");
        assert_eq!(AtMost(2).write("12"), "12");
        assert_eq!(AtMost(0).write("12.6"), "13");
        assert_eq!(Exactly(0).write("12.4"), "12");
    }

    /// Todo 3962: typed digits past `decimals` are cut, in either mark.
    #[test]
    fn decimals_cut_typed_digits() {
        use super::Decimals::{AtMost, Exactly};
        assert_eq!(Exactly(2).cut("12.345", ".", None), Some("12.34".into()));
        assert_eq!(Exactly(2).cut("12,345", ",", None), Some("12,34".into()));
        assert_eq!(
            Exactly(2).cut("12.345", ",", Some(" ")),
            Some("12.34".into())
        );
        assert_eq!(AtMost(2).cut("12.34", ".", None), None);
        assert_eq!(AtMost(2).cut("12", ".", None), None);
        assert_eq!(AtMost(0).cut("12.", ".", None), None);
        assert_eq!(AtMost(0).cut("12.5", ".", None), Some("12".into()));
        // Todo 4358: where `.` groups, `1.234` is a thousand, not a fraction to cut.
        assert_eq!(Exactly(2).cut("1.234", ",", Some(".")), None);
        assert_eq!(
            Exactly(2).cut("1.234,567", ",", Some(".")),
            Some("1.234,56".into())
        );
    }

    /// Todo 3962: the caller's slots draw, and `Exactly` pads the shown number.
    #[test]
    fn slots_and_exact_decimals_render() {
        use super::{Decimals, NumberField};
        use crate::LiberoProvider;
        use dioxus::prelude::*;

        let mut dom = VirtualDom::new(|| {
            rsx! {
                LiberoProvider {
                    NumberField::<f64> {
                        label: "Price",
                        value: Some(12.5),
                        decimals: Decimals::Exactly(2),
                        leading: rsx! { "$" },
                        describe_leading: true,
                        trailing: rsx! { "USD" },
                        onchange: |_| {},
                    }
                }
            }
        });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("value=\"12.50\""), "{html}");
        assert!(html.contains("$") && html.contains("USD"), "{html}");
        assert!(html.contains("aria-describedby"), "{html}");
    }

    /// Todo 2414: these typed digits never parsed and reverted on blur.
    #[test]
    fn non_ascii_digits_parse() {
        assert_eq!(parse::<i32>("\u{0661}\u{0662}", NONE), Some(12));
        assert_eq!(parse::<i32>("\u{06F4}\u{06F2}", NONE), Some(42));
        assert_eq!(parse::<i32>("\u{FF0D}\u{FF17}", NONE), Some(-7));
        assert_eq!(parse::<f64>("\u{FF11},\u{FF15}", COMMA), Some(1.5));
        assert_eq!(parse::<i32>("12", NONE), Some(12));
    }

    /// Todo 4269: rounding never lands past a bound; it cuts the bound toward the range.
    #[test]
    fn rounding_stays_inside_the_range() {
        let two = Some(Decimals::Exactly(2));
        assert_eq!(rounded_within(9.999, two, None, Some(9.999)), 9.99);
        assert_eq!(rounded_within(0.001, two, Some(0.001), None), 0.01);
        assert_eq!(rounded_within(5.006, two, Some(0.0), Some(9.999)), 5.01);
        assert_eq!(rounded_within(0.1 + 0.2, two, None, None), 0.3);
        // No shown number fits: the bound itself.
        assert_eq!(rounded_within(0.004, two, Some(0.001), Some(0.004)), 0.001);
    }

    /// Todo 4270: a step these places cannot show rounds to them, at least one unit.
    #[test]
    fn a_step_finer_than_the_places_coarsens() {
        assert_eq!(Decimals::AtMost(0).coarse_step("0.5").as_deref(), Some("1"));
        assert_eq!(
            Decimals::Exactly(1).coarse_step("0.16").as_deref(),
            Some("0.2")
        );
        assert_eq!(
            Decimals::AtMost(2).coarse_step("0.001").as_deref(),
            Some("0.01")
        );
        assert_eq!(Decimals::AtMost(1).coarse_step("0.5"), None);
        assert_eq!(Decimals::AtMost(0).coarse_step("2"), None);
    }
}
