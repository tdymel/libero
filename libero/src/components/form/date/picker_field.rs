//! The engine every date and time field shares: a text input read on blur or
//! Enter, a dropdown holding a picker, and a hidden input posting ISO 8601.
//! Generic over the value, but private - each public field names its own
//! type, so a caller never meets an inference error.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;

use super::{
    DateRange,
    format::{format_date, format_time},
    parse::{Unreadable, parse_date},
    parse_time::{parse_date_time, parse_time},
    range::{iso_date_time, split_range},
    today::use_today,
};
use crate::{
    components::{
        Caption, ClassList, FieldName, HtmlTag, Input, States, Validators,
        form::{FIELD_CONTROL_SX, FieldStatus, use_bound, use_field, use_field_frame},
        layout::use_box,
    },
    hooks::{PopoverOptions, use_element, use_popover, use_theme},
    sx::{StaticSx, Sx, sx},
    theme::{DateDefaults, Size, SizeCss, Z_INDEX_POPOVER},
};

static PICKER_FIELD_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    // Everything positional comes from `use_popover` as an inline style.
    sx().z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Sm))
        .background("white")
        .border_style("solid")
        .border_width("1px")
        .border_color("grey.3")
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .box_shadow("0 4px 8px rgba(0, 0, 0, 0.10), 0 8px 20px rgba(0, 0, 0, 0.14)")
});

/// The formats a field shows its value in, and reads typed text against.
#[derive(Clone)]
pub struct Formats {
    pub date: String,
    pub time: String,
    pub names: &'static DateDefaults,
}

/// A value a date or time field can hold.
pub trait FieldValue: Copy + PartialEq + 'static {
    /// The input's text.
    fn show(self, formats: &Formats) -> String;
    /// Typed text back. `current` is the field's value, for whatever the text
    /// leaves out.
    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable>;
    /// What the hidden input posts: ISO 8601.
    fn iso(self) -> String;
}

impl FieldValue for NaiveDate {
    fn show(self, formats: &Formats) -> String {
        format_date(self, &formats.date, formats.names)
    }

    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        let fallback_year = today.or(current).map(|day| day.year());
        parse_date(text, &formats.date, formats.names, fallback_year)
    }

    fn iso(self) -> String {
        self.to_string()
    }
}

impl FieldValue for NaiveTime {
    fn show(self, formats: &Formats) -> String {
        format_time(self, &formats.time, formats.names)
    }

    fn read(
        text: &str,
        formats: &Formats,
        _: Option<Self>,
        _: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        parse_time(text, formats.names)
    }

    fn iso(self) -> String {
        self.to_string()
    }
}

impl FieldValue for NaiveDateTime {
    fn show(self, formats: &Formats) -> String {
        format!(
            "{} {}",
            self.date().show(formats),
            self.time().show(formats)
        )
    }

    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        let fallback_year = today
            .or(current.map(|value| value.date()))
            .map(|day| day.year());
        parse_date_time(
            text,
            &formats.date,
            formats.names,
            fallback_year,
            current.map(|value| value.time()),
        )
    }

    fn iso(self) -> String {
        iso_date_time(self)
    }
}

impl<T: FieldValue + Ord> FieldValue for DateRange<T> {
    fn show(self, formats: &Formats) -> String {
        match self.end {
            Some(end) => format!(
                "{}{}{}",
                self.start.show(formats),
                formats.names.range_separator,
                end.show(formats)
            ),
            None => self.start.show(formats),
        }
    }

    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable> {
        let (start, end) = split_range(text);
        let start = T::read(start, formats, current.map(|range| range.start), today)?;
        let end = end
            .map(|end| {
                let current_end = current.and_then(|range| range.end).or(Some(start));
                T::read(end, formats, current_end, today)
            })
            .transpose()?;
        Ok(DateRange::new(start, end).ordered())
    }

    fn iso(self) -> String {
        format!(
            "{}/{}",
            self.start.iso(),
            self.end.map(T::iso).unwrap_or_default()
        )
    }
}

/// Everything a field hands the engine - its `field_props!` and what it
/// resolved from its own.
pub(super) struct PickerField<'a, V: 'static> {
    pub value: Option<V>,
    pub onchange: Option<EventHandler<Option<V>>>,
    pub validate: &'a Validators<Option<V>>,
    pub name: &'a FieldName<Option<V>>,
    pub formats: Formats,
    pub today: Option<NaiveDate>,
    pub placeholder: Option<String>,
    pub label: &'a Caption,
    pub description: &'a Caption,
    pub helper: &'a Caption,
    pub status: &'a Input<FieldStatus>,
    pub size: Input<Size>,
    pub radius: Input<Size>,
    pub required: Option<bool>,
    pub disabled: Option<bool>,
    pub class: &'a Input<ClassList>,
    pub sx: &'a Input<Sx>,
    pub states: &'a Input<States>,
    pub attributes: Vec<Attribute>,
}

/// What the dropdown's picker is drawn from.
pub struct DropdownArgs<V: 'static> {
    pub value: Option<V>,
    pub today: Option<NaiveDate>,
    pub size: Size,
    /// Emits a picked value; `true` also closes the dropdown.
    pub pick: Callback<(Option<V>, bool)>,
}

/// Builds a [`PickerField`] from a field's props, which all spell the shared
/// ones the same way.
macro_rules! picker_field {
    ($props:ident, $formats:expr, $today:expr) => {
        $crate::components::form::date::picker_field::PickerField {
            value: $props.value,
            onchange: $props.onchange,
            validate: &$props.validate,
            name: &$props.name,
            formats: $formats,
            today: $today,
            placeholder: $props.placeholder.clone(),
            label: &$props.label,
            description: &$props.description,
            helper: &$props.helper,
            status: &$props.status,
            size: $props.size.clone(),
            radius: $props.radius.clone(),
            required: $props.required,
            disabled: $props.disabled,
            class: &$props.class,
            sx: &$props.sx,
            states: &$props.states,
            attributes: $props.attributes,
        }
    };
}
pub(super) use picker_field;

/// Controlled: renders `value` and asks for a new one through `onchange`.
/// Typed text stays as typed until the field blurs or Enter is pressed; text
/// that `V` cannot read, or that `accepts` refuses, stays and shows
/// `DateDefaults::invalid_date`.
pub(super) fn use_picker_field<V: FieldValue>(
    field: PickerField<'_, V>,
    accepts: impl Fn(V) -> bool + Clone + 'static,
    dropdown: impl FnOnce(DropdownArgs<V>) -> Element,
) -> Element {
    let theme = use_theme();
    let names = &theme.date;
    let defaults = &theme.date_field;
    let size = field.size.copied_or(defaults.size);
    let radius = field.radius.copied_or(defaults.radius);
    let required = field.required.unwrap_or(false);
    let bound = use_bound(field.name, field.onchange.is_some());
    let disabled = bound.disabled(field.disabled);
    let value = bound.value().unwrap_or(field.value);
    let today = use_today(field.today);
    let formats = field.formats;

    let mut opened = use_signal(|| false);
    // The text as typed, until it is committed. `None` shows `value`.
    let mut draft = use_signal(|| Option::<String>::None);
    // The last commit found nothing it accepts; cleared by the next keystroke.
    let mut rejected = use_signal(|| false);

    let onchange = field.onchange;
    let setter = bound.setter();
    let emit = move |next: Option<V>| match (&onchange, &setter) {
        (Some(onchange), _) => onchange.call(next),
        (None, Some(setter)) => setter.set(next),
        (None, None) => {}
    };
    let text =
        draft().unwrap_or_else(|| value.map(|value| value.show(&formats)).unwrap_or_default());
    let commit = {
        let emit = emit.clone();
        move || {
            let Some(text) = draft.peek().clone() else {
                return;
            };
            if text.trim().is_empty() {
                draft.set(None);
                emit(None);
                return;
            }
            match V::read(&text, &formats, value, today) {
                Ok(next) if accepts(next) => {
                    draft.set(None);
                    emit(Some(next));
                }
                _ => rejected.set(true),
            }
        }
    };
    let (mut commit_on_blur, mut commit_on_enter) = (commit.clone(), commit);

    // Unaccepted text outranks the caller's status: the field cannot hold what
    // it shows.
    let status: Input<FieldStatus> = match rejected() {
        true => Input::Value(FieldStatus::Error(names.invalid_date.to_string())),
        false => field.status.clone(),
    };
    let field_box = use_field()
        .label(field.label)
        .description(field.description)
        .helper(field.helper)
        .status(&status)
        .rules(field.validate.check(&value))
        .bound(&bound)
        .required(required)
        .disabled(disabled)
        .size(size)
        .radius(radius)
        .class(field.class)
        .sx(field.sx)
        .states(field.states)
        .attributes(&field.attributes)
        .prepare();

    let frame = use_field_frame().states(field_box.states()).prepare();
    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // Every one of these is a hook, so all of them run before anything
    // branches on `opened`.
    let anchor = use_element();
    let showing = opened() && !disabled;
    let popover = use_popover(
        anchor,
        showing,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding),
    );
    let dropdown_box = use_box()
        .framework_sx(&PICKER_FIELD_DROPDOWN_SX)
        .style(popover.style())
        .prepare();

    let input = field_box
        .aria(control)
        .attr_default("type", "text")
        .attr("value", text)
        .attr("data-controlled", true)
        .attr("placeholder", field.placeholder)
        .attr("disabled", disabled)
        .attr("required", required)
        .attr("autocomplete", "off")
        .attr("aria-haspopup", "dialog")
        .attr("aria-expanded", showing.to_string())
        .event("oninput", move |event: FormEvent| {
            rejected.set(false);
            draft.set(Some(event.value()));
        })
        .event("onfocus", move |_: FocusEvent| opened.set(true))
        .event("onclick", move |_: MouseEvent| opened.set(true))
        .event("onblur", move |_: FocusEvent| {
            opened.set(false);
            commit_on_blur();
        })
        .event("onkeydown", move |event: KeyboardEvent| match event.key() {
            Key::Enter => commit_on_enter(),
            Key::Escape if opened() => {
                event.prevent_default();
                opened.set(false);
            }
            _ => {}
        })
        .render(HtmlTag::Input, field.attributes, ());

    // The text shows a format; the form gets ISO 8601.
    let hidden = bound.name().map(|name| {
        rsx! {
            input {
                r#type: "hidden",
                name: name.to_string(),
                value: value.map(FieldValue::iso).unwrap_or_default(),
            }
        }
    });

    // Portaled, so no `overflow: hidden` ancestor clips it. The pickers inside
    // are not focusable, and a mousedown anywhere in the box is cancelled: the
    // text input keeps focus throughout, and its blur is what closes the box.
    popover.show(showing.then(|| {
        let pick = Callback::new(move |(next, close): (Option<V>, bool)| {
            draft.set(None);
            rejected.set(false);
            emit(next);
            if close {
                opened.set(false);
            }
        });
        dropdown_box
            .element(popover.floating())
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            .render(
                HtmlTag::Div,
                Vec::new(),
                dropdown(DropdownArgs {
                    value,
                    today,
                    size,
                    pick,
                }),
            )
    }));

    let control = rsx! {
        div { onmounted: anchor.mount(),
            {frame.render(input)}
            {hidden}
        }
    };
    field_box.render(control)
}
