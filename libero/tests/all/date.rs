//! The date and time pickers and fields as a server renders them: the grid,
//! the marked days, and what posts with a form.

use crate::common::{body, render, tags_with};

use std::cell::Cell;
use std::collections::BTreeMap;

use dioxus::dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, Weekday},
    components::{
        ChronoField, ChronoPicker, DateField, DateLevel, DatePicker, DateRange, DateRangePicker,
        MonthPicker, SegmentedControl, TimePicker, YearPicker,
    },
    localization::Formats,
    theme::CalendarVariant,
};

#[test]
fn a_time_picker_draws_a_spinbutton_per_part_and_posts_the_time() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TimePicker { value: NaiveTime::from_hms_opt(9, 30, 0), onchange: move |_| {}, variant: "digital", step: 15, name: "at" }
            }
        }
    }
    let html = body(&render(app));

    // The default en-US clock: hours, minutes at 15 with their unit (todo 894), AM/PM.
    assert_eq!(html.matches("role=\"spinbutton\"").count(), 3);
    for text in ["09", "30 minutes", "AM"] {
        assert!(
            html.contains(&format!("aria-valuetext=\"{text}\"")),
            "{text}: {html}"
        );
    }
    assert!(html.contains("aria-label=\"AM/PM\""), "{html}");
    // Each value sits between its neighbours; the digits wrap round, AM/PM not.
    let neighbours: Vec<&str> = html
        .split("data-slot=\"neighbour\"")
        .skip(1)
        .filter_map(|after| after.split('>').nth(1)?.split('<').next())
        .collect();
    assert_eq!(neighbours, ["08", "10", "15", "45", "", "PM"]);
    assert!(html.contains("type=\"hidden\" name=\"at\" value=\"09:30:00\""));
}

/// Todo 894: the digital seconds say their unit, as the analog face does.
#[test]
fn a_time_pickers_seconds_say_their_unit() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TimePicker { value: NaiveTime::from_hms_opt(14, 1, 1), onchange: move |_| {}, variant: "digital", with_seconds: true }
            }
        }
    }
    let html = body(&render(app));
    for text in ["1 minute", "1 second"] {
        assert!(
            html.contains(&format!("aria-valuetext=\"{text}\"")),
            "{text}: {html}"
        );
    }
}

#[test]
fn the_analog_face_reads_an_hour_with_its_half() {
    fn app() -> Element {
        let value = NaiveTime::from_hms_opt(15, 20, 0);
        rsx! {
            LiberoProvider {
                TimePicker { value, onchange: move |_| {}, variant: "analog", twelve_hour: true }
                TimePicker { value, onchange: move |_| {}, variant: "analog", twelve_hour: false }
            }
        }
    }
    let html = body(&render(app));

    for text in ["3 PM", "15"] {
        assert!(
            html.contains(&format!("aria-valuetext=\"{text}\"")),
            "{text}: {html}"
        );
    }
}

#[test]
fn a_half_of_the_day_outside_min_and_max_is_disabled_on_both_faces() {
    fn app() -> Element {
        let (value, min) = (
            NaiveTime::from_hms_opt(15, 20, 0),
            NaiveTime::from_hms_opt(13, 0, 0),
        );
        rsx! {
            LiberoProvider {
                TimePicker { value, min, onchange: move |_| {}, variant: "analog", twelve_hour: true }
                TimePicker { value, min, onchange: move |_| {}, variant: "digital", twelve_hour: true }
            }
        }
    }
    let html = body(&render(app));

    // The tag of every button whose text starts with the label.
    let tags = |label: &str| -> Vec<String> {
        let end = format!(">{label}</button>");
        html.split(&end)
            .filter_map(|before| before.rsplit("<button").next())
            .take(html.matches(&end).count())
            .map(str::to_owned)
            .collect()
    };
    let (am, pm) = (tags("AM"), tags("PM"));
    assert_eq!((am.len(), pm.len()), (1, 1));
    assert!(am.iter().all(|tag| tag.contains("disabled")), "{am:?}");
    assert!(pm.iter().all(|tag| !tag.contains("disabled")), "{pm:?}");
    // The digital AM/PM column has no AM to step to: no neighbour shows it.
    assert!(html.contains("aria-valuetext=\"PM\""), "{html}");
    assert!(!html.contains(">AM</span>"), "{html}");
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
fn a_day_button_is_named_by_its_full_date() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DatePicker { value: NaiveDate::from_ymd_opt(2026, 9, 18), onchange: move |_| {} }
            }
        }
    }
    let html = body(&render(app));

    assert_eq!(html.matches("data-slot=\"day\"").count(), 42);
    // Every day, a neighbour's too, carries its full date; the text stays the number.
    assert_eq!(
        html.matches("data-slot=\"day\"").count(),
        html.matches(", 2026\"").count()
    );
    let at = html.find("data-date=\"2026-09-18\"").expect("the 18th");
    let button = &html[at..at + html[at..].find("</button>").expect("the button ends")];
    assert!(button.contains("aria-label=\"September 18, 2026\""));
    assert!(button.ends_with(">18"));
    assert!(html.contains("aria-label=\"August 31, 2026\""));
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
                DatePicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, name: "day" }
            }
        }
    }
    let html = body(&render(app));

    assert_eq!(html.matches("data-slot=\"day\"").count(), 42);
    // September 2026 starts on a Tuesday, so the default Sunday-first grid
    // opens on August 30 and ends on October 10.
    let first = html.find("data-date=\"2026-08-30\"").expect("Sunday first");
    assert!(first < html.find("data-date=\"2026-09-01\"").expect("the 1st"));
    assert!(html.contains("data-date=\"2026-10-10\""));
    assert!(!html.contains("data-date=\"2026-10-11\""));
    assert!(html.contains("September 2026"));

    assert_eq!(html.matches("data-selected=\"true\"").count(), 1);
    assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
    // The picked day is the grid's one tab stop, beside the heading and the two page buttons.
    assert_eq!(html.matches("tabindex=\"0\"").count(), 4);
    assert!(html.contains("name=\"day\""));
    assert!(html.contains("value=\"2026-09-14\""));
}

/// The weekday arrays are Sunday first whatever `first_weekday` says; the
/// grid and its headers rotate to it.
#[test]
fn a_monday_first_locale_starts_the_grid_and_the_headers_on_monday() {
    static MONDAY: Formats = Formats {
        first_weekday: Weekday::Mon,
        ..Formats::AMERICAN
    };
    fn app() -> Element {
        rsx! {
            LiberoProvider { formats: &MONDAY,
                DatePicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {} }
            }
        }
    }
    let html = body(&render(app));

    let headers = tags_with(&html, r#"role="columnheader""#);
    assert_eq!(headers.len(), 7);
    assert_eq!(headers[0]["aria-label"], "Monday", "{headers:?}");
    assert_eq!(headers[6]["aria-label"], "Sunday", "{headers:?}");
    // September 2026 starts on a Tuesday: the grid opens on Monday, August 31.
    let first = html.find("data-slot=\"day\"").expect("a day");
    assert_eq!(
        html[first..].find("data-date=\"2026-08-31\""),
        html[first..].find("data-date=")
    );
}

#[test]
fn days_outside_min_and_max_are_disabled() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DatePicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    min: NaiveDate::from_ymd_opt(2026, 9, 10),
                    max: NaiveDate::from_ymd_opt(2026, 9, 20),
                    onchange: move |_| {},
                }
            }
        }
    }
    let html = body(&render(app));

    let disabled = |date: &str| {
        let cells = tags_with(&html, &format!(r#"data-date="{date}""#));
        assert_eq!(cells.len(), 1, "{date}: {cells:?}");
        cells[0].contains_key("disabled")
    };
    assert!(disabled("2026-09-09"));
    assert!(!disabled("2026-09-10"));
    assert!(!disabled("2026-09-20"));
    assert!(disabled("2026-09-21"));
}

/// The `data-date` of the one `tabindex="0"` cell of `slot` in `html`.
fn tab_stop(html: &str, slot: &str) -> String {
    let stops: Vec<_> = tags_with(html, &format!(r#"data-slot="{slot}""#))
        .into_iter()
        .filter(|cell| cell.get("tabindex").is_some_and(|index| index == "0"))
        .collect();
    assert_eq!(stops.len(), 1, "{stops:?}");
    stops[0]["data-date"].clone()
}

/// A disabled day cannot take focus, so it never holds the grid's tab stop.
#[test]
fn the_tab_stop_skips_disabled_days() {
    fn min_after_the_first() -> Element {
        rsx! {
            LiberoProvider {
                DatePicker {
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
                DatePicker {
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
                DatePicker {
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
                DatePicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 19),
                    exclude_date: move |day: NaiveDate| day.weekday().num_days_from_monday() >= 5,
                    calendar: "mini",
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = body(&render(min_after_the_first));
    assert_eq!(tab_stop(&html, "day"), "2026-09-10");
    let html = body(&render(weekend_today));
    assert_eq!(tab_stop(&html, "day"), "2026-09-21");
    // Not onto a day of the next month: back to the Friday.
    let html = body(&render(weekend_at_the_month_end));
    assert_eq!(tab_stop(&html, "day"), "2026-05-29");
    let html = body(&render(mini_weekend));
    assert_eq!(tab_stop(&html, "day"), "2026-09-21");
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

    assert_eq!(tab_stop(&html, "cell"), "2026-04-01");
}

/// Todo 534: today is exposed, not only drawn - on the day, the month and the year.
#[test]
fn today_is_the_one_cell_with_aria_current_date() {
    fn day() -> Element {
        rsx! {
            LiberoProvider {
                DatePicker {
                    value: None,
                    today: NaiveDate::from_ymd_opt(2026, 3, 18),
                    onchange: move |_| {},
                }
            }
        }
    }
    fn month() -> Element {
        rsx! {
            LiberoProvider {
                MonthPicker {
                    value: None,
                    today: NaiveDate::from_ymd_opt(2026, 3, 18),
                    onchange: move |_| {},
                }
            }
        }
    }
    fn year() -> Element {
        rsx! {
            LiberoProvider {
                YearPicker {
                    value: None,
                    today: NaiveDate::from_ymd_opt(2026, 3, 18),
                    onchange: move |_| {},
                }
            }
        }
    }

    for (app, date) in [
        (day as fn() -> Element, "2026-03-18"),
        (month, "2026-03-01"),
        (year, "2026-01-01"),
    ] {
        let html = body(&render(app));
        let current = tags_with(&html, r#"aria-current="date""#);
        assert_eq!(current.len(), 1, "{html}");
        assert_eq!(current[0]["data-date"], date, "{current:?}");
        assert_eq!(current[0]["data-today"], "true", "{current:?}");
    }
}

#[test]
fn a_field_shows_its_format_and_posts_iso() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DateField { value: NaiveDate::from_ymd_opt(2026, 9, 4), onchange: move |_| {}, name: "arrival" }
                DateField {
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

/// APG Date Picker Combobox: the input is a combobox opening a dialog, and
/// points at it only while the dialog exists. The browser pass opens it.
#[test]
fn a_closed_field_is_a_combobox_over_a_dialog() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DateField { id: "due", value: NaiveDate::from_ymd_opt(2026, 9, 4), onchange: move |_| {} }
            }
        }
    }
    let html = body(&render(app));
    let input = tags_with(&html, r#"id="due""#);

    assert_eq!(input.len(), 1, "{html}");
    assert_eq!(input[0]["role"], "combobox", "{html}");
    assert_eq!(input[0]["aria-haspopup"], "dialog", "{html}");
    assert_eq!(input[0]["aria-expanded"], "false", "{html}");
    assert!(!input[0].contains_key("aria-controls"), "{html}");
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
    assert_eq!(tab_stop(&html, "cell"), "2026-09-01");
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
    assert_eq!(tab_stop(&html, "cell"), "2026-01-01");
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
                ChronoField::<NaiveDate> { value: NaiveDate::from_ymd_opt(2026, 9, 4), onchange: move |_| {}, name: "day" }
                ChronoField::<NaiveTime> { value: NaiveTime::from_hms_opt(13, 5, 0), onchange: move |_| {}, name: "time" }
                // `value` goes through `SuperInto`, so it never names `V`: a turbofish does.
                ChronoField::<DateRange<NaiveDateTime>> {
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
    assert!(html.contains("value=\"1:05 PM\""));
    assert!(html.contains("name=\"time\" value=\"13:05:00\""));
    assert!(html.contains("value=\"September 14, 2026 9:00 AM – September 16, 2026 5:00 PM\""));
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
                ChronoPicker::<NaiveDate> { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, name: "day", class: "day-picker" }
                ChronoPicker::<NaiveDate> { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, level: DateLevel::Month, name: "month" }
                ChronoPicker::<NaiveTime> { value: NaiveTime::from_hms_opt(9, 30, 0), onchange: move |_| {}, name: "time", id: "clock" }
                ChronoPicker::<NaiveDateTime> { value: moment(14, 9), onchange: move |_| {}, name: "moment", class: "flow" }
                ChronoPicker::<DateRange<NaiveDate>> {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14).map(|start| DateRange::new(start, NaiveDate::from_ymd_opt(2026, 9, 18))),
                    onchange: move |_| {},
                    name: "stay",
                }
            }
        }
    }
    let html = body(&render(app));

    // A day picker, a month picker, the day grid of the date-time flow, and
    // two months of range.
    assert_eq!(html.matches("role=\"grid\"").count(), 5);
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

#[test]
fn a_duration_shows_its_units_and_posts_iso_8601() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ChronoField::<TimeDelta> { value: TimeDelta::try_minutes(90), onchange: move |_| {}, name: "length",
                    min: TimeDelta::try_minutes(15), max: TimeDelta::try_hours(12), step: 15, with_seconds: true, label: "Length" }
                ChronoPicker::<TimeDelta> {
                    value: TimeDelta::try_seconds(2 * 3600 + 15 * 60 + 30),
                    onchange: move |_| {},
                    with_seconds: true,
                    step: 15,
                    max: TimeDelta::try_hours(12),
                    name: "rest",
                }
            }
        }
    }
    let html = body(&render(app));

    assert!(html.contains("value=\"1 h 30 min\""));
    assert!(html.contains("name=\"length\" value=\"PT1H30M\""));
    assert!(html.contains("name=\"rest\" value=\"PT2H15M30S\""));
    // A spinbutton per part, the hours up to `max`, the minutes at the step.
    assert_eq!(html.matches("role=\"spinbutton\"").count(), 3);
    for text in ["2 hours", "15 minutes", "30 seconds"] {
        assert!(
            html.contains(&format!("aria-valuetext=\"{text}\"")),
            "{text}: {html}"
        );
    }
    assert!(html.contains("aria-valuemax=12>"), "{html}");
    assert!(html.contains("aria-valuemax=45>"), "{html}");
    assert!(html.contains(">min</span>"), "{html}");
}

thread_local! {
    static LEVEL: Cell<DateLevel> = const { Cell::new(DateLevel::Day) };
}

#[test]
fn a_picker_follows_a_level_change_in_the_same_scope() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ChronoPicker::<NaiveDate> {
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
                DatePicker {
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
                DatePicker { value: NaiveDate::from_ymd_opt(2026, 9, 14), onchange: move |_| {}, calendar: "mini" }
            }
        }
    }
    fn limited() -> Element {
        rsx! {
            LiberoProvider {
                DatePicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    onchange: move |_| {},
                    calendar: "mini",
                    min: NaiveDate::from_ymd_opt(2026, 9, 14),
                    max: NaiveDate::from_ymd_opt(2026, 9, 20),
                }
            }
        }
    }
    let disabled = |app: fn() -> Element| -> Vec<String> {
        disabled_tags(&body(&render(app)))
            .iter()
            .map(|tag| tag.get("aria-label").cloned().unwrap_or_default())
            .collect()
    };

    assert_eq!(disabled(open), Vec::<String>::new());
    // Both buttons, and no day: the seven days are exactly the limits.
    assert_eq!(disabled(limited), ["Previous days", "Next days"]);
}

/// Every tag carrying the `disabled` attribute, not `aria-disabled`.
fn disabled_tags(html: &str) -> Vec<BTreeMap<String, String>> {
    tags_with(html, " disabled")
        .into_iter()
        .filter(|tag| tag.contains_key("disabled"))
        .collect()
}

thread_local! {
    static CALENDAR: Cell<CalendarVariant> = const { Cell::new(CalendarVariant::Full) };
}

#[test]
fn a_picker_follows_a_calendar_change_in_the_same_scope() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ChronoPicker::<NaiveDate> {
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
    let disabled = |dom: &VirtualDom| disabled_tags(&body(&dioxus_ssr::render(dom))).len();
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
                DatePicker {
                    value: NaiveDate::from_ymd_opt(2026, 9, 14),
                    onchange: move |_| {},
                    exclude_date: move |day: NaiveDate| day.weekday().num_days_from_monday() >= first,
                }
            }
        }
    }
    assert_eq!(disabled_per_flip(app), [0, 12, 0, 12, 0, 12, 0, 12, 0]);
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::body;
    use crate::dispatch::*;
    use dioxus::core::{AttributeValue, ElementId, WriteMutations};

    use dioxus::prelude::*;
    use libero::{LiberoProvider, chrono::NaiveTime, components::TimePicker};

    /// AM from a 14:15 with `min` 02:30 lands on 02:30, not on 02:15 below the
    /// limit (todo 445).
    #[test]
    fn a_time_pickers_am_button_clamps_to_min() {
        fn app() -> Element {
            let mut value = use_signal(|| NaiveTime::from_hms_opt(14, 15, 0));
            rsx! {
                LiberoProvider {
                    TimePicker {
                        value: value(),
                        min: NaiveTime::from_hms_opt(2, 30, 0),
                        onchange: move |next| value.set(next),
                        variant: "analog",
                        twelve_hour: true,
                        name: "at",
                    }
                }
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let am = find.element("click", "text", "AM");

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), am);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        let html = body(&dioxus_ssr::render(&dom));
        assert!(html.contains("name=\"at\" value=\"02:30:00\""), "{html}");
    }

    /// Todo 26 item 5: a range of days closes its dropdown on the second pick,
    /// not the first, and `close_on_change: false` keeps it open.
    mod range_close_on_change {
        use super::*;
        use chrono::NaiveDate;
        use libero::components::{ChronoField, DateRange};
        use std::collections::HashMap;

        thread_local! {
            static CLOSE: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
        }

        /// The input's click listener comes first, after the frame's; every day
        /// button carries its date in `data-date`.
        #[derive(Default)]
        struct FindDays {
            last: Option<ElementId>,
            first_click: Option<ElementId>,
            days: HashMap<String, ElementId>,
            frame: Option<ElementId>,
        }

        impl WriteMutations for FindDays {
            fn push_id(&mut self, id: ElementId) {
                self.last = Some(id);
            }
            fn set_id(&mut self, id: ElementId) {
                self.last = Some(id);
            }
            fn add_event_listener(&mut self, name: &str) {
                if name == "click" && self.last != self.frame {
                    self.first_click = self.first_click.or(self.last);
                }
            }
            fn set_attribute(&mut self, name: &str, _ns: Option<&str>, value: &AttributeValue) {
                if name == "data-frame" {
                    self.frame = self.last;
                }
                if name == "data-date"
                    && let (AttributeValue::Text(day), Some(id)) = (value, self.last)
                {
                    self.days.insert(day.clone(), id);
                }
            }
            fn child(&mut self, _index: usize) {}
            fn pop(&mut self) {}
            fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
            fn create_text(&mut self, _value: &str) {}
            fn clone(&mut self) {}
            fn append_children(&mut self, _m: usize) {}
            fn replace_with(&mut self, _m: usize) {}
            fn insert_after(&mut self, _m: usize) {}
            fn insert_before(&mut self, _m: usize) {}
            fn set_text(&mut self, _value: &str) {}
            fn remove_event_listener(&mut self, _name: &str) {}
            fn remove(&mut self) {}
        }

        fn trip() -> Element {
            let mut value = use_signal(|| None::<DateRange<NaiveDate>>);
            rsx! {
                LiberoProvider {
                    ChronoField::<DateRange<NaiveDate>> {
                        label: "Trip",
                        today: NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
                        close_on_change: CLOSE.with(|cell| cell.get()),
                        value: value(),
                        onchange: move |next| value.set(next),
                    }
                }
            }
        }

        fn open(dom: &VirtualDom) -> bool {
            body(&dioxus_ssr::render(dom)).contains(r#"role="dialog""#)
        }

        fn click(dom: &mut VirtualDom, find: &mut FindDays, id: ElementId) {
            dom.runtime()
                .handle_event("click", Event::new(click_event(), true), id);
            dom.render_immediate(find);
            dom.render_immediate(find);
        }

        fn after_two_picks(close: bool) -> (bool, bool) {
            dioxus::html::set_event_converter(Box::new(TestConverter));
            CLOSE.with(|cell| cell.set(close));
            let mut dom = VirtualDom::new(trip);
            let mut find = FindDays::default();
            dom.rebuild(&mut find);
            dom.render_immediate(&mut find);
            let input = find.first_click.expect("the input takes clicks");
            click(&mut dom, &mut find, input);
            assert!(open(&dom), "a click opens the dropdown");

            let day = |find: &FindDays, day: &str| *find.days.get(day).expect(day);
            let first = day(&find, "2026-09-10");
            click(&mut dom, &mut find, first);
            let after_first = open(&dom);
            let second = day(&find, "2026-09-14");
            click(&mut dom, &mut find, second);
            (after_first, open(&dom))
        }

        #[test]
        fn a_range_of_days_closes_on_its_second_pick() {
            assert_eq!(after_two_picks(true), (true, false));
        }

        #[test]
        fn close_on_change_false_keeps_a_range_open() {
            assert_eq!(after_two_picks(false), (true, true));
        }
    }
}
