mod render;
mod styles;
#[cfg(test)]
mod tests;
mod view;

use dioxus::prelude::*;

use chrono::{Datelike, Days, Months, NaiveDate};

use super::{
    DateRange,
    fields::day_allowed,
    format::{date_formatter, format_date},
    today::use_today,
};
use crate::{
    components::{
        common::{ClassList, HtmlTag, Input, States},
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_formats, use_localization},
    platform::ElementApi,
    sx::Sx,
    theme::{CalendarVariant, Size},
};

use self::{
    render::{Nav, Week, Weekdays},
    styles::CALENDAR_SX,
    view::{FocusRequest, Strip, View},
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

    /// The same marks on `first..=last` and none past them, with `hover`
    /// folded in. A mark moving outside those days leaves the result equal.
    fn clip(self, hover: Option<NaiveDate>, first: NaiveDate, last: NaiveDate) -> Self {
        let inside = |day: NaiveDate| (first..=last).contains(&day);
        match self {
            Self::Single(day) => Self::Single(day.filter(|day| inside(*day))),
            Self::Range(None) => self,
            Self::Range(Some(range)) => {
                let Some(end) = range.end.or(hover) else {
                    return Self::Range(inside(range.start).then_some(range));
                };
                let (from, to) = match end < range.start {
                    true => (end, range.start),
                    false => (range.start, end),
                };
                if to < first || from > last {
                    return Self::Range(None);
                }
                // An end past the days is pulled in to just past them: still
                // outside, so nothing inside changes.
                let from = from.max(add_days(first, -1));
                let to = to.min(add_days(last, 1));
                Self::Range(Some(DateRange::new(from, Some(to))))
            }
        }
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

// Lives in the theme: `DateLocale::format` takes it, and lower layers may not import `components`.
pub use crate::theme::DateLevel;

/// Where focus goes after the next render.
#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Date(NaiveDate),
    Title,
    /// The day or cell that holds the tab stop, wherever it moved to.
    Stop,
}

/// `exclude_date`, never equal to another rule: `Callback`s from different renders can compare
/// equal, and a changed rule would not redraw.
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

/// A selector to focus inside `root` after the next render. A level change replaces
/// the heading and cells, which would leave focus on `body`.
pub(super) fn use_focus_after_render(root: ElementHandle) -> Signal<Option<String>> {
    let mut focus_request = use_signal(|| None::<String>);
    use_effect(move || {
        let Some(selector) = focus_request() else {
            return;
        };
        focus_request.set(None);
        let _ = root
            .query_selector(&selector)
            .and_then(|element| element.focus());
    });
    focus_request
}

/// The engine every date picker draws: a header, then a day, month or year view.
/// Non-generic: a [`Selection`] of plain values keeps the props comparable.
#[component]
pub(super) fn Calendar(props: CalendarProps) -> Element {
    let names = &use_localization().date;
    let formats = use_formats();
    let selection = props.selection;
    let onpick = props.onpick;
    let columns = props.columns.max(1) as i64;
    let lowest = props.lowest;
    let (min, max, exclude_date) = (props.min, props.max, props.exclude_date.0);
    let focusable = props.focusable;

    let today = use_today(props.today);
    // A new `lowest` remounts the calendar (`DateValue::picker` keys it).
    let mut level = use_signal(|| lowest);
    let mut paged = use_signal(|| None::<NaiveDate>);
    let active = use_signal(|| None::<NaiveDate>);
    let mut nav_targets = use_hook(|| CopyValue::new([FALLBACK_MONTH; 2]));
    let root = use_element();
    let focus = FocusRequest {
        root,
        request: use_focus_after_render(root),
        focusable,
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
    let view = View {
        names,
        formats,
        selection,
        onpick,
        lowest,
        level,
        paged,
        nav_targets,
        active,
        focus,
        first,
        last,
        columns,
        decade: first.year() - first.year().rem_euclid(10),
        today,
        min,
        max,
        exclude_date,
        size: props.size,
        focusable,
    };
    let shown = move |day: NaiveDate| (first..=last).contains(&first_of_month(day));
    let tab_stop = active()
        .filter(|day| shown(*day))
        .or(selection.anchor().filter(|day| shown(*day)))
        .or(today.filter(|day| shown(*day)))
        .unwrap_or(first);
    // A disabled day cannot take focus: move the stop to the nearest shown one that can.
    let tab_stop = view.nearest_in(tab_stop, shown).unwrap_or(tab_stop);

    let first_weekday = formats.first_weekday.num_days_from_sunday() as usize;
    let column = move |day: NaiveDate| {
        ((day.weekday().num_days_from_sunday() as usize + 7 - first_weekday) % 7) as i64
    };
    let onkeydown = move |event: KeyboardEvent| view.day_keydown(event, tab_stop, column(tab_stop));
    let cell_stop = view.cell_stop();

    let awaits_end = selection.awaits_end();
    // Read only while a range waits, so a plain picker never re-renders on mouse movement.
    let hovered = if awaits_end { hover() } else { None };

    // The first day of each of a month's six weeks.
    let weeks = move |month: NaiveDate| {
        let start = add_days(month, -column(month));
        (0..6).map(move |week| (week, add_days(start, week * 7)))
    };
    let in_week = |day: Option<NaiveDate>, first: NaiveDate| {
        day.filter(|day| (first..=add_days(first, 6)).contains(day))
    };
    let size = props.size;

    let body = match level() {
        _ if props.variant == CalendarVariant::Mini => {
            view.strip_view(Strip::new(view, props.days))
        }
        DateLevel::Day => {
            nav_targets.set([add_months(first, -1), add_months(first, 1)]);
            let months = (0..columns).map(|index| (index, add_months(first, index)));
            // Inline, see `Week`. Keyed by column, so paging diffs the grid in place.
            rsx! {
                div { "data-slot": "header",
                    Nav {
                        label: names.previous_month,
                        disabled: min.is_some_and(|min| add_days(first, -1) < min),
                        targets: nav_targets,
                        forward: false,
                        size,
                        focusable,
                        paged,
                    }
                    for (index, month) in months.clone() {
                        button {
                            key: "{index}",
                            r#type: "button",
                            "data-slot": "title",
                            "aria-live": "polite",
                            tabindex: view.tabindex(true),
                            onclick: move |_| {
                                paged.set(Some(month));
                                level.set(DateLevel::Month);
                                focus.to(Focus::Title);
                            },
                            {format_date(month, formats.month_heading, names)}
                        }
                    }
                    Nav {
                        label: names.next_month,
                        disabled: max.is_some_and(|max| add_months(last, 1) > max),
                        targets: nav_targets,
                        forward: true,
                        size,
                        focusable,
                        paged,
                    }
                }
                div {
                    "data-slot": "months",
                    onmouseleave: move |_| {
                        if awaits_end {
                            hover.set(None);
                        }
                    },
                    for (index, month) in months {
                        div {
                            key: "{index}",
                            role: "grid",
                            "aria-label": format_date(month, formats.month_heading, names),
                            onkeydown,
                            Weekdays { first_weekday }
                            for (week, first) in weeks(month) {
                                Week {
                                    key: "{week}",
                                    first,
                                    month,
                                    blanks: columns > 1,
                                    selection: selection.clip(hovered, first, add_days(first, 6)),
                                    today: in_week(today, first),
                                    tab_stop: in_week(Some(tab_stop), first),
                                    min,
                                    max,
                                    exclude_date,
                                    focusable,
                                    awaits_end,
                                    hover,
                                    active,
                                    paged,
                                    onpick,
                                }
                            }
                        }
                    }
                }
            }
        }
        DateLevel::Month => view.month_view(cell_stop),
        DateLevel::Year => view.year_view(cell_stop),
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
