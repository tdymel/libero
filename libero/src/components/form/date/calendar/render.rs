use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use chrono::{Datelike, NaiveDate};

use crate::{
    components::{
        buttons::ActionIcon,
        common::{Glyph, Input, Part, Variant},
        form::ChronoPickerPart,
    },
    context::IconSlot,
    hooks::{use_formats, use_localization},
    sx::ThemeAwareValue,
    theme::Size,
};

use super::styles::NAV_SX;
use super::view::{Strip, View};
use super::{
    DateLevel, DayRule, Focus, Selection, add_days, add_months, date_formatter, day_allowed,
    first_of_month, format_date,
};

impl View {
    pub(super) fn nav(
        self,
        label: &'static str,
        disabled: bool,
        target: NaiveDate,
        forward: bool,
    ) -> Element {
        let View {
            size,
            focusable,
            paged,
            mut nav_targets,
            ..
        } = self;
        nav_targets.write()[usize::from(forward)] = target;
        rsx! {
            Nav { label, disabled, targets: nav_targets, forward, size, focusable, paged }
        }
    }

    /// The mini variant: one row of `strip.days` days from its own first day.
    /// The buttons page it; an arrow key past an end slides it.
    pub(super) fn strip_view(self, strip: Strip) -> Element {
        let View {
            names,
            formats,
            selection,
            onpick,
            today,
            min,
            max,
            mut paged,
            mut active,
            ..
        } = self;
        let Strip {
            start,
            end,
            days,
            stop,
        } = strip;
        let cell = move |day: NaiveDate| {
            let (picked, _) = selection.marks(day, None);
            rsx! {
                div {
                    key: "{day}",
                    role: "gridcell",
                    "aria-selected": picked.to_string(),
                    button {
                        r#type: "button",
                        "data-slot": ChronoPickerPart::Day.slot(),
                        "data-date": "{day}",
                        "data-today": (today == Some(day)).then_some("true"),
                        "aria-current": (today == Some(day)).then_some("date"),
                        "data-selected": picked.then_some("true"),
                        "aria-label": format_date(day, (formats.date)(DateLevel::Day), names),
                        disabled: self.day_disabled(day).then_some(true),
                        tabindex: self.tabindex(day == stop),
                        onclick: move |_| {
                            // Pinned, so the new value does not move the row.
                            paged.set(Some(start));
                            active.set(Some(day));
                            onpick.call(day);
                        },
                        span { "data-slot": ChronoPickerPart::Month.slot(),{format_date(day, "MMM", names)} }
                        span { "{day.day()}" }
                    }
                }
            }
        };
        rsx! {
            div { "data-slot": ChronoPickerPart::Strip.slot(),
                {self.nav(names.previous_days, min.is_some_and(|min| add_days(start, -1) < min), add_days(start, -days), false)}
                // The slot a field's dropdown looks for to hand focus in.
                div { "data-slot": ChronoPickerPart::Months.slot(),
                    div {
                        role: "grid",
                        "aria-label": strip.name(formats, names),
                        onkeydown: move |event| strip.keydown(self, event),
                        div { role: "row",
                            for offset in 0..days {
                                {cell(add_days(start, offset))}
                            }
                        }
                    }
                }
                {self.nav(names.next_days, max.is_some_and(|max| add_days(end, 1) > max), add_days(start, days), true)}
            }
        }
    }

    pub(super) fn month_view(self, cell_stop: NaiveDate) -> Element {
        let View {
            names,
            formats,
            selection,
            onpick,
            lowest,
            mut level,
            mut paged,
            mut active,
            focus,
            first,
            today,
            min,
            max,
            ..
        } = self;
        let year = first.year();
        let cell = move |index: u32| {
            let Some(month) = NaiveDate::from_ymd_opt(year, index + 1, 1) else {
                return rsx! {};
            };
            let month_end = add_days(add_months(month, 1), -1);
            let disabled = (min.is_some_and(|min| month_end < min)
                || max.is_some_and(|max| month > max))
            .then_some(true);
            let picked = selection
                .picks()
                .into_iter()
                .flatten()
                .any(|day| first_of_month(day) == month);
            let is_today = today.is_some_and(|today| first_of_month(today) == month);
            // A `gridcell` around the button, as the day view: a button may not be a gridcell.
            rsx! {
                div {
                    role: "gridcell",
                    "aria-selected": if picked { "true" } else { "false" },
                    button {
                        r#type: "button",
                        "data-slot": ChronoPickerPart::Cell.slot(),
                        "data-date": "{month}",
                        "data-selected": picked.then_some("true"),
                        "data-today": is_today.then_some("true"),
                        "aria-current": is_today.then_some("date"),
                        "aria-label": format_date(month, (formats.date)(DateLevel::Month), names),
                        disabled,
                        tabindex: self.tabindex(month == cell_stop),
                        onclick: move |_| {
                            if lowest == DateLevel::Month {
                                onpick.call(month);
                            } else {
                                paged.set(Some(month));
                                level.set(DateLevel::Day);
                                active.set(Some(month));
                                // The 1st may be disabled; the tab stop is not.
                                focus.to(Focus::Stop);
                            }
                        },
                        {names.months_short[index as usize]}
                    }
                }
            }
        };
        rsx! {
            div { "data-slot": ChronoPickerPart::Header.slot(),
                {self.nav(names.previous_year, min.is_some_and(|min| year <= min.year()), add_months(first, -12), false)}
                button {
                    r#type: "button",
                    "data-slot": ChronoPickerPart::Title.slot(),
                    "aria-live": "polite",
                    tabindex: self.tabindex(true),
                    onclick: move |_| {
                        level.set(DateLevel::Year);
                        // The decade's title is plain text, so focus goes to the year.
                        focus.to(Focus::Stop);
                    },
                    "{year}"
                }
                {self.nav(names.next_year, max.is_some_and(|max| year >= max.year()), add_months(first, 12), true)}
            }
            div {
                "data-slot": ChronoPickerPart::Cells.slot(),
                role: "grid",
                "aria-label": "{year}",
                onkeydown: move |event| self.cell_keydown(event, cell_stop),
                // Rows written out, not looped: one template, cells diffed in place.
                div { role: "row", {cell(0)} {cell(1)} {cell(2)} }
                div { role: "row", {cell(3)} {cell(4)} {cell(5)} }
                div { role: "row", {cell(6)} {cell(7)} {cell(8)} }
                div { role: "row", {cell(9)} {cell(10)} {cell(11)} }
            }
        }
    }

    pub(super) fn year_view(self, cell_stop: NaiveDate) -> Element {
        let View {
            names,
            selection,
            onpick,
            lowest,
            mut level,
            mut paged,
            mut active,
            focus,
            first,
            decade,
            today,
            min,
            max,
            ..
        } = self;
        let cell = move |offset: i32| {
            let Some(start) = NaiveDate::from_ymd_opt(decade + offset, first.month(), 1) else {
                return rsx! {};
            };
            let shown_year = start.year();
            let year_start = NaiveDate::from_ymd_opt(shown_year, 1, 1).expect("a real day");
            let disabled = (min.is_some_and(|min| shown_year < min.year())
                || max.is_some_and(|max| shown_year > max.year()))
            .then_some(true);
            let picked = selection
                .picks()
                .into_iter()
                .flatten()
                .any(|day| day.year() == shown_year);
            let outside = !(0..10).contains(&offset);
            let is_today = today.is_some_and(|today| today.year() == shown_year);
            rsx! {
                div {
                    role: "gridcell",
                    "aria-selected": if picked { "true" } else { "false" },
                    button {
                        r#type: "button",
                        "data-slot": ChronoPickerPart::Cell.slot(),
                        "data-date": "{year_start}",
                        "data-outside": outside.then_some("true"),
                        "data-selected": picked.then_some("true"),
                        "data-today": is_today.then_some("true"),
                        "aria-current": is_today.then_some("date"),
                        disabled,
                        tabindex: self.tabindex(year_start == cell_stop),
                        onclick: move |_| {
                            if lowest == DateLevel::Year {
                                onpick.call(year_start);
                            } else {
                                paged.set(Some(start));
                                level.set(DateLevel::Month);
                                active.set(Some(start));
                                focus.to(Focus::Stop);
                            }
                        },
                        "{shown_year}"
                    }
                }
            }
        };
        rsx! {
            div { "data-slot": ChronoPickerPart::Header.slot(),
                {self.nav(names.previous_decade, min.is_some_and(|min| decade <= min.year()), add_months(first, -120), false)}
                // The top level: nothing to climb to, so plain text, not a dimmed button.
                div {
                    "data-slot": ChronoPickerPart::Title.slot(),
                    "aria-live": "polite",
                    "{decade} – {decade + 9}"
                }
                {self.nav(names.next_decade, max.is_some_and(|max| decade + 9 >= max.year()), add_months(first, 120), true)}
            }
            div {
                "data-slot": ChronoPickerPart::Cells.slot(),
                role: "grid",
                "aria-label": "{decade} – {decade + 9}",
                onkeydown: move |event| self.cell_keydown(event, cell_stop),
                div { role: "row", {cell(-1)} {cell(0)} {cell(1)} }
                div { role: "row", {cell(2)} {cell(3)} {cell(4)} }
                div { role: "row", {cell(5)} {cell(6)} {cell(7)} }
                div { role: "row", {cell(8)} {cell(9)} {cell(10)} }
            }
        }
    }
}

/// One row of a month grid, own scope with only its seven days' marks, so a pick
/// elsewhere leaves its props equal and skips the re-render.
#[derive(Props, Clone, PartialEq)]
pub(super) struct WeekProps {
    first: NaiveDate,
    /// The month the grid shows. The other days are its neighbours'.
    month: NaiveDate,
    /// Side by side, a neighbour's days would appear twice: draw blanks.
    blanks: bool,
    /// Clipped to the week.
    selection: Selection,
    /// `selection` without the preview: what `aria-selected` says.
    value: Selection,
    today: Option<NaiveDate>,
    tab_stop: Option<NaiveDate>,
    min: Option<NaiveDate>,
    max: Option<NaiveDate>,
    #[props(into)]
    exclude_date: DayRule,
    focusable: bool,
    awaits_end: bool,
    hover: Signal<Option<NaiveDate>>,
    active: Signal<Option<NaiveDate>>,
    paged: Signal<Option<NaiveDate>>,
    onpick: EventHandler<NaiveDate>,
}

#[component]
pub(super) fn Week(props: WeekProps) -> Element {
    let WeekProps {
        first,
        month,
        blanks,
        selection,
        value,
        today,
        tab_stop,
        min,
        max,
        exclude_date,
        focusable,
        awaits_end,
        hover,
        active,
        paged,
        onpick,
    } = props;
    let allowed = day_allowed(min, max, exclude_date.0);
    let names = &use_localization().date;
    // APG: the full date, not only the number shown.
    let label = date_formatter((use_formats().date)(DateLevel::Day), names);
    let (mut hover, mut active, mut paged) = (hover, active, paged);

    let day = move |offset: i64| add_days(first, offset);
    let outside = move |day: NaiveDate| first_of_month(day) != month;
    // Outside days sit at the ends of a week, so the blanks do too.
    let (lead, trail) = match blanks {
        true => (
            (0..7).take_while(|offset| day(*offset) < month).count() as i64,
            (0..7)
                .rev()
                .take_while(|offset| first_of_month(day(*offset)) > month)
                .count() as i64,
        ),
        false => (0, 0),
    };
    let cell = move |offset: i64| {
        let day = day(offset);
        let (picked, in_range) = selection.marks(day, None);
        // Every day of a picked range is selected (APG grid); a preview is not.
        let (end, inside) = value.marks(day, None);
        let selected = end || inside;
        (offset, day, outside(day), picked, in_range, selected)
    };

    // Inline, not a helper returning `rsx!`: a nested element costs ~1 µs per item.
    // Keyed by column, so paging diffs the days in place.
    rsx! {
        div { role: "row",
            for offset in 0..lead {
                div { key: "{offset}", role: "gridcell", "data-slot": ChronoPickerPart::Blank.slot() }
            }
            for (offset, day, outside, picked, in_range, selected) in (lead..7 - trail).map(cell) {
                div {
                    key: "{offset}",
                    role: "gridcell",
                    "aria-selected": if selected { "true" } else { "false" },
                    button {
                        r#type: "button",
                        "data-slot": ChronoPickerPart::Day.slot(),
                        "data-date": "{day}",
                        "data-outside": outside.then_some("true"),
                        "data-today": (today == Some(day)).then_some("true"),
                        "aria-current": (today == Some(day)).then_some("date"),
                        "data-selected": picked.then_some("true"),
                        "data-in-range": in_range.then_some("true"),
                        "aria-label": label(day),
                        // dioxus-native writes `disabled="false"`, which Blitz reads as disabled.
                        disabled: (!allowed(day)).then_some(true),
                        tabindex: if focusable && !outside && tab_stop == Some(day) { "0" } else { "-1" },
                        onmouseenter: move |_| {
                            if awaits_end {
                                hover.set(Some(day));
                            }
                        },
                        // The keyboard previews the range too.
                        onfocus: move |_| {
                            if awaits_end {
                                hover.set(Some(day));
                            }
                        },
                        onclick: move |_| {
                            active.set(Some(day));
                            // A neighbour's day pages to its month.
                            if outside {
                                paged.set(Some(first_of_month(day)));
                            }
                            onpick.call(day);
                        },
                        "{day.day()}"
                    }
                }
            }
            for offset in 7 - trail..7 {
                div { key: "{offset}", role: "gridcell", "data-slot": ChronoPickerPart::Blank.slot() }
            }
        }
    }
}

/// The weekday names over a month, own scope so it skips every re-render.
#[component]
pub(super) fn Weekdays(first_weekday: usize) -> Element {
    let names = &use_localization().date;
    rsx! {
        div { role: "row",
            for index in 0..7 {
                div {
                    key: "{index}",
                    role: "columnheader",
                    "data-slot": ChronoPickerPart::Weekday.slot(),
                    "aria-label": names.weekdays[(first_weekday + index) % 7],
                    {names.weekdays_min[(first_weekday + index) % 7]}
                }
            }
        }
    }
}

/// A button that pages the calendar. Reads its target on click, so paging leaves its props equal.
#[derive(Props, Clone, PartialEq)]
pub(super) struct NavProps {
    label: &'static str,
    disabled: bool,
    targets: CopyValue<[NaiveDate; 2]>,
    forward: bool,
    size: Size,
    focusable: bool,
    paged: Signal<Option<NaiveDate>>,
}

#[component]
pub(super) fn Nav(props: NavProps) -> Element {
    let (targets, forward, mut paged) = (props.targets, props.forward, props.paged);
    rsx! {
        ActionIcon {
            "data-slot": ChronoPickerPart::Nav.slot(),
            aria_label: props.label,
            sx: &NAV_SX,
            variant: Input::Value(Variant::Standard),
            size: ThemeAwareValue::Size(nav_size(props.size)),
            tabindex: if props.focusable { "0" } else { "-1" },
            disabled: props.disabled,
            // Paging to `min` or `max` disables the focused button: focus stays on it.
            focusable_when_disabled: true,
            onclick: move |_| paged.set(Some(targets.peek()[usize::from(forward)])),
            if props.forward {
                Glyph { slot: IconSlot::ChevronRight, icon: lucide::chevron_right::outlined }
            } else {
                Glyph { slot: IconSlot::ChevronLeft, icon: lucide::chevron_left::outlined }
            }
        }
    }
}

/// The nav button size for a picker size: `ActionIcon`'s scale climbs faster, so two share one.
const fn nav_size(size: Size) -> Size {
    match size {
        Size::Xs | Size::Sm => Size::Xs,
        Size::Md | Size::Lg => Size::Sm,
        Size::Xl | Size::Xxl => Size::Md,
    }
}
