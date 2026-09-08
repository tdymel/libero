//! The date and time pickers and fields as a server renders them: the grid,
//! the marked days, and what posts with a form.

use crate::common::{body, render};

use std::cell::Cell;

use dioxus::dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime},
    components::{
        DateField, DateLevel, DatePicker, DateRange, DateRangePicker, DayField, DayPicker,
        MonthPicker, SegmentedControl, TimePicker, YearPicker,
    },
    theme::CalendarVariant,
};

#[test]
fn a_time_picker_draws_a_column_per_part_and_posts_the_time() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TimePicker { value: NaiveTime::from_hms_opt(9, 30, 0), onchange: move |_| {}, variant: "digital", step: 15, name: "at" }
            }
        }
    }
    let html = body(&render(app));

    // 24 hours, then 00, 15, 30 and 45.
    assert_eq!(html.matches("data-slot=\"option\"").count(), 28);
    assert_eq!(html.matches("data-selected=\"true\"").count(), 2);
    assert_eq!(html.matches("aria-pressed=\"true\"").count(), 2);
    assert_eq!(html.matches("aria-pressed=\"false\"").count(), 26);
    assert!(html.contains("type=\"hidden\" name=\"at\" value=\"09:30:00\""));
}

#[test]
fn an_analog_time_picker_tells_its_toggles_apart_by_aria_pressed() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TimePicker { value: NaiveTime::from_hms_opt(21, 30, 0), onchange: move |_| {}, variant: "analog", twelve_hour: true }
            }
        }
    }
    let html = body(&render(app));

    // The hour hand and PM are pressed; the minute hand and AM are not.
    assert_eq!(html.matches("aria-pressed=\"true\"").count(), 2);
    assert_eq!(html.matches("aria-pressed=\"false\"").count(), 2);
    let pressed = html
        .split("aria-pressed=\"true\"")
        .nth(2)
        .expect("a second pressed toggle");
    assert!(
        pressed
            .split("</button>")
            .next()
            .unwrap_or_default()
            .contains(">PM")
    );
}

#[test]
fn a_range_picker_blanks_neighbour_days_and_tints_the_inside() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DateRangePicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14).map(|start| DateRange::new(start, NaiveDate::from_ymd_opt(2026, 9, 18))),
                    onchange: move |_| {},
                    name: "stay",
                }
            }
        }
    }
    let html = body(&render(app));

    assert_eq!(html.matches("role=\"grid\"").count(), 2);
    // Side by side, only each month's own days are buttons: 30 + 31.
    assert_eq!(html.matches("data-slot=\"day\"").count(), 61);
    assert_eq!(html.matches("data-selected=\"true\"").count(), 2);
    assert_eq!(html.matches("data-in-range=\"true\"").count(), 3);
    assert!(html.contains("name=\"stay\" value=\"2026-09-14/2026-09-18\""));
}

#[test]
fn a_non_focusable_segmented_control_leaves_the_tab_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl::<String> {
                    value: "a".to_string(),
                    options: vec!["a".to_string(), "b".to_string()],
                    onchange: move |_| {},
                    focusable: false,
                }
            }
        }
    }
    let html = body(&render(app));

    assert_eq!(html.matches("type=\"radio\"").count(), 2);
    assert_eq!(html.matches("tabindex=\"-1\"").count(), 2);
}

#[test]
fn a_picker_draws_six_weeks_from_the_first_weekday() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, name: "day" }
            }
        }
    }
    let html = body(&render(app));

    assert_eq!(html.matches("data-slot=\"day\"").count(), 42);
    // September 2026 starts on a Tuesday, so a Monday-first grid opens on
    // August 31 and ends on October 11.
    let first = html.find("data-date=\"2026-08-31\"").expect("Monday first");
    assert!(first < html.find("data-date=\"2026-09-01\"").expect("the 1st"));
    assert!(html.contains("data-date=\"2026-10-11\""));
    assert!(!html.contains("data-date=\"2026-10-12\""));
    assert!(html.contains("September 2026"));

    assert_eq!(html.matches("data-selected=\"true\"").count(), 1);
    assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
    // The picked day is the grid's one tab stop, beside the heading and the two page buttons.
    assert_eq!(html.matches("tabindex=\"0\"").count(), 4);
    assert!(html.contains("name=\"day\""));
    assert!(html.contains("value=\"2026-09-14\""));
}

#[test]
fn days_outside_min_and_max_are_disabled() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    min: NaiveDate::from_ymd_opt(2026, 9, 10),
                    max: NaiveDate::from_ymd_opt(2026, 9, 20),
                    onchange: move |_| {},
                }
            }
        }
    }
    let html = body(&render(app));

    let cell = |date: &str| {
        let at = html
            .find(&format!("data-date=\"{date}\""))
            .expect("a day cell");
        let end = at + html[at..].find('>').expect("the tag ends");
        html[at..end].to_string()
    };
    assert!(cell("2026-09-09").contains("disabled"));
    assert!(!cell("2026-09-10").contains("disabled"));
    assert!(!cell("2026-09-20").contains("disabled"));
    assert!(cell("2026-09-21").contains("disabled"));
}

/// The one `tabindex="0"` day in `html`.
fn day_stop(html: &str) -> &str {
    let stops = tags_with(html, &["data-slot=\"day\"", "tabindex=\"0\""]);
    assert_eq!(stops.len(), 1, "{stops:?}");
    stops[0]
}

/// A disabled day cannot take focus, so it never holds the grid's tab stop.
#[test]
fn the_tab_stop_skips_disabled_days() {
    fn min_after_the_first() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker {
                    value: None,
                    min: NaiveDate::from_ymd_opt(2026, 9, 10),
                    today: NaiveDate::from_ymd_opt(2026, 9, 1),
                    onchange: move |_| {},
                }
            }
        }
    }
    fn weekend_today() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker {
                    value: None,
                    // A Saturday.
                    today: NaiveDate::from_ymd_opt(2026, 9, 19),
                    exclude_date: move |day: NaiveDate| day.weekday().num_days_from_monday() >= 5,
                    onchange: move |_| {},
                }
            }
        }
    }
    fn weekend_at_the_month_end() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker {
                    value: None,
                    // A Saturday; the Monday after is in June.
                    today: NaiveDate::from_ymd_opt(2026, 5, 30),
                    exclude_date: move |day: NaiveDate| day.weekday().num_days_from_monday() >= 5,
                    onchange: move |_| {},
                }
            }
        }
    }
    fn mini_weekend() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 19),
                    exclude_date: move |day: NaiveDate| day.weekday().num_days_from_monday() >= 5,
                    calendar: "mini",
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = body(&render(min_after_the_first));
    assert!(day_stop(&html).contains("data-date=\"2026-09-10\""));
    let html = body(&render(weekend_today));
    assert!(day_stop(&html).contains("data-date=\"2026-09-21\""));
    // Not onto a day of the next month: back to the Friday.
    let html = body(&render(weekend_at_the_month_end));
    assert!(day_stop(&html).contains("data-date=\"2026-05-29\""));
    let html = body(&render(mini_weekend));
    assert!(day_stop(&html).contains("data-date=\"2026-09-21\""));
}

/// The month and year views disable only the cells past `min` and `max`.
#[test]
fn a_month_picker_keeps_its_tab_stop_inside_its_limits() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                MonthPicker {
                    value: None,
                    today: NaiveDate::from_ymd_opt(2026, 1, 15),
                    min: NaiveDate::from_ymd_opt(2026, 4, 10),
                    onchange: move |_| {},
                }
            }
        }
    }
    let html = body(&render(app));

    let stops = tags_with(&html, &["data-slot=\"cell\"", "tabindex=\"0\""]);
    assert_eq!(stops.len(), 1);
    assert!(stops[0].contains("data-date=\"2026-04-01\""));
}

#[test]
fn a_field_shows_its_format_and_posts_iso() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DayField { value: NaiveDate::from_ymd_opt(2026, 9, 4), onchange: move |_| {}, name: "arrival" }
                DayField {
                    value: NaiveDate::from_ymd_opt(2026, 9, 4),
                    onchange: move |_| {},
                    format: "DD.MM.YYYY",
                }
            }
        }
    }
    let html = body(&render(app));

    assert!(html.contains("value=\"September 4, 2026\""));
    assert!(html.contains("value=\"04.09.2026\""));
    // The name sits on the hidden input, which holds ISO 8601.
    assert!(html.contains("type=\"hidden\" name=\"arrival\" value=\"2026-09-04\""));
    assert_eq!(html.matches("name=\"arrival\"").count(), 1);
}

/// The opening tags in `html` that carry every one of `attributes`.
fn tags_with<'a>(html: &'a str, attributes: &[&str]) -> Vec<&'a str> {
    html.split('<')
        .map(|tag| tag.split('>').next().unwrap_or_default())
        .filter(|tag| attributes.iter().all(|attribute| tag.contains(attribute)))
        .collect()
}

#[test]
fn a_month_picker_is_one_tab_stop_on_the_picked_month() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                MonthPicker { value: NaiveDate::from_ymd_opt(2026, 9, 1), onchange: move |_| {}, name: "month" }
            }
        }
    }
    let html = body(&render(app));

    assert_eq!(html.matches("data-slot=\"cell\"").count(), 12);
    let stops = tags_with(&html, &["data-slot=\"cell\"", "tabindex=\"0\""]);
    assert_eq!(stops.len(), 1);
    assert!(stops[0].contains("data-date=\"2026-09-01\""));
    assert!(html.contains("name=\"month\" value=\"2026-09-01\""));
}

#[test]
fn a_year_picker_is_one_tab_stop_on_the_picked_year() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                YearPicker { value: NaiveDate::from_ymd_opt(2026, 5, 20), onchange: move |_| {}, name: "year" }
            }
        }
    }
    let html = body(&render(app));

    // 2019 and 2030 border the decade.
    assert_eq!(html.matches("data-slot=\"cell\"").count(), 12);
    assert_eq!(html.matches("data-outside=\"true\"").count(), 2);
    let stops = tags_with(&html, &["data-slot=\"cell\"", "tabindex=\"0\""]);
    assert_eq!(stops.len(), 1);
    assert!(stops[0].contains("data-date=\"2026-01-01\""));
    // Any day of the year reads as the year.
    assert!(html.contains("name=\"year\" value=\"2026-01-01\""));
}

#[test]
fn one_field_takes_its_format_and_posting_from_the_value_type() {
    fn app() -> Element {
        let moment = |day, hour| {
            NaiveDate::from_ymd_opt(2026, 9, day)
                .zip(NaiveTime::from_hms_opt(hour, 0, 0))
                .map(|(day, time)| NaiveDateTime::new(day, time))
        };
        rsx! {
            LiberoProvider {
                DateField::<NaiveDate> { value: NaiveDate::from_ymd_opt(2026, 9, 4), onchange: move |_| {}, name: "day" }
                DateField::<NaiveTime> { value: NaiveTime::from_hms_opt(13, 5, 0), onchange: move |_| {}, name: "time" }
                // `value` goes through `SuperInto`, so it never names `V`: a turbofish does.
                DateField::<DateRange<NaiveDateTime>> {
                    value: moment(14, 9).map(|start| DateRange::new(start, moment(16, 17))),
                    onchange: move |_| {},
                    name: "stay",
                }
            }
        }
    }
    let html = body(&render(app));

    assert!(html.contains("value=\"September 4, 2026\""));
    assert!(html.contains("name=\"day\" value=\"2026-09-04\""));
    assert!(html.contains("value=\"13:05\""));
    assert!(html.contains("name=\"time\" value=\"13:05:00\""));
    assert!(html.contains("value=\"September 14, 2026 09:00 – September 16, 2026 17:00\""));
    // chrono's `Display` would put a space where ISO 8601 puts the `T`.
    assert!(html.contains("name=\"stay\" value=\"2026-09-14T09:00:00/2026-09-16T17:00:00\""));
}

#[test]
fn one_picker_draws_what_the_value_type_calls_for() {
    fn app() -> Element {
        let moment = |day, hour| {
            NaiveDate::from_ymd_opt(2026, 9, day)
                .zip(NaiveTime::from_hms_opt(hour, 0, 0))
                .map(|(day, time)| NaiveDateTime::new(day, time))
        };
        rsx! {
            LiberoProvider {
                DatePicker::<NaiveDate> { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, name: "day", class: "day-picker" }
                DatePicker::<NaiveDate> { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, level: DateLevel::Month, name: "month" }
                DatePicker::<NaiveTime> { value: NaiveTime::from_hms_opt(9, 30, 0), onchange: move |_| {}, name: "time", id: "clock" }
                DatePicker::<NaiveDateTime> { value: moment(14, 9), onchange: move |_| {}, name: "moment", class: "flow" }
                DatePicker::<DateRange<NaiveDate>> {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14).map(|start| DateRange::new(start, NaiveDate::from_ymd_opt(2026, 9, 18))),
                    onchange: move |_| {},
                    name: "stay",
                }
            }
        }
    }
    let html = body(&render(app));

    // A day picker, the day grid of the date-time flow, and two months of range.
    assert_eq!(html.matches("role=\"grid\"").count(), 4);
    assert!(html.contains("name=\"day\" value=\"2026-09-14\""));
    // A month is held as its first day.
    assert!(html.contains("name=\"month\" value=\"2026-09-01\""));
    assert!(html.contains("name=\"time\" value=\"09:30:00\""));
    assert!(html.contains("name=\"moment\" value=\"2026-09-14T09:00:00\""));
    assert!(html.contains("name=\"stay\" value=\"2026-09-14/2026-09-18\""));
    // The caller's attributes reach every kind of root.
    assert!(html.contains("day-picker"));
    assert!(html.contains("id=\"clock\""));
    assert!(html.contains("flow"));
}

thread_local! {
    static LEVEL: Cell<DateLevel> = const { Cell::new(DateLevel::Day) };
}

#[test]
fn a_picker_follows_a_level_change_in_the_same_scope() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DatePicker::<NaiveDate> {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    onchange: move |_| {},
                    level: LEVEL.with(Cell::get),
                }
            }
        }
    }
    LEVEL.with(|level| level.set(DateLevel::Day));
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    assert_eq!(
        body(&dioxus_ssr::render(&dom))
            .matches("data-slot=\"day\"")
            .count(),
        42
    );

    // The scope is reused: its own view state must not keep the days.
    LEVEL.with(|level| level.set(DateLevel::Month));
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate(&mut NoOpMutations);
    let html = body(&dioxus_ssr::render(&dom));
    assert_eq!(html.matches("data-slot=\"day\"").count(), 0);
    assert_eq!(html.matches("data-slot=\"cell\"").count(), 12);
}

#[test]
fn a_mini_calendar_draws_one_row_of_days_and_posts_the_day() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    onchange: move |_| {},
                    calendar: "mini",
                    days: 5,
                    name: "arrival",
                }
            }
        }
    }
    let html = body(&render(app));

    assert_eq!(html.matches("data-slot=\"day\"").count(), 5);
    assert_eq!(html.matches("data-slot=\"month\"").count(), 5);
    assert_eq!(html.matches("role=\"grid\"").count(), 1);
    // The row starts at the value.
    assert!(html.contains("data-date=\"2026-09-14\""));
    assert!(html.contains("data-date=\"2026-09-18\""));
    assert!(!html.contains("data-date=\"2026-09-19\""));
    assert!(html.contains(">Sep<"));
    assert!(html.contains("aria-label=\"Previous days\""));
    assert!(html.contains("aria-label=\"Next days\""));
    assert!(html.contains("name=\"arrival\" value=\"2026-09-14\""));
}

#[test]
fn a_mini_calendar_cannot_page_past_its_limits() {
    fn open() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, calendar: "mini" }
            }
        }
    }
    fn limited() -> Element {
        rsx! {
            LiberoProvider {
                DayPicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    onchange: move |_| {},
                    calendar: "mini",
                    min: NaiveDate::from_ymd_opt(2026, 9, 14),
                    max: NaiveDate::from_ymd_opt(2026, 9, 20),
                }
            }
        }
    }
    let disabled = |app: fn() -> Element| body(&render(app)).matches("disabled=true").count();

    assert_eq!(disabled(open), 0);
    // Both buttons, and no day: the seven days are exactly the limits.
    assert_eq!(disabled(limited), 2);
}

thread_local! {
    static CALENDAR: Cell<CalendarVariant> = const { Cell::new(CalendarVariant::Full) };
}

#[test]
fn a_picker_follows_a_calendar_change_in_the_same_scope() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DatePicker::<NaiveDate> {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    onchange: move |_| {},
                    calendar: CALENDAR.with(Cell::get).as_str(),
                }
            }
        }
    }
    let days = |dom: &VirtualDom| {
        body(&dioxus_ssr::render(dom))
            .matches("data-slot=\"day\"")
            .count()
    };
    CALENDAR.with(|calendar| calendar.set(CalendarVariant::Full));
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    assert_eq!(days(&dom), 42);

    CALENDAR.with(|calendar| calendar.set(CalendarVariant::Mini));
    dom.mark_dirty(ScopeId::APP);
    dom.render_immediate(&mut NoOpMutations);
    assert_eq!(days(&dom), 7);
}

thread_local! {
    static FIRST_EXCLUDED_WEEKDAY: Cell<u32> = const { Cell::new(7) };
}

/// Re-renders `app` in one scope while the rule flips between nothing and the
/// weekends, and counts the disabled days after each pass.
fn disabled_per_flip(app: fn() -> Element) -> Vec<usize> {
    let disabled = |dom: &VirtualDom| body(&dioxus_ssr::render(dom)).matches("disabled").count();
    FIRST_EXCLUDED_WEEKDAY.with(|first| first.set(7));
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let mut counts = vec![disabled(&dom)];
    for pass in 1..=8 {
        FIRST_EXCLUDED_WEEKDAY.with(|first| first.set(if pass % 2 == 1 { 5 } else { 7 }));
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate(&mut NoOpMutations);
        counts.push(disabled(&dom));
    }
    counts
}

#[test]
fn a_picker_follows_a_new_exclude_date_rule_in_the_same_scope() {
    fn app() -> Element {
        // Captured while rendering, as a caller's own state would be: only the
        // closure changes between the two passes.
        let first = FIRST_EXCLUDED_WEEKDAY.with(Cell::get);
        rsx! {
            LiberoProvider {
                DayPicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    onchange: move |_| {},
                    exclude_date: move |day: NaiveDate| day.weekday().num_days_from_monday() >= first,
                }
            }
        }
    }
    assert_eq!(disabled_per_flip(app), [0, 12, 0, 12, 0, 12, 0, 12, 0]);
}
