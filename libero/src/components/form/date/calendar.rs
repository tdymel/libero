use dioxus::prelude::*;

use chrono::{Datelike, Days, Months, NaiveDate};

use super::{DateRange, format::format_date, today::use_today};
use crate::{
    components::{
        ActionIcon, ButtonVariant, ClassList, HtmlTag, Input, States, common::focus_ring_sx,
        layout::use_box,
    },
    hooks::{use_element, use_theme},
    platform::ElementApi,
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CalendarVariant, DATE_PICKER_DAY, DATE_PICKER_FONT_SIZE, DatePickerDefaults, Size, SizeCss,
    },
};

crate::components::common::input_from_str!(CalendarVariant);

/// Where a calendar opens with no value and no clock.
const FALLBACK_MONTH: NaiveDate = match NaiveDate::from_ymd_opt(1970, 1, 1) {
    Some(date) => date,
    None => unreachable!(),
};

pub(super) fn first_of_month(day: NaiveDate) -> NaiveDate {
    day.with_day(1).expect("every month has a first day")
}

/// Stops at `chrono`'s first and last day rather than failing.
fn add_days(day: NaiveDate, days: i64) -> NaiveDate {
    let step = Days::new(days.unsigned_abs());
    match days < 0 {
        true => day.checked_sub_days(step),
        false => day.checked_add_days(step),
    }
    .unwrap_or(day)
}

/// Keeps the day where the month has it and takes the month's last day where
/// it does not: January 31 plus a month is February 28, or 29.
fn add_months(day: NaiveDate, months: i64) -> NaiveDate {
    let step = Months::new(u32::try_from(months.unsigned_abs()).unwrap_or(u32::MAX));
    match months < 0 {
        true => day.checked_sub_months(step),
        false => day.checked_add_months(step),
    }
    .unwrap_or(day)
}

static CALENDAR_SX: StaticSx = StaticSx::new(|| {
    let day = DATE_PICKER_DAY.value();
    let button = sx()
        .display("flex")
        .align_items("center")
        .justify_content("center")
        .padding("0")
        .margin("0")
        .border_style("solid")
        .border_width("1px")
        .border_color("transparent")
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        .background("transparent")
        .color("inherit")
        .font_family("inherit")
        .font_size("inherit")
        .cursor("pointer")
        .hover(sx().background("grey.1"));
    DatePickerDefaults::theme_vars()
        .display("inline-flex")
        .flex_direction("column")
        .gap("4px")
        .font_size(DATE_PICKER_FONT_SIZE.value())
        .selector(
            "& > [data-slot='header']",
            sx().display("flex").align_items("center").gap("4px"),
        )
        .selector(
            "& [data-slot='title']",
            button
                .clone()
                .flex("1 1 0")
                .height(day.clone())
                .font_weight("600"),
        )
        .selector(
            "& [data-slot='title']:disabled",
            sx().cursor("default").hover(sx().background("transparent")),
        )
        .selector(
            "& > [data-slot='months']",
            sx().display("flex").gap("16px").align_items("flex-start"),
        )
        .selector(
            "& [role='grid']",
            sx().display("grid")
                .grid_template_columns(format!("repeat(7, {day})")),
        )
        // Rows exist for assistive technology; the grid lays out the cells.
        .selector("& [role='row']", sx().display("contents"))
        .selector(
            "& [role='columnheader']",
            sx().height(day.clone())
                .display("flex")
                .align_items("center")
                .justify_content("center")
                .font_size("0.85em")
                .font_weight("500"),
        )
        .selector("& [role='gridcell']", sx().display("flex"))
        .selector(
            "& [data-slot='blank']",
            sx().width(day.clone()).height(day.clone()),
        )
        .selector(
            "& [data-slot='cells']",
            sx().display("grid")
                .grid_template_columns("repeat(3, 1fr)")
                .gap("4px")
                .width(format!("calc(7 * {day})")),
        )
        .selector(
            "& [data-slot='day']",
            button.clone().width(day.clone()).height(day.clone()),
        )
        // The mini variant: one row of taller days, a month label over the
        // number.
        .selector(
            "& [data-slot='strip']",
            sx().display("flex").align_items("center").gap("4px"),
        )
        .selector(
            "& [data-slot='strip'] [role='grid']",
            sx().display("flex").gap("2px"),
        )
        .selector(
            "& [data-slot='strip'] [data-slot='day']",
            sx().flex_direction("column")
                .gap("2px")
                .width(format!("calc(1.4 * {day})"))
                .height(format!("calc(1.6 * {day})")),
        )
        .selector(
            "& [data-slot='strip'] [data-slot='month']",
            sx().font_size("0.75em").opacity("0.7"),
        )
        .selector(
            "& [data-slot='cell']",
            button.width("100%").height(format!("calc(1.25 * {day})")),
        )
        .selector("& [data-outside]", sx().color("grey.6"))
        .selector("& [data-today]", sx().border_color("primary.6"))
        .selector(
            "& [data-in-range]",
            sx().background("primary.1")
                .color("primary-contrast.1")
                .border_radius("0")
                .hover(sx().background("primary.2")),
        )
        // After the plain hover and the range tint, so a picked day keeps its
        // fill under the mouse: equal specificity, source order decides.
        .selector(
            "& [data-selected]",
            sx().background("primary.6")
                .color("primary-contrast.6")
                .hover(sx().background("primary.7")),
        )
        .selector(
            "& :is([data-slot='day'], [data-slot='cell']):disabled",
            sx().opacity("0.4")
                .cursor("not-allowed")
                .hover(sx().background("transparent")),
        )
        .selector("& button:focus-visible", focus_ring_sx())
        // A picked cell's fill sets the ring's contrast colour, which is
        // meant for its inside: drawn around it, a light ring vanishes on the
        // page. After the ring above, and more specific, so it wins.
        .selector(
            "& [data-selected]:focus-visible",
            sx().outline_offset("-4px"),
        )
});

/// What a calendar marks as picked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Selection {
    Single(Option<NaiveDate>),
    Range(Option<DateRange<NaiveDate>>),
}

impl Selection {
    fn anchor(self) -> Option<NaiveDate> {
        match self {
            Self::Single(day) => day,
            Self::Range(range) => range.map(|range| range.start),
        }
    }

    /// The picked days, at most two.
    fn picks(self) -> [Option<NaiveDate>; 2] {
        match self {
            Self::Single(day) => [day, None],
            Self::Range(range) => [
                range.map(|range| range.start),
                range.and_then(|range| range.end),
            ],
        }
    }

    fn awaits_end(self) -> bool {
        matches!(self, Self::Range(Some(range)) if range.end.is_none())
    }

    /// Whether `day` is picked, and whether it lies strictly inside the range.
    /// `hover` stands in for an end not picked yet.
    fn marks(self, day: NaiveDate, hover: Option<NaiveDate>) -> (bool, bool) {
        match self {
            Self::Single(picked) => (picked == Some(day), false),
            Self::Range(None) => (false, false),
            Self::Range(Some(range)) => {
                let Some(end) = range.end.or(hover) else {
                    return (day == range.start, false);
                };
                let (from, to) = match end < range.start {
                    true => (end, range.start),
                    false => (range.start, end),
                };
                (day == from || day == to, day > from && day < to)
            }
        }
    }
}

/// The view a calendar shows: days of a month, months of a year, years of a
/// decade. `DatePicker`'s `level` picks the lowest one - the one a pick lands
/// on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DateLevel {
    Day,
    Month,
    Year,
}

/// Where focus goes after the next render.
#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Date(NaiveDate),
    Title,
}

/// `exclude_date` as a calendar holds it. Two `Callback`s built on different
/// renders can compare equal, so a changed rule would not redraw a calendar
/// whose other props did not change. A rule never compares equal: only a
/// calendar without one skips a re-render from above.
#[derive(Clone, Copy, Default)]
pub(super) struct DayRule(Option<Callback<NaiveDate, bool>>);

impl PartialEq for DayRule {
    fn eq(&self, other: &Self) -> bool {
        self.0.is_none() && other.0.is_none()
    }
}

impl From<Option<Callback<NaiveDate, bool>>> for DayRule {
    fn from(rule: Option<Callback<NaiveDate, bool>>) -> Self {
        Self(rule)
    }
}

#[derive(Props, Clone, PartialEq)]
pub(super) struct CalendarProps {
    selection: Selection,
    /// A day at `lowest`, or the first day of a month or a year above it.
    onpick: EventHandler<NaiveDate>,
    /// Months side by side at the day level.
    #[props(default = 1)]
    columns: usize,
    /// The level a pick lands on; the levels above only navigate.
    #[props(default = DateLevel::Day)]
    lowest: DateLevel,
    /// A month grid, or one row of `days` days. Only at the day level.
    #[props(default)]
    variant: CalendarVariant,
    #[props(default = 7)]
    days: usize,
    #[props(default)]
    min: Option<NaiveDate>,
    #[props(default)]
    max: Option<NaiveDate>,
    #[props(default, into)]
    exclude_date: DayRule,
    #[props(default)]
    today: Option<NaiveDate>,
    size: Size,
    #[props(default = true)]
    focusable: bool,
    /// A hidden input's name and value, so the calendar posts with a form.
    #[props(default)]
    hidden: Option<(String, String)>,
    #[props(default)]
    class: Input<ClassList>,
    #[props(default)]
    sx: Input<Sx>,
    #[props(default)]
    states: Input<States>,
    #[props(default)]
    attributes: Vec<Attribute>,
}

/// The engine every date picker draws: a header, then a day, month or year
/// view. Non-generic - what is picked arrives as a [`Selection`] of plain
/// values, so the props compare properly.
#[component]
pub(super) fn Calendar(props: CalendarProps) -> Element {
    let theme = use_theme();
    let names = &theme.date;
    let selection = props.selection;
    let onpick = props.onpick;
    let columns = props.columns.max(1) as i64;
    let lowest = props.lowest;
    let (min, max, exclude_date) = (props.min, props.max, props.exclude_date.0);
    let focusable = props.focusable;
    let tabindex = move |stop: bool| if focusable && stop { "0" } else { "-1" };

    let today = use_today(props.today);
    // A new `lowest` remounts the calendar - `DateValue::picker` keys it - so
    // the view only has to start there.
    let mut level = use_signal(|| lowest);
    // Only ever set by paging; until then the view follows the value.
    let mut paged = use_signal(|| None::<NaiveDate>);
    // The keyboard's day, and where focus goes after the next render.
    let mut active = use_signal(|| None::<NaiveDate>);
    let focus_request = use_signal(|| None::<Focus>);
    let root = use_element();
    // Only moves focus that is already in the calendar: a click inside a
    // field's dropdown leaves it on the text input.
    let focus_to = move |target: Focus| {
        if focusable && root.query_selector(":focus").is_ok() {
            let mut request = focus_request;
            request.set(Some(target));
        }
    };
    // The day under the mouse while a range waits for its end.
    let mut hover = use_signal(|| None::<NaiveDate>);

    let first = first_of_month(
        paged()
            .or(selection.anchor())
            .or(today)
            .unwrap_or(FALLBACK_MONTH),
    );
    let last = add_months(first, columns - 1);
    let shown = move |day: NaiveDate| (first..=last).contains(&first_of_month(day));
    let tab_stop = active()
        .filter(|day| shown(*day))
        .or(selection.anchor().filter(|day| shown(*day)))
        .or(today.filter(|day| shown(*day)))
        .unwrap_or(first);

    // Runs after the render that drew the target, so it exists. A level change
    // replaces the heading and the cells, which would leave focus on `body`.
    use_effect(move || {
        let Some(target) = focus_request() else {
            return;
        };
        let mut request = focus_request;
        request.set(None);
        let selector = match target {
            Focus::Title => "[data-slot='title']".to_string(),
            Focus::Date(day) => format!("[data-date='{day}']:not([data-outside])"),
        };
        let _ = root
            .query_selector(&selector)
            .and_then(|element| element.focus());
    });

    let first_weekday = names.first_weekday.num_days_from_monday() as usize;
    let column = move |day: NaiveDate| {
        ((day.weekday().num_days_from_monday() as usize + 7 - first_weekday) % 7) as i64
    };
    let day_disabled = move |day: NaiveDate| {
        min.is_some_and(|min| day < min)
            || max.is_some_and(|max| day > max)
            || exclude_date.is_some_and(|exclude| exclude.call(day))
    };

    let onkeydown = move |event: KeyboardEvent| {
        let years = if event.modifiers().shift() { 12 } else { 1 };
        let next = match event.key() {
            Key::ArrowLeft => add_days(tab_stop, -1),
            Key::ArrowRight => add_days(tab_stop, 1),
            Key::ArrowUp => add_days(tab_stop, -7),
            Key::ArrowDown => add_days(tab_stop, 7),
            Key::Home => add_days(tab_stop, -column(tab_stop)),
            Key::End => add_days(tab_stop, 6 - column(tab_stop)),
            Key::PageUp => add_months(tab_stop, -years),
            Key::PageDown => add_months(tab_stop, years),
            _ => return,
        };
        event.prevent_default();
        let month = first_of_month(next);
        if month < first {
            paged.set(Some(month));
        } else if month > last {
            paged.set(Some(add_months(month, 1 - columns)));
        }
        active.set(Some(next));
        focus_to(Focus::Date(next));
    };

    // The month and year views: a grid three wide, one cell a month or a
    // year, with one tab stop. A step off the shown year or decade pages it.
    let decade = first.year() - first.year().rem_euclid(10);
    let cell_of = move |day: NaiveDate| match level() {
        DateLevel::Year => NaiveDate::from_ymd_opt(day.year(), 1, 1).unwrap_or(day),
        _ => first_of_month(day),
    };
    let in_view = move |cell: NaiveDate| match level() {
        DateLevel::Year => (decade..decade + 10).contains(&cell.year()),
        _ => cell.year() == first.year(),
    };
    let cell_stop = [active(), selection.anchor(), today]
        .into_iter()
        .flatten()
        .map(cell_of)
        .find(|cell| in_view(*cell))
        .unwrap_or_else(|| {
            let year = if level() == DateLevel::Year {
                decade
            } else {
                first.year()
            };
            NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(first)
        });
    let cell_keydown = move |event: KeyboardEvent| {
        let (months_per_cell, cells_per_page, column) = match level() {
            DateLevel::Day => return,
            DateLevel::Month => (1, 12, i64::from(cell_stop.month0() % 3)),
            DateLevel::Year => (
                12,
                10,
                i64::from((cell_stop.year() - decade + 1).rem_euclid(3)),
            ),
        };
        let cells = match event.key() {
            Key::ArrowLeft => -1,
            Key::ArrowRight => 1,
            Key::ArrowUp => -3,
            Key::ArrowDown => 3,
            Key::Home => -column,
            Key::End => 2 - column,
            Key::PageUp => -cells_per_page,
            Key::PageDown => cells_per_page,
            _ => return,
        };
        event.prevent_default();
        let next = add_months(cell_stop, cells * months_per_cell);
        paged.set(Some(next));
        active.set(Some(next));
        focus_to(Focus::Date(next));
    };

    let awaits_end = selection.awaits_end();
    // Read only while a range waits, so a plain picker never re-renders on
    // mouse movement.
    let hovered = if awaits_end { hover() } else { None };

    let day_cell = move |day: NaiveDate, month: NaiveDate| {
        let outside = first_of_month(day) != month;
        // Side by side, a neighbour's days would appear twice.
        if outside && columns > 1 {
            return rsx! {
                div { key: "{day}", role: "gridcell", "data-slot": "blank" }
            };
        }
        let (picked, in_range) = selection.marks(day, hovered);
        rsx! {
            div {
                key: "{day}",
                role: "gridcell",
                "aria-selected": picked.to_string(),
                button {
                    r#type: "button",
                    "data-slot": "day",
                    "data-date": "{day}",
                    "data-outside": outside.then_some("true"),
                    "data-today": (today == Some(day)).then_some("true"),
                    "data-selected": picked.then_some("true"),
                    "data-in-range": in_range.then_some("true"),
                    disabled: day_disabled(day),
                    tabindex: tabindex(!outside && day == tab_stop),
                    onmouseenter: move |_| {
                        if awaits_end {
                            hover.set(Some(day));
                        }
                    },
                    onclick: move |_| {
                        active.set(Some(day));
                        if !shown(day) {
                            paged.set(Some(first_of_month(day)));
                        }
                        onpick.call(day);
                    },
                    "{day.day()}"
                }
            }
        }
    };

    let month_grid = move |month: NaiveDate| {
        let start = add_days(month, -column(month));
        let title = format_date(month, names.month_format, names);
        rsx! {
            div {
                key: "{month}",
                role: "grid",
                "aria-label": "{title}",
                onkeydown,
                div { role: "row",
                    for index in 0..7 {
                        div {
                            key: "{index}",
                            role: "columnheader",
                            "aria-label": names.weekdays[(first_weekday + index) % 7],
                            {names.weekdays_min[(first_weekday + index) % 7]}
                        }
                    }
                }
                for week in 0..6 {
                    div { key: "{week}", role: "row",
                        for offset in 0..7 {
                            {day_cell(add_days(start, week * 7 + offset), month)}
                        }
                    }
                }
            }
        }
    };

    let icon_size = ThemeAwareValue::Size(nav_size(props.size));
    let nav = move |label: &'static str, disabled: bool, target: NaiveDate, glyph: Element| {
        rsx! {
            ActionIcon {
                aria_label: label,
                variant: Input::Value(ButtonVariant::Standard),
                size: icon_size.clone(),
                tabindex: tabindex(true),
                disabled,
                onclick: move |_| paged.set(Some(target)),
                {glyph}
            }
        }
    };
    let back = rsx! { ChevronLeftIcon {} };
    let forward = rsx! { ChevronRightIcon {} };

    // The mini variant: one row of `days` days from its own first day. The
    // buttons page it; an arrow key past an end slides it.
    let mini = props.variant == CalendarVariant::Mini;
    let strip_days = props.days.max(1) as i64;
    let strip_start = paged()
        .or(selection.anchor())
        .or(today)
        .unwrap_or(FALLBACK_MONTH);
    let strip_end = add_days(strip_start, strip_days - 1);
    let in_strip = move |day: NaiveDate| (strip_start..=strip_end).contains(&day);
    let strip_stop = [active(), selection.anchor(), today]
        .into_iter()
        .flatten()
        .find(|day| in_strip(*day))
        .unwrap_or(strip_start);
    let strip_keydown = move |event: KeyboardEvent| {
        let next = match event.key() {
            Key::ArrowLeft => add_days(strip_stop, -1),
            Key::ArrowRight => add_days(strip_stop, 1),
            Key::Home => strip_start,
            Key::End => strip_end,
            Key::PageUp => add_days(strip_stop, -strip_days),
            Key::PageDown => add_days(strip_stop, strip_days),
            _ => return,
        };
        event.prevent_default();
        // A step off an end moves the row as far: an arrow a day, a page a
        // page.
        let start = match in_strip(next) {
            true => strip_start,
            false => add_days(strip_start, (next - strip_stop).num_days()),
        };
        paged.set(Some(start));
        active.set(Some(next));
        focus_to(Focus::Date(next));
    };
    let strip_cell = move |day: NaiveDate| {
        let (picked, _) = selection.marks(day, None);
        rsx! {
            div {
                key: "{day}",
                role: "gridcell",
                "aria-selected": picked.to_string(),
                button {
                    r#type: "button",
                    "data-slot": "day",
                    "data-date": "{day}",
                    "data-today": (today == Some(day)).then_some("true"),
                    "data-selected": picked.then_some("true"),
                    "aria-label": format_date(day, names.format, names),
                    disabled: day_disabled(day),
                    tabindex: tabindex(day == strip_stop),
                    onclick: move |_| {
                        // Pinned, so the new value does not move the row.
                        paged.set(Some(strip_start));
                        active.set(Some(day));
                        onpick.call(day);
                    },
                    span { "data-slot": "month", {format_date(day, "MMM", names)} }
                    span { "{day.day()}" }
                }
            }
        }
    };

    let body = match level() {
        _ if mini => rsx! {
            div { "data-slot": "strip",
                {nav(names.previous_days, min.is_some_and(|min| add_days(strip_start, -1) < min), add_days(strip_start, -strip_days), rsx! { ChevronLeftIcon {} })}
                // The slot a field's dropdown looks for to hand focus in.
                div { "data-slot": "months",
                    div {
                        role: "grid",
                        "aria-label": format_date(strip_start, names.month_format, names),
                        onkeydown: strip_keydown,
                        div { role: "row",
                            for offset in 0..strip_days {
                                {strip_cell(add_days(strip_start, offset))}
                            }
                        }
                    }
                }
                {nav(names.next_days, max.is_some_and(|max| add_days(strip_end, 1) > max), add_days(strip_start, strip_days), rsx! { ChevronRightIcon {} })}
            }
        },
        DateLevel::Day => {
            let months: Vec<NaiveDate> =
                (0..columns).map(|index| add_months(first, index)).collect();
            rsx! {
                div { "data-slot": "header",
                    {nav(names.previous_month, min.is_some_and(|min| add_days(first, -1) < min), add_months(first, -1), back)}
                    for month in months.iter().copied() {
                        button {
                            key: "{month}",
                            r#type: "button",
                            "data-slot": "title",
                            "aria-live": "polite",
                            tabindex: tabindex(true),
                            onclick: move |_| {
                                paged.set(Some(month));
                                level.set(DateLevel::Month);
                                focus_to(Focus::Title);
                            },
                            {format_date(month, names.month_format, names)}
                        }
                    }
                    {nav(names.next_month, max.is_some_and(|max| add_months(last, 1) > max), add_months(first, 1), forward)}
                }
                div {
                    "data-slot": "months",
                    onmouseleave: move |_| {
                        if awaits_end {
                            hover.set(None);
                        }
                    },
                    for month in months {
                        {month_grid(month)}
                    }
                }
            }
        }
        DateLevel::Month => {
            let year = first.year();
            let cell = move |index: u32| {
                let Some(month) = NaiveDate::from_ymd_opt(year, index + 1, 1) else {
                    return rsx! {};
                };
                let month_end = add_days(add_months(month, 1), -1);
                let disabled =
                    min.is_some_and(|min| month_end < min) || max.is_some_and(|max| month > max);
                let picked = selection
                    .picks()
                    .into_iter()
                    .flatten()
                    .any(|day| first_of_month(day) == month);
                rsx! {
                    button {
                        key: "{month}",
                        r#type: "button",
                        "data-slot": "cell",
                        "data-date": "{month}",
                        "data-selected": picked.then_some("true"),
                        "data-today": today.is_some_and(|today| first_of_month(today) == month).then_some("true"),
                        disabled,
                        tabindex: tabindex(month == cell_stop),
                        onclick: move |_| {
                            if lowest == DateLevel::Month {
                                onpick.call(month);
                            } else {
                                paged.set(Some(month));
                                level.set(DateLevel::Day);
                                active.set(Some(month));
                                focus_to(Focus::Date(month));
                            }
                        },
                        {names.months_short[index as usize]}
                    }
                }
            };
            rsx! {
                div { "data-slot": "header",
                    {nav(names.previous_year, min.is_some_and(|min| year <= min.year()), add_months(first, -12), back)}
                    button {
                        r#type: "button",
                        "data-slot": "title",
                        "aria-live": "polite",
                        tabindex: tabindex(true),
                        onclick: move |_| {
                            level.set(DateLevel::Year);
                            focus_to(Focus::Title);
                        },
                        "{year}"
                    }
                    {nav(names.next_year, max.is_some_and(|max| year >= max.year()), add_months(first, 12), forward)}
                }
                div { "data-slot": "cells", onkeydown: cell_keydown,
                    for index in 0..12 {
                        {cell(index)}
                    }
                }
            }
        }
        DateLevel::Year => {
            let cell = move |offset: i32| {
                let Some(start) = NaiveDate::from_ymd_opt(decade + offset, first.month(), 1) else {
                    return rsx! {};
                };
                let shown_year = start.year();
                let year_start = NaiveDate::from_ymd_opt(shown_year, 1, 1).expect("a real day");
                let disabled = min.is_some_and(|min| shown_year < min.year())
                    || max.is_some_and(|max| shown_year > max.year());
                let picked = selection
                    .picks()
                    .into_iter()
                    .flatten()
                    .any(|day| day.year() == shown_year);
                let outside = !(0..10).contains(&offset);
                rsx! {
                    button {
                        key: "{shown_year}",
                        r#type: "button",
                        "data-slot": "cell",
                        "data-date": "{year_start}",
                        "data-outside": outside.then_some("true"),
                        "data-selected": picked.then_some("true"),
                        "data-today": today.is_some_and(|today| today.year() == shown_year).then_some("true"),
                        disabled,
                        tabindex: tabindex(year_start == cell_stop),
                        onclick: move |_| {
                            if lowest == DateLevel::Year {
                                onpick.call(year_start);
                            } else {
                                paged.set(Some(start));
                                level.set(DateLevel::Month);
                                active.set(Some(start));
                                focus_to(Focus::Date(start));
                            }
                        },
                        "{shown_year}"
                    }
                }
            };
            rsx! {
                div { "data-slot": "header",
                    {nav(names.previous_decade, min.is_some_and(|min| decade <= min.year()), add_months(first, -120), back)}
                    button {
                        r#type: "button",
                        "data-slot": "title",
                        "aria-live": "polite",
                        disabled: true,
                        "{decade} – {decade + 9}"
                    }
                    {nav(names.next_decade, max.is_some_and(|max| decade + 9 >= max.year()), add_months(first, 120), forward)}
                }
                div { "data-slot": "cells", onkeydown: cell_keydown,
                    for offset in -1..11 {
                        {cell(offset)}
                    }
                }
            }
        }
    };

    let states: Input<States> = props
        .states
        .unwrap_or_default()
        .with(props.size.state_name(), true)
        .into();
    let root_box = use_box()
        .framework_sx(&CALENDAR_SX)
        .class(&props.class)
        .sx(&props.sx)
        .states(&states)
        .prepare();
    let hidden = props.hidden.map(|(name, value)| {
        rsx! {
            input { r#type: "hidden", name, value }
        }
    });

    root_box.element(&root).render(
        HtmlTag::Div,
        props.attributes,
        rsx! {
            {body}
            {hidden}
        },
    )
}

/// The navigation buttons' icon step for a picker step. `ActionIcon`'s scale
/// climbs faster than a day cell's, so two picker steps share one icon step.
const fn nav_size(size: Size) -> Size {
    match size {
        Size::Xs | Size::Sm => Size::Xs,
        Size::Md | Size::Lg => Size::Sm,
        Size::Xl | Size::Xxl => Size::Md,
    }
}

#[component]
fn ChevronLeftIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M15 6l-6 6 6 6" }
        }
    }
}

#[component]
fn ChevronRightIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M9 6l6 6-6 6" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 9, day).expect("a real day")
    }

    #[test]
    fn a_waiting_range_previews_up_to_the_hovered_day() {
        let waiting = Selection::Range(Some(DateRange::new(day(10), None)));
        assert_eq!(waiting.marks(day(12), Some(day(14))), (false, true));
        assert_eq!(waiting.marks(day(14), Some(day(14))), (true, false));
        // Backwards works too.
        assert_eq!(waiting.marks(day(8), Some(day(6))), (false, true));
        assert_eq!(waiting.marks(day(12), None), (false, false));
        assert!(waiting.awaits_end());
    }

    #[test]
    fn a_complete_range_ignores_the_mouse() {
        let complete = Selection::Range(Some(DateRange::new(day(10), Some(day(12)))));
        assert_eq!(complete.marks(day(11), Some(day(20))), (false, true));
        assert_eq!(complete.marks(day(15), Some(day(20))), (false, false));
        assert!(!complete.awaits_end());
    }
}
