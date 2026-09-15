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
        ActionIcon, ClassList, HtmlTag, Input, States, Variant,
        common::{
            ChevronLeftIcon, ChevronRightIcon, focus_ring_sx, has_shortcut_modifier,
            inset_focus_ring_sx,
        },
        layout::use_box,
    },
    hooks::{ElementHandle, use_element, use_theme},
    platform::ElementApi,
    sx::{FORCED_COLORS, StaticSx, Sx, ThemeAwareValue, sx},
    theme::{
        CalendarVariant, DATE_PICKER_DAY, DATE_PICKER_FONT_SIZE, DateDefaults, DatePickerDefaults,
        Size, SizeCss,
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
        .hover(sx().background("muted.1"));
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
        .selector("& [data-outside]", sx().color("text-dimmed"))
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
            inset_focus_ring_sx("-4px"),
        )
        // Forced colours paint the picked day's fill `Canvas`, like every other,
        // drop the range tint, and draw every transparent border.
        .media(
            FORCED_COLORS,
            sx().selector(
                "& :is([data-slot='day'], [data-slot='cell'])",
                sx().border_color("Canvas"),
            )
            .selector("& [data-in-range]", sx().border_color("Highlight"))
            .selector("& [data-today]", sx().border_color("CanvasText"))
            .selector(
                "& [data-selected]",
                sx().background("Highlight")
                    .color("HighlightText")
                    .hover(sx().background("Highlight")),
            ),
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

// In the theme, whose `DateDefaults::format` takes it; the layers below
// `components` may not import from it.
pub use crate::theme::DateLevel;

/// Where focus goes after the next render.
#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Date(NaiveDate),
    Title,
    /// The day or cell that holds the tab stop, wherever it moved to.
    Stop,
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

/// Where focus goes after the next render. Only moves focus that is already
/// in the calendar: a click inside a field's dropdown leaves it on the text
/// input.
#[derive(Clone, Copy)]
struct FocusRequest {
    root: ElementHandle,
    request: Signal<Option<String>>,
    focusable: bool,
}

impl FocusRequest {
    fn to(self, target: Focus) {
        if self.focusable && self.root.query_selector(":focus").is_ok() {
            let mut request = self.request;
            request.set(Some(match target {
                Focus::Title => "[data-slot='title']".to_string(),
                Focus::Stop => "[role='grid'] [tabindex='0']".to_string(),
                Focus::Date(day) => format!("[data-date='{day}']:not([data-outside])"),
            }));
        }
    }
}

/// A selector to focus inside `root` after the next render, which drew the
/// target. A level change replaces the heading and the cells, which would
/// leave focus on `body`.
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

/// What the keys and the views of every level read, for one render.
#[derive(Clone, Copy)]
struct View {
    names: &'static DateDefaults,
    selection: Selection,
    onpick: EventHandler<NaiveDate>,
    lowest: DateLevel,
    level: Signal<DateLevel>,
    /// Only ever set by paging; until then the view follows the value.
    paged: Signal<Option<NaiveDate>>,
    /// Where the back and forward buttons page to, written each render.
    nav_targets: CopyValue<[NaiveDate; 2]>,
    /// The keyboard's day.
    active: Signal<Option<NaiveDate>>,
    focus: FocusRequest,
    /// The first and last month shown.
    first: NaiveDate,
    last: NaiveDate,
    columns: i64,
    /// The first year of the decade `first` is in.
    decade: i32,
    today: Option<NaiveDate>,
    min: Option<NaiveDate>,
    max: Option<NaiveDate>,
    exclude_date: Option<Callback<NaiveDate, bool>>,
    size: Size,
    focusable: bool,
}

impl View {
    fn tabindex(self, stop: bool) -> &'static str {
        if self.focusable && stop { "0" } else { "-1" }
    }

    fn day_disabled(self, day: NaiveDate) -> bool {
        self.min.is_some_and(|min| day < min)
            || self.max.is_some_and(|max| day > max)
            || self.exclude_date.is_some_and(|exclude| exclude.call(day))
    }

    /// `day` pulled inside `min`/`max`.
    fn clamp_day(self, day: NaiveDate) -> NaiveDate {
        let mut day = day;
        if let Some(min) = self.min {
            day = day.max(min);
        }
        if let Some(max) = self.max {
            day = day.min(max);
        }
        day
    }

    /// The first day a pick can land on, from `from` on, `step` days at a
    /// time. `None` once a step leaves `min`/`max`, or after a year of steps
    /// that `exclude_date` all refuses.
    fn seek(self, from: NaiveDate, step: i64) -> Option<NaiveDate> {
        let mut day = from;
        for _ in 0..366 {
            if self.clamp_day(day) != day {
                return None;
            }
            if !self.day_disabled(day) {
                return Some(day);
            }
            let next = add_days(day, step);
            if next == day {
                return None;
            }
            day = next;
        }
        None
    }

    /// [`View::seek`] from `from` pulled inside `min`/`max`, in the direction
    /// of `step`, then back the other way.
    fn nearest(self, from: NaiveDate, step: i64) -> Option<NaiveDate> {
        let from = self.clamp_day(from);
        self.seek(from, step).or_else(|| self.seek(from, -step))
    }

    /// The day a tab stop moves to when `from` is disabled: the next one
    /// that is not, else the previous one, as long as it is `shown`.
    fn nearest_in(self, from: NaiveDate, shown: impl Fn(NaiveDate) -> bool) -> Option<NaiveDate> {
        let from = self.clamp_day(from);
        [1, -1]
            .into_iter()
            .filter_map(|step| self.seek(from, step))
            .find(|day| shown(*day))
    }

    fn nav(self, label: &'static str, disabled: bool, target: NaiveDate, forward: bool) -> Element {
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

    /// The day grid's keys, from `tab_stop` in weekday column `column`.
    fn day_keydown(self, event: KeyboardEvent, tab_stop: NaiveDate, column: i64) {
        // Alt+ArrowLeft is Back, Ctrl+PageUp switches tabs: a chord is the browser's.
        if has_shortcut_modifier(&event) {
            return;
        }
        let years = if event.modifiers().shift() { 12 } else { 1 };
        // Disabled days are skipped: `focus()` on one does nothing, and the
        // grid would lose its only tab stop. An arrow with nothing left to
        // land on stays put.
        let next = match event.key() {
            Key::ArrowLeft => self.seek(add_days(tab_stop, -1), -1),
            Key::ArrowRight => self.seek(add_days(tab_stop, 1), 1),
            Key::ArrowUp => self.seek(add_days(tab_stop, -7), -7),
            Key::ArrowDown => self.seek(add_days(tab_stop, 7), 7),
            Key::Home => self.nearest(add_days(tab_stop, -column), 1),
            Key::End => self.nearest(add_days(tab_stop, 6 - column), -1),
            Key::PageUp => self.nearest(add_months(tab_stop, -years), -1),
            Key::PageDown => self.nearest(add_months(tab_stop, years), 1),
            _ => return,
        };
        event.prevent_default();
        let Some(next) = next else {
            return;
        };
        let (mut paged, mut active) = (self.paged, self.active);
        let month = first_of_month(next);
        if month < self.first {
            paged.set(Some(month));
        } else if month > self.last {
            paged.set(Some(add_months(month, 1 - self.columns)));
        }
        active.set(Some(next));
        self.focus.to(Focus::Date(next));
    }

    /// The month and year views' one tab stop: a grid three wide, one cell a
    /// month or a year.
    fn cell_stop(self) -> NaiveDate {
        let level = (self.level)();
        let in_view = |cell: NaiveDate| match level {
            DateLevel::Year => (self.decade..self.decade + 10).contains(&cell.year()),
            _ => cell.year() == self.first.year(),
        };
        let stop = [(self.active)(), self.selection.anchor(), self.today]
            .into_iter()
            .flatten()
            .map(|day| self.cell_of(day))
            .find(|cell| in_view(*cell))
            .unwrap_or_else(|| {
                let year = if level == DateLevel::Year {
                    self.decade
                } else {
                    self.first.year()
                };
                NaiveDate::from_ymd_opt(year, 1, 1).unwrap_or(self.first)
            });
        // Onto the nearest cell that is not disabled, while one is in view.
        Some(self.clamp_cell(stop))
            .filter(|cell| in_view(*cell))
            .unwrap_or(stop)
    }

    /// A month or year cell pulled inside the cells of `min` and `max`: the
    /// only ones these views disable, and always at the ends.
    fn clamp_cell(self, cell: NaiveDate) -> NaiveDate {
        let mut cell = cell;
        if let Some(min) = self.min {
            cell = cell.max(self.cell_of(min));
        }
        if let Some(max) = self.max {
            cell = cell.min(self.cell_of(max));
        }
        cell
    }

    /// The month or year cell `day` falls in.
    fn cell_of(self, day: NaiveDate) -> NaiveDate {
        match (self.level)() {
            DateLevel::Year => NaiveDate::from_ymd_opt(day.year(), 1, 1).unwrap_or(day),
            _ => first_of_month(day),
        }
    }

    /// A step off the shown year or decade pages it.
    fn cell_keydown(self, event: KeyboardEvent, cell_stop: NaiveDate) {
        if has_shortcut_modifier(&event) {
            return;
        }
        let level = (self.level)();
        let (months_per_cell, cells_per_page, column) = match level {
            DateLevel::Day => return,
            DateLevel::Month => (1, 12, i64::from(cell_stop.month0() % 3)),
            DateLevel::Year => (
                12,
                10,
                i64::from((cell_stop.year() - self.decade + 1).rem_euclid(3)),
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
        let (mut paged, mut active) = (self.paged, self.active);
        let next = self.clamp_cell(add_months(cell_stop, cells * months_per_cell));
        // A year cell is January 1; the page keeps the month a pick climbs back down to.
        let page = match level {
            DateLevel::Year => NaiveDate::from_ymd_opt(next.year(), self.first.month(), 1),
            _ => None,
        };
        paged.set(Some(page.unwrap_or(next)));
        active.set(Some(next));
        self.focus.to(Focus::Date(next));
    }

    /// The mini variant: one row of `strip.days` days from its own first day.
    /// The buttons page it; an arrow key past an end slides it.
    fn strip_view(self, strip: Strip) -> Element {
        let View {
            names,
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
                        "data-slot": "day",
                        "data-date": "{day}",
                        "data-today": (today == Some(day)).then_some("true"),
                        "aria-current": (today == Some(day)).then_some("date"),
                        "data-selected": picked.then_some("true"),
                        "aria-label": format_date(day, (names.format)(DateLevel::Day), names),
                        disabled: self.day_disabled(day).then_some(true),
                        tabindex: self.tabindex(day == stop),
                        onclick: move |_| {
                            // Pinned, so the new value does not move the row.
                            paged.set(Some(start));
                            active.set(Some(day));
                            onpick.call(day);
                        },
                        span { "data-slot": "month", {format_date(day, "MMM", names)} }
                        span { "{day.day()}" }
                    }
                }
            }
        };
        rsx! {
            div { "data-slot": "strip",
                {self.nav(names.previous_days, min.is_some_and(|min| add_days(start, -1) < min), add_days(start, -days), false)}
                // The slot a field's dropdown looks for to hand focus in.
                div { "data-slot": "months",
                    div {
                        role: "grid",
                        "aria-label": format_date(start, names.month_format, names),
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

    fn month_view(self, cell_stop: NaiveDate) -> Element {
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
            rsx! {
                button {
                    r#type: "button",
                    role: "gridcell",
                    "aria-selected": if picked { "true" } else { "false" },
                    "data-slot": "cell",
                    "data-date": "{month}",
                    "data-selected": picked.then_some("true"),
                    "data-today": is_today.then_some("true"),
                    "aria-current": is_today.then_some("date"),
                    "aria-label": format_date(month, (names.format)(DateLevel::Month), names),
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
        };
        rsx! {
            div { "data-slot": "header",
                {self.nav(names.previous_year, min.is_some_and(|min| year <= min.year()), add_months(first, -12), false)}
                button {
                    r#type: "button",
                    "data-slot": "title",
                    "aria-live": "polite",
                    tabindex: self.tabindex(true),
                    onclick: move |_| {
                        level.set(DateLevel::Year);
                        // The decade's title is disabled: there is no level
                        // above it. Focus goes to the year the view stands on.
                        focus.to(Focus::Stop);
                    },
                    "{year}"
                }
                {self.nav(names.next_year, max.is_some_and(|max| year >= max.year()), add_months(first, 12), true)}
            }
            div {
                "data-slot": "cells",
                role: "grid",
                "aria-label": "{year}",
                onkeydown: move |event| self.cell_keydown(event, cell_stop),
                // Rows written out, not looped: one template, cells diffed in
                // place.
                div { role: "row", {cell(0)} {cell(1)} {cell(2)} }
                div { role: "row", {cell(3)} {cell(4)} {cell(5)} }
                div { role: "row", {cell(6)} {cell(7)} {cell(8)} }
                div { role: "row", {cell(9)} {cell(10)} {cell(11)} }
            }
        }
    }

    fn year_view(self, cell_stop: NaiveDate) -> Element {
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
                button {
                    r#type: "button",
                    role: "gridcell",
                    "aria-selected": if picked { "true" } else { "false" },
                    "data-slot": "cell",
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
        };
        rsx! {
            div { "data-slot": "header",
                {self.nav(names.previous_decade, min.is_some_and(|min| decade <= min.year()), add_months(first, -120), false)}
                button {
                    r#type: "button",
                    "data-slot": "title",
                    "aria-live": "polite",
                    disabled: true,
                    "{decade} – {decade + 9}"
                }
                {self.nav(names.next_decade, max.is_some_and(|max| decade + 9 >= max.year()), add_months(first, 120), true)}
            }
            div {
                "data-slot": "cells",
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

/// The mini variant's row of days.
#[derive(Clone, Copy)]
struct Strip {
    start: NaiveDate,
    end: NaiveDate,
    days: i64,
    /// The row's one tab stop.
    stop: NaiveDate,
}

impl Strip {
    fn new(view: View, days: usize) -> Self {
        let days = days.max(1) as i64;
        let start = (view.paged)()
            .or(view.selection.anchor())
            .or(view.today)
            .unwrap_or(FALLBACK_MONTH);
        let end = add_days(start, days - 1);
        let in_strip = |day: NaiveDate| (start..=end).contains(&day);
        let stop = [(view.active)(), view.selection.anchor(), view.today]
            .into_iter()
            .flatten()
            .find(|day| in_strip(*day))
            .unwrap_or(start);
        let stop = view.nearest_in(stop, in_strip).unwrap_or(stop);
        Self {
            start,
            end,
            days,
            stop,
        }
    }

    fn keydown(self, view: View, event: KeyboardEvent) {
        if has_shortcut_modifier(&event) {
            return;
        }
        let next = match event.key() {
            Key::ArrowLeft => view.seek(add_days(self.stop, -1), -1),
            Key::ArrowRight => view.seek(add_days(self.stop, 1), 1),
            Key::Home => view.nearest(self.start, 1),
            Key::End => view.nearest(self.end, -1),
            Key::PageUp => view.nearest(add_days(self.stop, -self.days), -1),
            Key::PageDown => view.nearest(add_days(self.stop, self.days), 1),
            _ => return,
        };
        event.prevent_default();
        let Some(next) = next else {
            return;
        };
        // A step off an end moves the row as far: an arrow a day, a page a
        // page.
        let start = match (self.start..=self.end).contains(&next) {
            true => self.start,
            false => add_days(self.start, (next - self.stop).num_days()),
        };
        let (mut paged, mut active) = (view.paged, view.active);
        paged.set(Some(start));
        active.set(Some(next));
        view.focus.to(Focus::Date(next));
    }
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

    let today = use_today(props.today);
    // A new `lowest` remounts the calendar - `DateValue::picker` keys it - so
    // the view only has to start there.
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
    // A disabled day cannot take focus, so the stop moves to the nearest one
    // that can, while one is shown.
    let tab_stop = view.nearest_in(tab_stop, shown).unwrap_or(tab_stop);

    let first_weekday = names.first_weekday.num_days_from_sunday() as usize;
    let column = move |day: NaiveDate| {
        ((day.weekday().num_days_from_sunday() as usize + 7 - first_weekday) % 7) as i64
    };
    let onkeydown = move |event: KeyboardEvent| view.day_keydown(event, tab_stop, column(tab_stop));
    let cell_stop = view.cell_stop();

    let awaits_end = selection.awaits_end();
    // Read only while a range waits, so a plain picker never re-renders on
    // mouse movement.
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
            // Items drawn inline: see `Week`. Keyed by column, so a page
            // diffs the grid in place rather than remounting it.
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
                            {format_date(month, names.month_format, names)}
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
                            "aria-label": format_date(month, names.month_format, names),
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

/// One row of a month grid. Its own scope, and the calendar hands it only what
/// lands on its seven days, so a pick or a range preview elsewhere leaves its
/// props equal and it skips the re-render.
#[derive(Props, Clone, PartialEq)]
struct WeekProps {
    first: NaiveDate,
    /// The month the grid shows. The other days are its neighbours'.
    month: NaiveDate,
    /// Side by side, a neighbour's days would appear twice: draw blanks.
    blanks: bool,
    /// Clipped to the week.
    selection: Selection,
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
fn Week(props: WeekProps) -> Element {
    let WeekProps {
        first,
        month,
        blanks,
        selection,
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
    let names = &use_theme().date;
    // APG: the full date, not only the number shown.
    let label = date_formatter((names.format)(DateLevel::Day), names);
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
        (offset, day, outside(day), picked, in_range)
    };

    // Items drawn inline, not through a helper returning `rsx!`: a nested
    // element per item costs about a microsecond each. Keyed by column, so a
    // page diffs the days in place rather than remounting them.
    rsx! {
        div { role: "row",
            for offset in 0..lead {
                div { key: "{offset}", role: "gridcell", "data-slot": "blank" }
            }
            for (offset, day, outside, picked, in_range) in (lead..7 - trail).map(cell) {
                div {
                    key: "{offset}",
                    role: "gridcell",
                    "aria-selected": if picked { "true" } else { "false" },
                    button {
                        r#type: "button",
                        "data-slot": "day",
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
                div { key: "{offset}", role: "gridcell", "data-slot": "blank" }
            }
        }
    }
}

/// The weekday names over a month. Its own scope: nothing in it moves with the
/// calendar's state, so it skips every re-render.
#[component]
fn Weekdays(first_weekday: usize) -> Element {
    let names = &use_theme().date;
    rsx! {
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
    }
}

/// A button that pages the calendar. Its own scope with its icon drawn inside,
/// and its target read on click, so a page leaves its props equal.
#[derive(Props, Clone, PartialEq)]
struct NavProps {
    label: &'static str,
    disabled: bool,
    targets: CopyValue<[NaiveDate; 2]>,
    forward: bool,
    size: Size,
    focusable: bool,
    paged: Signal<Option<NaiveDate>>,
}

#[component]
fn Nav(props: NavProps) -> Element {
    let (targets, forward, mut paged) = (props.targets, props.forward, props.paged);
    rsx! {
        ActionIcon {
            aria_label: props.label,
            variant: Input::Value(Variant::Standard),
            size: ThemeAwareValue::Size(nav_size(props.size)),
            tabindex: if props.focusable { "0" } else { "-1" },
            disabled: props.disabled,
            onclick: move |_| paged.set(Some(targets.peek()[usize::from(forward)])),
            if props.forward {
                ChevronRightIcon {}
            } else {
                ChevronLeftIcon {}
            }
        }
    }
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
    fn a_clipped_selection_marks_its_days_as_the_whole_one_does() {
        let hover = Some(day(24));
        let selections = [
            Selection::Single(Some(day(9))),
            Selection::Range(Some(DateRange::new(day(20), Some(day(3))))),
            Selection::Range(Some(DateRange::new(day(10), None))),
        ];
        for selection in selections {
            for first in (1..=22).map(day) {
                let last = add_days(first, 6);
                let clipped = selection.clip(hover, first, last);
                for offset in 0..7 {
                    let shown = add_days(first, offset);
                    assert_eq!(clipped.marks(shown, None), selection.marks(shown, hover));
                }
            }
        }
        // A range moving past a week leaves the week's marks equal.
        let range = |end| Selection::Range(Some(DateRange::new(day(1), Some(day(end)))));
        assert_eq!(
            range(20).clip(None, day(8), day(14)),
            range(27).clip(None, day(8), day(14))
        );
    }

    #[test]
    fn a_complete_range_ignores_the_mouse() {
        let complete = Selection::Range(Some(DateRange::new(day(10), Some(day(12)))));
        assert_eq!(complete.marks(day(11), Some(day(20))), (false, true));
        assert_eq!(complete.marks(day(15), Some(day(20))), (false, false));
        assert!(!complete.awaits_end());
    }
}
