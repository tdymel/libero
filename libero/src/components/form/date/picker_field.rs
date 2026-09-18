//! The engine every date and time field shares: a text input read on blur or
//! Enter, a dropdown holding a picker, and a hidden input posting ISO 8601.
//! Generic over the value, but private - `DateField` and the typed fields
//! reach it through `date_field::date_field`.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;

use super::{
    DateRange,
    calendar::DateLevel,
    format::{format_date, format_time},
    parse::{Unreadable, parse_at},
    parse_time::{parse_date_time, parse_time},
    range::{iso_date_time, split_range},
    today::use_today,
};
use crate::{
    components::{
        Caption, ClassList, FieldName, HtmlTag, Input, States, Validators,
        common::{FOCUSABLE_SELECTOR, NavigationChord, navigation_chord},
        form::{
            FIELD_CONTROL_SX, FieldStatus, use_announcer, use_bound, use_field, use_field_frame,
        },
        layout::{paper_sx, use_box},
    },
    hooks::{
        PopoverOptions, use_element, use_field_list_layer, use_focus_within, use_localization,
        use_popover_on, use_theme,
    },
    localization::DateLocale,
    platform::{ElementApi, next_task},
    sx::{StaticSx, Sx},
    theme::{Size, SizeCss, Z_INDEX_POPOVER},
};

static PICKER_FIELD_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    // A surface, so the background and the `bordered` border are
    // `paper_sx()`'s. Everything positional comes from `use_popover` as an
    // inline style.
    paper_sx()
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Sm))
        // The field's own corner rather than the surface default, and a
        // dropdown floats over the page, where that default rests.
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
});

/// Where Arrow Down in the text input puts focus: the picker's own tab stop in
/// the days, the months or years, or the clock - not the navigation above it.
const DROPDOWN_ENTRY: &str = ":is([data-slot='months'], [data-slot='cells'], [data-slot='columns']) [tabindex='0'], [data-slot='face'][tabindex='0']";

/// The formats a field shows its value in, and reads typed text against.
#[derive(Clone)]
pub struct Formats {
    pub date: String,
    pub time: String,
    pub names: &'static DateLocale,
    /// Between a range's two ends, from `localization::Formats`.
    pub range_separator: &'static str,
    /// What a `NaiveDate` stands for: typed text reads as its first day.
    pub level: DateLevel,
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
    /// Names the dropdown's dialog.
    fn dialog_label(names: &DateLocale) -> &'static str {
        names.date_label
    }
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
        parse_at(
            text,
            &formats.date,
            formats.names,
            fallback_year,
            formats.level,
        )
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
        parse_time(text, &formats.time, formats.names)
    }

    fn iso(self) -> String {
        self.to_string()
    }

    fn dialog_label(names: &DateLocale) -> &'static str {
        names.time_label
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
            &formats.time,
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
                formats.range_separator,
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
        let (start, end) = split_range(text, formats.range_separator);
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

    fn dialog_label(names: &DateLocale) -> &'static str {
        T::dialog_label(names)
    }
}

/// Everything a field hands the engine from its `field_props!`.
pub(super) struct PickerField<'a, V: 'static> {
    pub value: Option<V>,
    pub onchange: Option<EventHandler<Option<V>>>,
    pub validate: &'a Validators<Option<V>>,
    pub name: &'a FieldName<Option<V>>,
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
    pub readonly: Option<bool>,
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
    ($props:ident, $today:expr) => {
        $crate::components::form::date::picker_field::PickerField {
            value: $props.value,
            onchange: $props.onchange,
            validate: &$props.validate,
            name: &$props.name,
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
            readonly: $props.readonly,
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
/// that `V` cannot read stays and shows `DateLocale::invalid_date`, a value
/// `accepts` refuses the error it returns.
pub(super) fn use_picker_field<V: FieldValue>(
    field: PickerField<'_, V>,
    formats: Formats,
    accepts: impl Fn(V) -> Result<(), String> + Clone + 'static,
    dropdown: impl FnOnce(DropdownArgs<V>) -> Element,
) -> Element {
    let theme = use_theme();
    let names = &use_localization().date;
    let defaults = &theme.date_field;
    let size = field.size.copied_or(defaults.size);
    let radius = field.radius.copied_or(defaults.radius);
    let required = field.required.unwrap_or(false);
    let bound = use_bound(field.name, field.onchange.is_some());
    let disabled = bound.disabled(field.disabled);
    // Two editors again: the text (native `readonly`) and the dropdown, which
    // is refused rather than opened - so focus, Tab and the form post are
    // untouched.
    let readonly = field.readonly.unwrap_or(false);
    let value = bound.value().unwrap_or(field.value);
    let today = use_today(field.today);

    let mut opened = use_signal(|| false);
    // The text as typed, until it is committed. `None` shows `value`.
    let mut draft = use_signal(|| Option::<String>::None);
    // Why the last commit found nothing it accepts; cleared by the next
    // keystroke.
    let mut rejected = use_signal(|| Option::<String>::None);
    // Focus is coming back to the text input from the dropdown, which closed:
    // that focus must not open it again.
    let mut returning = use_signal(|| false);
    // Arrow Down asked for focus in the picker, once the dropdown is drawn.
    let mut entering = use_signal(|| false);

    let onchange = field.onchange;
    let setter = bound.setter();
    let emit = move |next: Option<V>| match (&onchange, &setter) {
        (Some(onchange), _) => onchange.call(next),
        (None, Some(setter)) => setter.set(next),
        (None, None) => {}
    };
    let text =
        draft().unwrap_or_else(|| value.map(|value| value.show(&formats)).unwrap_or_default());
    // Says a refusal on Enter, where focus stays and the error line is not live.
    let announcer = use_announcer();
    // Returns the error when the text was refused.
    let commit = {
        let emit = emit.clone();
        move || {
            let text = draft.peek().clone()?;
            if text.trim().is_empty() {
                draft.set(None);
                emit(None);
                return None;
            }
            let read =
                V::read(&text, &formats, value, today).map_err(|_| names.invalid_date.to_string());
            match read.and_then(|next| accepts(next).map(|()| next)) {
                Ok(next) => {
                    draft.set(None);
                    emit(Some(next));
                    None
                }
                Err(error) => {
                    rejected.set(Some(error.clone()));
                    Some(error)
                }
            }
        }
    };
    let (commit_on_blur, mut commit_on_enter) = (commit.clone(), commit.clone());
    let mut commit_on_enter_picker = commit;

    // Unaccepted text outranks the caller's status: the field cannot hold what
    // it shows.
    let status: Input<FieldStatus> = match rejected() {
        Some(error) => Input::Value(FieldStatus::Error(error)),
        None => field.status.clone(),
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
    let showing = opened() && !disabled && !readonly;
    // On the Escape stack exactly while the key handler below would take
    // Escape, so a `HoverCard` around this field leaves the press to it.
    use_field_list_layer(opened() && !readonly);
    let popover = use_popover_on(
        anchor,
        use_element(),
        showing,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding),
    );
    let floating = *popover.floating();
    let dropdown_states: Input<States> = States::new().active("bordered").into();
    let dropdown_box = use_box()
        .framework_sx(&PICKER_FIELD_DROPDOWN_SX)
        .states(&dropdown_states)
        .style(popover.style())
        .prepare();
    // Waits for placement too: Arrow Down on a closed field opens it, and the
    // picker is not drawn until the box has been measured.
    use_effect(move || {
        if !entering() || !popover.placed() {
            return;
        }
        entering.set(false);
        let _ = floating
            .query_selector(DROPDOWN_ENTRY)
            .and_then(|element| element.focus());
    });

    // After the platform's next task focus has landed, so this can tell
    // whether it went somewhere else in the field - the text input or the
    // dropdown - or left. Without a platform answer it counts as left.
    let settle = move || {
        spawn(async move {
            next_task().await;
            let inside = anchor.query_selector(":focus").is_ok()
                || floating.query_selector(":focus").is_ok();
            if !inside {
                opened.set(false);
            }
        });
    };
    let mut focus_input = move || {
        returning.set(true);
        if anchor
            .query_selector("input[data-controlled]")
            .and_then(|input| input.focus())
            .is_err()
        {
            returning.set(false);
        }
    };
    let focused = move || match (returning(), readonly) {
        (true, _) | (_, true) => returning.set(false),
        (false, false) => opened.set(true),
    };
    // Element 0 is the text input, 1 the dropdown; the box closes once focus
    // is in neither.
    let focus = use_focus_within(
        move || vec![anchor.mounted(), floating.mounted()],
        move |change| {
            let mut opened = opened;
            match (change.element, change.within) {
                (0, true) => {
                    let mut focused = focused;
                    focused();
                }
                (0, false) => {
                    commit_on_blur.clone()();
                }
                _ => {}
            }
            match change.in_group {
                Some(true) => {}
                Some(false) => opened.set(false),
                None => settle(),
            }
        },
    );

    let dialog_id = format!("{}-dialog", field_box.id());
    let input = field_box
        .aria(control)
        .attr_default("type", "text")
        .attr("value", text)
        .attr("data-controlled", true)
        .attr("placeholder", field.placeholder)
        .attr("disabled", disabled)
        .attr("readonly", readonly)
        .attr("required", required)
        .attr("autocomplete", "off")
        // APG Date Picker Combobox: a textbox may not carry `aria-expanded`.
        .attr("role", "combobox")
        .attr("aria-haspopup", "dialog")
        .attr("aria-expanded", showing.to_string())
        .attr("aria-controls", showing.then(|| dialog_id.clone()))
        .event("oninput", move |event: FormEvent| {
            rejected.set(None);
            announcer.clear();
            draft.set(Some(event.value()));
        })
        .event("onfocus", focus.focusin(0))
        .event("onclick", move |_: MouseEvent| {
            if !readonly {
                opened.set(true);
            }
        })
        .event("onblur", focus.focusout(0))
        .event("onkeydown", move |event: KeyboardEvent| match event.key() {
            _ if readonly => {}
            Key::Enter => {
                if let Some(error) = commit_on_enter() {
                    announcer.say(error);
                }
            }
            // APG: Alt+ArrowDown enters like ArrowDown; Ctrl/Meta is the caret's.
            Key::ArrowDown if navigation_chord(&event) != Some(NavigationChord::Browser) => {
                event.prevent_default();
                // The picker then enters on the typed day, not the one it showed.
                commit_on_enter_picker();
                opened.set(true);
                entering.set(true);
            }
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

    // Portaled, so no `overflow: hidden` ancestor clips it. A mousedown in the
    // box is cancelled, so a click keeps focus on the text input; the keyboard
    // enters the picker with Arrow Down and leaves it with Escape. The box
    // closes once focus is in neither.
    popover.show(showing.then(|| {
        let pick = Callback::new(move |(next, close): (Option<V>, bool)| {
            draft.set(None);
            rejected.set(None);
            emit(next);
            if close {
                // The focused cell is about to go; focus goes back first.
                if floating.query_selector(":focus").is_ok() {
                    focus_input();
                }
                opened.set(false);
            }
        });
        dropdown_box
            .element(popover.floating())
            .attr("id", dialog_id)
            .attr("role", "dialog")
            .attr("aria-label", V::dialog_label(names))
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            .event("onfocusout", focus.focusout(1))
            .event("onkeydown", move |event: KeyboardEvent| match event.key() {
                Key::Escape => {
                    event.prevent_default();
                    focus_input();
                    opened.set(false);
                }
                // Portaled after the page: Tab past either end goes back
                // through the text input, then on as from the field.
                Key::Tab => {
                    let Ok(stops) = floating.query_selector_all(FOCUSABLE_SELECTOR) else {
                        return;
                    };
                    let backwards = event.modifiers().shift();
                    let edge = if backwards {
                        stops.first()
                    } else {
                        stops.last()
                    };
                    if edge.is_some_and(|stop| stop.is_focused()) {
                        if backwards {
                            event.prevent_default();
                        }
                        focus_input();
                    }
                }
                _ => {}
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
            {announcer.render()}
        }
    };
    field_box.render(control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::Stylesheet;

    /// The dropdown is a `Paper` surface with two overrides. The `bordered`
    /// token it is rendered with only exists in a browser - the box opens on
    /// an event.
    #[test]
    fn the_dropdown_starts_from_the_paper_surface() {
        let css = Stylesheet::from(&*PICKER_FIELD_DROPDOWN_SX);
        let css = css.as_str();

        assert!(
            css.contains("background:var(--lsx-paper-background);"),
            "{css}"
        );
        assert!(
            css.contains("--lsx-focus-contrast:var(--lsx-paper-contrast);"),
            "{css}"
        );
        assert!(
            css.contains("border:1px solid var(--lsx-paper-border-color);"),
            "{css}"
        );
        // The overrides replace the surface defaults rather than race them.
        assert!(css.contains("border-radius:var(--lsx-radius-sm);"), "{css}");
        assert!(css.contains("box-shadow:var(--lsx-shadow-lg);"), "{css}");
        assert!(!css.contains("var(--lsx-paper-radius)"), "{css}");
        assert!(!css.contains("var(--lsx-paper-shadow)"), "{css}");
    }

    fn wave_dash_formats() -> Formats {
        Formats {
            date: "YYYY年M月D日".to_string(),
            time: crate::localization::Formats::AMERICAN.time.to_string(),
            names: &DateLocale::ENGLISH,
            range_separator: " ～ ",
            level: DateLevel::Day,
        }
    }

    #[test]
    fn ranges_read_back_what_they_show_with_the_theme_separator() {
        let formats = wave_dash_formats();
        let day = |day| NaiveDate::from_ymd_opt(2026, 9, day).expect("a real day");
        let range = DateRange::new(day(1), Some(day(5)));
        let text = range.show(&formats);
        assert_eq!(text, "2026年9月1日 ～ 2026年9月5日");
        assert_eq!(DateRange::read(&text, &formats, None, None), Ok(range));

        let moments = DateRange::new(
            day(1).and_hms_opt(9, 0, 0).expect("a real time"),
            Some(day(5).and_hms_opt(17, 30, 0).expect("a real time")),
        );
        let text = moments.show(&formats);
        assert_eq!(DateRange::read(&text, &formats, None, None), Ok(moments));
    }

    #[test]
    fn months_and_years_read_back_what_they_show() {
        let names = &DateLocale::ENGLISH;
        let american = crate::localization::Formats::AMERICAN;
        let at = |date: &str, level| Formats {
            date: date.to_string(),
            time: american.time.to_string(),
            names,
            range_separator: american.range_separator,
            level,
        };
        let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).expect("a real day");
        let month = at((american.date)(DateLevel::Month), DateLevel::Month);
        assert_eq!(day(9, 1).show(&month), "September 2026");
        for text in ["September 2026", "sep 2026", "9/2026", "09.2026"] {
            assert_eq!(
                NaiveDate::read(text, &month, None, None),
                Ok(day(9, 1)),
                "{text}"
            );
        }
        // Typed text lands on the month's first day.
        let text = day(9, 25).show(&month);
        assert_eq!(NaiveDate::read(&text, &month, None, None), Ok(day(9, 1)));

        let year = at((american.date)(DateLevel::Year), DateLevel::Year);
        assert_eq!(day(9, 25).show(&year), "2026");
        assert_eq!(NaiveDate::read("2026", &year, None, None), Ok(day(1, 1)));

        let japanese = at("YYYY年M月", DateLevel::Month);
        assert_eq!(day(9, 1).show(&japanese), "2026年9月");
        assert_eq!(
            NaiveDate::read("2026年9月", &japanese, None, None),
            Ok(day(9, 1))
        );
    }
}
