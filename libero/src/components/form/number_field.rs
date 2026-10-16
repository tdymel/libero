use std::{any::TypeId, borrow::Cow, rc::Rc};

use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        buttons::{ACTION_ICON_COMPACT, ActionIcon},
        common::{
            COARSE_POINTER, Glyph, HtmlTag, Input, NumberValue, ScaleOrCss, TOOLBAR_ITEM,
            ToolbarItem, navigation_chord, use_no_toolbar, use_toolbar_item,
        },
        form::{
            FormScope, LiveControl, field_control_sx, field_props, slot_button_sx, slot_icon_size,
            use_bound, use_field, use_field_frame,
        },
        layout::use_box,
    },
    context::IconSlot,
    hooks::{
        current_localization, use_element, use_form_owner, use_formats, use_silent_focus_in,
        use_silent_focus_out_of, use_theme,
    },
    localization::Formats,
    platform::{ElementApi, next_task},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::Size,
    utils::{ascii_digit, warn},
};

/// The steppers in the trailing slot. Not the native spinner: `type="number"`
/// empties text it cannot parse, the in-progress text the edit buffer keeps.
static STEPPERS_SX: StaticSx =
    StaticSx::new(|| sx().display("flex").align_items("center").gap("2px"));

/// The input is `dir="ltr"`; its text still sits at the page's start.
static NUMBER_CONTROL_SX: StaticSx = StaticSx::new(|| {
    field_control_sx()
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
        /// Writes the thousands grouped (`12,500`) while the field is not focused; typing
        /// sees the bare number. Built-in number types only: a custom one writes itself.
        /// A WebView cannot read or set the caret: focusing the field may send it to the end.
        #[props(default)]
        group_digits: bool,
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
    let radius = ScaleOrCss::new(props.radius.as_ref(), theme.number_field.radius);
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
    // A form reset drops the typed text, which `-` or `.` would outlive on a `None` value.
    let anchor = use_element();
    let scope = try_use_context::<FormScope>();
    let owner = use_form_owner(anchor, scope.is_none());
    let resets = scope.map_or(0, |form| form.resets()) + owner().unwrap_or(0);
    use_effect(use_reactive!(|resets| {
        let _ = resets;
        if buffer.peek().is_some() {
            buffer.set(None);
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
        .framework_sx(&NUMBER_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    let input = field
        .aria(control)
        .element(&anchor)
        .attr_default("type", "text")
        .attr_default("inputmode", keypad(kind, min))
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
            // A composing arrow picks an IME candidate; Ctrl+PageDown switches tabs, Ctrl+ArrowUp moves the caret.
            if event.is_composing() || navigation_chord(&event).is_some() {
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
    // `focusin`/`focusout`: a caller's own `onfocus` and `onblur` stay theirs.
    let input = match grouped {
        true => input
            .event("onfocusin", move |_: FocusEvent| focused.set(true))
            .event("onfocusout", move |_: FocusEvent| focused.set(false)),
        false => input,
    };
    let attributes = props.attributes;
    // The caret of the grouped text a focus is about to strip, put right after the redraw.
    let mut stripped = use_hook(|| CopyValue::new(None::<Stripped>));
    let grouped_shown = use_hook(|| CopyValue::new(false));
    use_effect(move || {
        focused();
        if let Some(stripped) = stripped.write().take() {
            spawn(async move {
                next_task().await;
                if let Some(at) = anchor
                    .selection_start()
                    .and_then(|after| stripped.fix(after))
                {
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
            stripped.set(caret.map(|((at, text), group)| Stripped::of(&text, at, group)));
        }
        let display = match &*buffer.read() {
            Some(edit) if edit.shows(current, separator) => edit.text.clone(),
            _ => current
                .map(|value| match (grouped && !focused(), separator.decimal) {
                    (true, _) => formats.write_number(&value.format()),
                    (false, Some(decimal)) => value.format().replacen('.', decimal, 1),
                    (false, None) => value.format(),
                })
                .unwrap_or_default(),
        };
        input
            .clone()
            .attr("aria-valuenow", aria_number(current))
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
    fn shows(&self, current: Option<T>, separator: Separators) -> bool {
        self.held == current || parse::<T>(&self.text, separator) == current
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

impl Separators {
    fn of<T: 'static>(formats: &Formats) -> Self {
        let float =
            TypeId::of::<T>() == TypeId::of::<f64>() || TypeId::of::<T>() == TypeId::of::<f32>();
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

/// `text` without its group separators when they split the whole into threes, as
/// `12.500,5`; else `None`, so a German `1.5` from a numpad stays a decimal.
fn ungrouped(text: &str, group: &str) -> Option<String> {
    let (sign, rest) = match text.trim().strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", text.trim()),
    };
    let parts: Vec<&str> = rest.split(group).collect();
    let digits = |part: &str| part.chars().take_while(char::is_ascii_digit).count();
    let (first, last) = (parts.first()?, parts.last()?);
    let grouped = parts.len() > 1
        && (1..=3).contains(&first.len())
        && digits(first) == first.len()
        && parts[1..parts.len() - 1]
            .iter()
            .all(|part| part.len() == 3 && digits(part) == 3)
        && digits(last) == 3;
    grouped.then(|| format!("{sign}{}", parts.concat()))
}

/// The caret of grouped text (UTF-16 units) and where it lands once the separators go.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Stripped {
    before: u32,
    mapped: u32,
    end: u32,
}

impl Stripped {
    fn of(text: &str, before: u32, group: &str) -> Self {
        Self {
            before,
            mapped: caret_without_groups(text, before, group),
            end: caret_without_groups(text, u32::MAX, group),
        }
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

/// `T::parse`, taking the separator as well as `.`: a German user on a US
/// layout or a numpad types `1.5`. Grouped text (`12,500`) parses too.
fn parse<T: NumberValue>(text: &str, separator: Separators) -> Option<T> {
    let text = ascii_digits(text);
    let text = match separator.group.and_then(|group| ungrouped(&text, group)) {
        Some(bare) => Cow::Owned(bare),
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

#[cfg(test)]
mod tests {
    use super::{DECREMENT_SX, Separators, Stripped, aria_number, caret_without_groups, parse};
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
        let stripped = Stripped::of("1,234,567.5", 4, ",");
        assert_eq!(stripped.fix(4), Some(3), "the index kept");
        assert_eq!(stripped.fix(9), Some(3), "sent to the end");
        assert_eq!(stripped.fix(6), None, "a press placed it");
        assert_eq!(stripped.fix(3), None, "already there");
        assert_eq!(
            Stripped::of("1,234,567.5", 0, ",").fix(9),
            None,
            "tabbed in"
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

    /// Todo 3390: text grouped in the locale parses; a numpad's `1.5` stays a decimal.
    #[test]
    fn grouped_text_parses_in_the_locale() {
        let german = Separators::of::<f64>(&Formats::GERMAN);
        assert_eq!(parse::<f64>("12.500,5", german), Some(12_500.5));
        assert_eq!(parse::<f64>("-1.234.567", german), Some(-1_234_567.0));
        assert_eq!(parse::<f64>("1.5", german), Some(1.5));
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

    /// Todo 2414: these typed digits never parsed and reverted on blur.
    #[test]
    fn non_ascii_digits_parse() {
        assert_eq!(parse::<i32>("\u{0661}\u{0662}", NONE), Some(12));
        assert_eq!(parse::<i32>("\u{06F4}\u{06F2}", NONE), Some(42));
        assert_eq!(parse::<i32>("\u{FF0D}\u{FF17}", NONE), Some(-7));
        assert_eq!(parse::<f64>("\u{FF11},\u{FF15}", COMMA), Some(1.5));
        assert_eq!(parse::<i32>("12", NONE), Some(12));
    }
}
