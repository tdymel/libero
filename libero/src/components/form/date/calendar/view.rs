use dioxus::prelude::*;

use chrono::{Datelike, NaiveDate};

use crate::{
    components::common::has_shortcut_modifier,
    hooks::ElementHandle,
    localization::{DateLocale, Formats},
    platform::{ElementApi, logical_key},
    theme::Size,
};

use super::{DateLevel, FALLBACK_MONTH, Focus, Selection, add_days, add_months, first_of_month};

/// Where focus goes after the next render. Only moves focus already in the calendar,
/// so a click in a field's dropdown leaves it on the input.
#[derive(Clone, Copy)]
pub(super) struct FocusRequest {
    pub(super) root: ElementHandle,
    pub(super) request: Signal<Option<String>>,
    pub(super) focusable: bool,
}

impl FocusRequest {
    pub(super) fn to(self, target: Focus) {
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

/// What the keys and the views of every level read, for one render.
#[derive(Clone, Copy)]
pub(super) struct View {
    pub(super) names: &'static DateLocale,
    pub(super) formats: &'static Formats,
    pub(super) selection: Selection,
    pub(super) onpick: EventHandler<NaiveDate>,
    pub(super) lowest: DateLevel,
    pub(super) level: Signal<DateLevel>,
    /// Only ever set by paging; until then the view follows the value.
    pub(super) paged: Signal<Option<NaiveDate>>,
    /// Where the back and forward buttons page to, written each render.
    pub(super) nav_targets: CopyValue<[NaiveDate; 2]>,
    /// The keyboard's day.
    pub(super) active: Signal<Option<NaiveDate>>,
    pub(super) focus: FocusRequest,
    /// The first and last month shown.
    pub(super) first: NaiveDate,
    pub(super) last: NaiveDate,
    pub(super) columns: i64,
    /// The first year of the decade `first` is in.
    pub(super) decade: i32,
    pub(super) today: Option<NaiveDate>,
    pub(super) min: Option<NaiveDate>,
    pub(super) max: Option<NaiveDate>,
    pub(super) exclude_date: Option<Callback<NaiveDate, bool>>,
    pub(super) size: Size,
    pub(super) focusable: bool,
}

impl View {
    pub(super) fn tabindex(self, stop: bool) -> &'static str {
        if self.focusable && stop { "0" } else { "-1" }
    }

    pub(super) fn day_disabled(self, day: NaiveDate) -> bool {
        self.min.is_some_and(|min| day < min)
            || self.max.is_some_and(|max| day > max)
            || self.exclude_date.is_some_and(|exclude| exclude.call(day))
    }

    /// `day` pulled inside `min`/`max`.
    pub(super) fn clamp_day(self, day: NaiveDate) -> NaiveDate {
        let mut day = day;
        if let Some(min) = self.min {
            day = day.max(min);
        }
        if let Some(max) = self.max {
            day = day.min(max);
        }
        day
    }

    /// The first pickable day from `from`, `step` days at a time. `None` once a step
    /// leaves `min`/`max`, or after a year of excluded days.
    pub(super) fn seek(self, from: NaiveDate, step: i64) -> Option<NaiveDate> {
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
    pub(super) fn nearest(self, from: NaiveDate, step: i64) -> Option<NaiveDate> {
        let from = self.clamp_day(from);
        self.seek(from, step).or_else(|| self.seek(from, -step))
    }

    /// The day a tab stop moves to when `from` is disabled: the next one
    /// that is not, else the previous one, as long as it is `shown`.
    pub(super) fn nearest_in(
        self,
        from: NaiveDate,
        shown: impl Fn(NaiveDate) -> bool,
    ) -> Option<NaiveDate> {
        let from = self.clamp_day(from);
        [1, -1]
            .into_iter()
            .filter_map(|step| self.seek(from, step))
            .find(|day| shown(*day))
    }

    /// The day grid's keys, from `tab_stop` in weekday column `column`.
    pub(super) fn day_keydown(self, event: KeyboardEvent, tab_stop: NaiveDate, column: i64) {
        // Alt+ArrowLeft is Back, Ctrl+PageUp switches tabs: a chord is the browser's.
        if has_shortcut_modifier(&event) {
            return;
        }
        let years = if event.modifiers().shift() { 12 } else { 1 };
        // Skip disabled days: `focus()` on one does nothing and the grid would lose its tab stop.
        let next = match logical_key(&event) {
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

    /// The month and year views' one tab stop, in a grid three wide.
    pub(super) fn cell_stop(self) -> NaiveDate {
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

    /// A month or year cell pulled inside the cells of `min` and `max`, the only ones disabled.
    pub(super) fn clamp_cell(self, cell: NaiveDate) -> NaiveDate {
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
    pub(super) fn cell_of(self, day: NaiveDate) -> NaiveDate {
        match (self.level)() {
            DateLevel::Year => NaiveDate::from_ymd_opt(day.year(), 1, 1).unwrap_or(day),
            _ => first_of_month(day),
        }
    }

    /// A step off the shown year or decade pages it.
    pub(super) fn cell_keydown(self, event: KeyboardEvent, cell_stop: NaiveDate) {
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
        let cells = match logical_key(&event) {
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
}

/// The mini variant's row of days.
#[derive(Clone, Copy)]
pub(super) struct Strip {
    pub(super) start: NaiveDate,
    pub(super) end: NaiveDate,
    pub(super) days: i64,
    /// The row's one tab stop.
    pub(super) stop: NaiveDate,
}

impl Strip {
    pub(super) fn new(view: View, days: usize) -> Self {
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

    pub(super) fn keydown(self, view: View, event: KeyboardEvent) {
        if has_shortcut_modifier(&event) {
            return;
        }
        let next = match logical_key(&event) {
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
        // A step off an end moves the row as far.
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
