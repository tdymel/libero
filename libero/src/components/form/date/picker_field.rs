//! The engine every date and time field shares: a text input read on blur or Enter,
//! a dropdown holding a picker, and a hidden input posting ISO 8601.

use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime};
use dioxus::prelude::*;

use super::{
    DateRange,
    calendar::DateLevel,
    flows::{short_day, short_days},
    format::{format_date, format_time},
    parse::{Unreadable, parse_at},
    parse_time::{parse_date_time, parse_time},
    range::{iso_date_time, split_range},
    today::use_today_on_open,
};
use crate::{
    components::{
        accessibility::use_announcer,
        common::{
            ClassList, FOCUSABLE_SELECTOR, HtmlTag, Input, NavigationChord, Part, Parts,
            ScaleOrCss, States, navigation_chord,
        },
        form::{
            Caption, ChronoDropdownPart, FIELD_CONTROL_SX, FieldName, FieldPart, FieldStatus,
            FormScope, Validators,
            keyboardless::{PickerInput, use_keyboardless},
            phone_sheet::{
                SheetSwipe, narrow_query, phone_sheet_sx, sheet_clearance_style,
                use_sheet_clearance, use_sheet_room, use_sheet_swipe,
            },
            use_bound, use_field, use_field_frame,
        },
        layout::{paper_sx, use_box},
    },
    hooks::{
        POPOVER_AVAILABLE_HEIGHT, PopoverOptions, placement_sx, use_element, use_field_list_layer,
        use_focus_within, use_form_owner, use_localization, use_media_query, use_popover_on,
        use_theme,
    },
    localization::DateLocale,
    platform::{ElementApi, is_tab, next_task},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{Size, SizeCss, Z_INDEX_POPOVER},
};

static PICKER_FIELD_DROPDOWN_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .and(placement_sx(true))
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Sm))
        // The field's corner, and a floating shadow over the surface's resting one.
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .box_shadow(SizeCss::SHADOW.value(Size::Lg))
        // Never past the room on its side: it scrolls instead (WCAG 1.4.10).
        .max_height(POPOVER_AVAILABLE_HEIGHT.value_or("none"))
        .overflow_y("auto")
        .selector(
            "& > [data-slot='text-input']",
            sx().margin_top(SizeCss::SPACING.value(Size::Sm)),
        )
        // Below `sm` the phone sheet, which a sticky header cannot cover (todo 3900).
        .and(phone_sheet_sx())
        .media(
            narrow_query(),
            // The clock face fills the sheet's width (3926); `div` outranks the picker's own rule.
            sx().selector("& :has(> div[data-slot='face'])", sx().width("100%"))
                .selector(
                    "& div[data-slot='face']",
                    sx().width("min(100%, 50dvh)")
                        .height("auto")
                        .with("aspect-ratio", "1"),
                ),
        )
});

/// Where Arrow Down in the input puts focus: the picker's tab stop, not the navigation above it.
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
    /// A narrow field's text, where `show` would overflow it; `None` keeps `show`.
    fn show_short(self, _: &Formats) -> Option<String> {
        None
    }
    /// A range of these as a narrow field shows it.
    fn show_short_range(_: DateRange<Self>, _: &Formats) -> Option<String> {
        None
    }
    /// Typed text back. `current` fills in whatever the text leaves out.
    fn read(
        text: &str,
        formats: &Formats,
        current: Option<Self>,
        today: Option<NaiveDate>,
    ) -> Result<Self, Unreadable>;
    /// What the hidden input posts: ISO 8601.
    fn iso(self) -> String;
    /// Whether the value counts as one: a range still missing its end posts nothing and
    /// fails a `required` rule.
    fn is_complete(self) -> bool {
        true
    }
    /// Names the dropdown's dialog.
    fn dialog_label(names: &DateLocale) -> &'static str {
        names.date_label
    }
    /// The error for text `read` cannot read.
    fn unreadable(names: &DateLocale) -> &'static str {
        names.invalid_date
    }
}

impl FieldValue for NaiveDate {
    fn show(self, formats: &Formats) -> String {
        format_date(self, &formats.date, formats.names)
    }

    fn show_short_range(range: DateRange<Self>, formats: &Formats) -> Option<String> {
        Some(short_days(range, formats.names))
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

    /// `14 Sep 09:00 – 16 Sep 17:00`, years only when they differ.
    fn show_short_range(range: DateRange<Self>, formats: &Formats) -> Option<String> {
        let years = range
            .end
            .is_some_and(|end| end.year() != range.start.year());
        let moment = |at: Self| {
            let day = short_day(at.date(), years, formats.names);
            format!("{day} {}", at.time().show(formats))
        };
        Some(match range.end {
            Some(end) if end.date() == range.start.date() => {
                format!("{} – {}", moment(range.start), end.time().show(formats))
            }
            Some(end) => format!("{} – {}", moment(range.start), moment(end)),
            None => format!("{} –", moment(range.start)),
        })
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

    fn show_short(self, formats: &Formats) -> Option<String> {
        T::show_short_range(self, formats)
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

    fn is_complete(self) -> bool {
        self.end.is_some()
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
    pub radius: Input<ThemeAwareValue>,
    pub required: Option<bool>,
    pub disabled: Option<bool>,
    pub readonly: Option<bool>,
    pub class: &'a Input<ClassList>,
    pub sx: &'a Input<Sx>,
    pub parts: &'a Input<Parts<FieldPart>>,
    pub dropdown_parts: &'a Input<Parts<ChronoDropdownPart>>,
    pub states: &'a Input<States>,
    pub attributes: Vec<Attribute>,
}

/// What the dropdown's picker is drawn from.
pub struct DropdownArgs<V: 'static> {
    pub value: Option<V>,
    pub today: Option<NaiveDate>,
    pub size: Size,
    /// Emits a picked value, closing the dropdown when the field's close rule says so.
    pub pick: Callback<Option<V>>,
}

/// Builds a [`PickerField`] from a field's props.
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
            parts: &$props.parts,
            dropdown_parts: &$props.dropdown_parts,
            states: &$props.states,
            attributes: $props.attributes,
        }
    };
}
pub(super) use picker_field;

/// Typed text stays until blur or Enter. Unreadable or refused text stays and shows an error.
pub(super) fn use_picker_field<V: FieldValue>(
    field: PickerField<'_, V>,
    formats: Formats,
    accepts: impl Fn(V) -> Result<(), String> + Clone + 'static,
    closes: impl Fn(Option<V>) -> bool + 'static,
    dropdown: impl FnOnce(DropdownArgs<V>) -> Element,
) -> Element {
    let theme = use_theme();
    let names = &use_localization().date;
    let defaults = &theme.chrono_field;
    let size = field.size.copied_or(defaults.size);
    let radius = ScaleOrCss::new(field.radius.as_ref(), defaults.radius);
    let required = field.required.unwrap_or(false);
    let bound = use_bound(field.name, field.onchange.is_some());
    let disabled = bound.disabled(field.disabled);
    // Readonly: native on the text; the dropdown refuses to open, leaving focus and posting alone.
    let readonly = field.readonly.unwrap_or(false);
    let value = bound.value().unwrap_or(field.value);
    let opened = use_signal(|| false);
    let today = use_today_on_open(field.today, opened());
    // Peek-compared: focus then click both open, and an unchanged `set` still redraws.
    let open_to = move |next: bool| {
        let mut opened = opened;
        if *opened.peek() != next {
            opened.set(next);
        }
    };
    // The text as typed, until it is committed. `None` shows `value`.
    let mut draft = use_signal(|| Option::<String>::None);
    // Why the last commit was refused; cleared by the next keystroke.
    let mut rejected = use_signal(|| Option::<String>::None);
    // Arrow Down asked for focus in the picker, once the dropdown is drawn.
    let mut entering = use_signal(|| false);
    let anchor = use_element();
    // A form reset drops the typed text and its refusal; a raw `<form>` is heard off the DOM.
    let scope = try_use_context::<FormScope>();
    let owner = use_form_owner(anchor, scope.is_none());
    let resets = scope.map_or(0, |form| form.resets()) + owner().unwrap_or(0);
    use_effect(use_reactive!(|resets| {
        let _ = resets;
        if draft.peek().is_some() {
            draft.set(None);
        }
        if rejected.peek().is_some() {
            rejected.set(None);
        }
    }));

    let onchange = field.onchange;
    let setter = bound.setter();
    let emit = move |next: Option<V>| match (&onchange, &setter) {
        (Some(onchange), _) => onchange.call(next),
        (None, Some(setter)) => setter.set(next),
        (None, None) => {}
    };
    // Below `sm` a range shows short while the input is not focused, so typed text never
    // reads it back (todo 3904).
    let narrow = use_media_query(&narrow_query());
    let focused = use_signal(|| false);
    let short = narrow() && !focused();
    let text = draft().unwrap_or_else(|| {
        value
            .map(|value| {
                short
                    .then(|| value.show_short(&formats))
                    .flatten()
                    .unwrap_or_else(|| value.show(&formats))
            })
            .unwrap_or_default()
    });
    // The picker's own text input always shows the whole text.
    let full_text =
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
            let read = V::read(&text, &formats, value, today)
                .map_err(|_| V::unreadable(names).to_string());
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
    let (mut commit_on_enter_picker, mut commit_in_picker) = (commit.clone(), commit.clone());
    let mut commit_on_picker_blur = commit;

    // Refused text outranks the caller's status: the field cannot hold what it shows.
    let status: Input<FieldStatus> = match rejected() {
        Some(error) => Input::Value(FieldStatus::Error(error)),
        None => field.status.clone(),
    };
    let field_box = use_field()
        .label(field.label)
        .description(field.description)
        .helper(field.helper)
        .status(&status)
        .rules(
            field
                .validate
                .check(&value.filter(|value| value.is_complete())),
        )
        .bound(&bound)
        .required(required)
        .empty(!value.is_some_and(|value| value.is_complete()))
        .readonly(readonly)
        .disabled(disabled)
        .size(size)
        .radius(radius.clone())
        .class(field.class)
        .sx(field.sx)
        .parts(field.parts)
        .states(field.states)
        .attributes(&field.attributes)
        .prepare();

    let frame = use_field_frame()
        .states(field_box.states())
        .placeholder(field.placeholder.as_deref())
        .prepare();
    // The frame draws the ring, so the control must not draw a second one.
    let control = use_box()
        .framework_sx(&FIELD_CONTROL_SX)
        .focus_ring(false)
        .prepare();

    // Hooks: all run before anything branches on `opened`.
    let showing = opened() && !disabled && !readonly;
    let popover = use_popover_on(
        anchor,
        use_element(),
        showing,
        PopoverOptions::new(theme.popover.gap, theme.popover.padding),
    );
    let floating = *popover.floating();
    let room = use_sheet_room(floating, narrow() && popover.placed());
    let clearance = use_sheet_clearance(
        anchor,
        field_box.root(),
        popover,
        showing,
        theme.popover.gap,
    );
    let dropdown_states: Input<States> = States::new().active("bordered").into();
    let dropdown_box = use_box()
        .framework_sx(&PICKER_FIELD_DROPDOWN_SX)
        .parts(field.dropdown_parts)
        .states(&dropdown_states)
        .style(Some(format!("{}{room}", popover.vars())))
        .prepare();
    let keyboardless = use_keyboardless();
    let close = use_callback(move |()| open_to(false));
    let SheetSwipe { strip, listeners } = use_sheet_swipe(close);
    // Waits for placement: the picker is not drawn until the box is measured.
    use_effect(move || {
        if !entering() || !popover.placed() {
            return;
        }
        entering.set(false);
        let _ = floating
            .query_selector(DROPDOWN_ENTRY)
            .and_then(|element| element.focus());
    });

    // After the next task focus has landed: closes unless it is in the input or dropdown.
    // Without a platform answer it counts as left.
    let press = keyboardless.press();
    let settle = move || {
        // A tap into the picker's text box stays: a WebView answers no `:focus` query (3905).
        if press.taken() {
            return;
        }
        spawn(async move {
            next_task().await;
            let inside = anchor.query_selector(":focus").is_ok()
                || floating.query_selector(":focus").is_ok();
            if !inside {
                open_to(false);
            }
        });
    };
    let focus_input = move || {
        let _ = anchor
            .query_selector("input[data-controlled]")
            .and_then(|input| input.focus());
    };
    // On the Escape stack exactly while the handler below takes Escape, so a `HoverCard` leaves it.
    use_field_list_layer(
        opened() && !readonly,
        use_callback(move |()| {
            if floating.query_selector(":focus").is_ok() {
                focus_input();
            }
            open_to(false);
        }),
    );
    // Element 0 is the text input, 1 the dropdown; closes once focus is in neither.
    // Focus alone does not open (WCAG 3.2.1): Arrow Down or a click does.
    let focus = use_focus_within(
        move || vec![anchor.mounted(), floating.mounted()],
        move |change| {
            if change.element == 0 && *focused.peek() != change.within {
                let mut focused = focused;
                focused.set(change.within);
            }
            if let (0, false) = (change.element, change.within) {
                commit_on_blur.clone()();
            }
            match change.in_group {
                Some(true) => {}
                Some(false) => open_to(false),
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
        // A phone's keyboard would cover the picker, which is the input there (todos 1012, 3905).
        .attr("inputmode", keyboardless.inputmode())
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
                open_to(true);
            }
        })
        .event("onblur", focus.focusout(0))
        .event("onkeydown", move |event: KeyboardEvent| match event.key() {
            // A composing Enter picks an IME candidate, it does not commit the draft.
            _ if readonly || event.is_composing() => {}
            Key::Named(NamedKey::Enter) => {
                if let Some(error) = commit_on_enter() {
                    announcer.say(error);
                }
            }
            // APG: Alt+ArrowDown enters like ArrowDown; Ctrl/Meta is the caret's.
            Key::Named(NamedKey::ArrowDown)
                if navigation_chord(&event) != Some(NavigationChord::Browser) =>
            {
                event.prevent_default();
                // The picker then enters on the typed day, not the one it showed.
                commit_on_enter_picker();
                open_to(true);
                entering.set(true);
            }
            Key::Named(NamedKey::Escape) if opened() => {
                event.prevent_default();
                open_to(false);
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
                // A disabled field posts nothing (todo 3873).
                disabled,
                value: value
                    .filter(|value| value.is_complete())
                    .map(FieldValue::iso)
                    .unwrap_or_default(),
            }
        }
    });

    // Portaled past `overflow: hidden`. Mousedown is cancelled so a click keeps focus on the input;
    // the keyboard enters with Arrow Down and leaves with Escape.
    // Stable across renders, so the picker memoizes while the field redraws.
    let pick = use_callback(move |next: Option<V>| {
        draft.set(None);
        rejected.set(None);
        emit(next);
        if closes(next) {
            // The focused cell is about to go; focus goes back first.
            if floating.query_selector(":focus").is_ok() {
                focus_input();
            }
            open_to(false);
        }
    });
    // Shares the field's draft: Enter commits and hands focus back, a blur commits (3905).
    let picker_input = keyboardless.input(PickerInput {
        field_id: field_box.id().to_string(),
        labelled: !field.label.is_none(),
        fallback_label: V::dialog_label(names),
        value: full_text,
        invalid: rejected().is_some(),
        oninput: move |event: FormEvent| {
            rejected.set(None);
            announcer.clear();
            draft.set(Some(event.value()));
        },
        onkeydown: move |event: KeyboardEvent| {
            if event.key() != Key::Named(NamedKey::Enter) || event.is_composing() {
                return;
            }
            match commit_in_picker() {
                Some(error) => announcer.say(error),
                None => {
                    focus_input();
                    open_to(false);
                }
            }
        },
        onblur: move |_: FocusEvent| {
            commit_on_picker_blur();
        },
    });
    popover.show(showing.then(|| {
        dropdown_box
            .element(popover.floating())
            .attr("id", dialog_id)
            .attr("data-slot", ChronoDropdownPart::Panel.slot())
            .attr("role", "dialog")
            .attr("aria-label", V::dialog_label(names))
            .event("onmousedown", move |event: MouseEvent| {
                event.prevent_default()
            })
            .event("onfocusout", focus.focusout(1))
            .event("onkeydown", move |event: KeyboardEvent| match event.key() {
                Key::Named(NamedKey::Escape) => {
                    event.prevent_default();
                    focus_input();
                    open_to(false);
                }
                // Portaled after the page: Tab past either end goes back via the input.
                _ if is_tab(&event) => {
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
                listeners,
                rsx! {
                    {strip}
                    {dropdown(DropdownArgs {
                        value,
                        today,
                        size,
                        pick,
                    })}
                    {picker_input}
                },
            )
    }));

    let control = rsx! {
        div { onmounted: anchor.mount(), style: sheet_clearance_style(clearance),
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

    /// The dropdown is a `Paper` surface with two overrides. Its `bordered` token needs a browser.
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

    /// Below `sm` the dropdown is the phone sheet, and a clock face fills it (3900, 3926).
    #[test]
    fn the_dropdown_becomes_the_phone_sheet_below_sm() {
        let css = Stylesheet::from(&*PICKER_FIELD_DROPDOWN_SX);
        let css = css.as_str();
        let narrow = css
            .find(&format!("@media {}", narrow_query()))
            .unwrap_or_else(|| panic!("no narrow block: {css}"));
        assert!(
            css[narrow..].contains("bottom:var(--lsx-sheet-foot, 0px)"),
            "{css}"
        );
        assert!(css[narrow..].contains("div[data-slot='face']"), "{css}");
    }

    #[test]
    fn a_narrow_range_shows_short() {
        let formats = Formats {
            date: "D. MMMM YYYY".to_string(),
            time: "HH:mm".to_string(),
            names: &DateLocale::ENGLISH,
            range_separator: " – ",
            level: DateLevel::Day,
        };
        let day = |month, day| NaiveDate::from_ymd_opt(2026, month, day).expect("a real day");
        let at = |month, date, hour| day(month, date).and_hms_opt(hour, 0, 0).expect("a time");
        let days = DateRange::new(day(9, 14), Some(day(9, 16)));
        assert_eq!(days.show_short(&formats).as_deref(), Some("14–16 Sep"));
        let moments = DateRange::new(at(9, 14, 9), Some(at(9, 16, 17)));
        assert_eq!(
            moments.show_short(&formats).as_deref(),
            Some("14 Sep 09:00 – 16 Sep 17:00")
        );
        let one_day = DateRange::new(at(9, 14, 9), Some(at(9, 14, 17)));
        assert_eq!(
            one_day.show_short(&formats).as_deref(),
            Some("14 Sep 09:00 – 17:00")
        );
        // A single value keeps its full text.
        assert_eq!(day(9, 14).show_short(&formats), None);
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
