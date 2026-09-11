//! Dates and times: `DateField`, `DatePicker`, `TimeField`, `TimePicker`.
//! `today` is pinned wherever a grid is drawn.

use blitz_traits::events::{
    BlitzPointerEvent, BlitzPointerId, MouseEventButton, MouseEventButtons, Point, PointerCoords,
    PointerDetails, UiEvent,
};
use dioxus::prelude::*;
use libero::{
    chrono::{NaiveDate, NaiveTime},
    components::{DateField, DateLevel, DatePicker, TimeField, TimePicker},
};
use native_tests::{Key, Modifiers, Page, mount};

const INPUT: &str = "input[data-controlled]";
const DIALOG: &str = "[role=dialog]";

fn typing(page: &mut Page, text: &str) {
    for c in text.chars() {
        page.press(Key::Character(c.to_string()));
    }
}

/// Empties the focused text input: Blitz has no select-all shortcut here.
fn clear(page: &mut Page) {
    page.press(Key::End);
    for _ in 0..40 {
        page.press(Key::Backspace);
    }
}

fn day(y: i32, m: u32, d: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(y, m, d)
}

fn date_field() -> Element {
    let mut value = use_signal(|| day(2026, 9, 25));
    rsx! {
        DateField::<NaiveDate> {
            label: "Arrival",
            today: day(2026, 9, 18),
            value: value(),
            onchange: move |next| value.set(next),
        }
        span { id: "echo", {value().map(|d| d.to_string()).unwrap_or_default()} }
    }
}

#[test]
fn a_typed_date_commits_on_enter() {
    let mut page = mount(date_field);
    page.click(INPUT);
    clear(&mut page);
    typing(&mut page, "October 3, 2026");
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "2026-10-03", "{}", page.tree());
}

#[test]
fn a_click_opens_the_date_dialog_and_leaves_focus_in_the_input() {
    let mut page = mount(date_field);
    page.click(INPUT);
    assert!(page.exists(DIALOG), "{}", page.tree());
    assert!(page.is_focused(INPUT), "{}", page.focus_owner());
}

/// On the web focus alone opens it; Tab fires no `focus` natively.
#[test]
#[ignore = "N6 (focus events): Tab fires no focus event, so the dialog does not open"]
fn tab_opens_the_date_dialog() {
    let mut page = mount(date_field);
    page.tab();
    assert!(page.exists(DIALOG), "{}", page.tree());
}

#[test]
fn the_keys_pick_a_day_in_the_date_dialog() {
    let mut page = mount(date_field);
    page.click(INPUT);
    page.press(Key::ArrowDown);
    assert!(
        page.is_focused("[data-date='2026-09-25']"),
        "Arrow Down focuses the held day, not {}",
        page.focus_owner()
    );
    page.press(Key::ArrowRight);
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "2026-09-26", "{}", page.tree());
    assert!(!page.exists(DIALOG), "{}", page.tree());
    assert!(page.is_focused(INPUT), "{}", page.focus_owner());
}

#[test]
fn escape_closes_the_date_dialog_and_returns_to_the_input() {
    let mut page = mount(date_field);
    page.click(INPUT);
    page.press(Key::ArrowDown);
    page.press(Key::Escape);
    assert_eq!(
        page.attr(INPUT, "aria-expanded").as_deref(),
        Some("false"),
        "{}",
        page.tree()
    );
    assert!(page.is_focused(INPUT), "{}", page.focus_owner());
}

#[test]
fn a_click_on_a_day_in_the_date_dialog_picks_it() {
    let mut page = mount(date_field);
    page.click(INPUT);
    page.click("[data-date='2026-09-10']");
    assert_eq!(page.text("#echo"), "2026-09-10", "{}", page.tree());
}

fn month_field() -> Element {
    let mut value = use_signal(|| day(2026, 3, 1));
    rsx! {
        DateField::<NaiveDate> {
            label: "Billing month",
            level: DateLevel::Month,
            today: day(2026, 3, 18),
            value: value(),
            onchange: move |next| value.set(next),
        }
        span { id: "echo", {value().map(|d| d.to_string()).unwrap_or_default()} }
    }
}

#[test]
fn the_keys_pick_a_month_in_a_month_field() {
    let mut page = mount(month_field);
    page.click(INPUT);
    page.press(Key::ArrowDown);
    assert!(
        page.is_focused("[data-date='2026-03-01']"),
        "{}",
        page.focus_owner()
    );
    page.press(Key::ArrowRight);
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "2026-04-01", "{}", page.tree());
}

fn picker() -> Element {
    let mut value = use_signal(|| day(2026, 9, 25));
    rsx! {
        DatePicker::<NaiveDate> {
            today: day(2026, 9, 18),
            value: value(),
            onchange: move |next| value.set(next),
        }
        span { id: "echo", {value().map(|d| d.to_string()).unwrap_or_default()} }
    }
}

#[test]
fn a_click_on_a_day_picks_it() {
    let mut page = mount(picker);
    page.click("[data-date='2026-09-03']");
    assert_eq!(page.text("#echo"), "2026-09-03", "{}", page.tree());
}

#[test]
fn the_arrows_walk_the_days_and_enter_picks() {
    let mut page = mount(picker);
    page.focus("[data-date='2026-09-25']");
    page.press(Key::ArrowDown);
    assert!(
        page.is_focused("[data-date='2026-10-02']"),
        "{}",
        page.focus_owner()
    );
    page.press(Key::ArrowLeft);
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "2026-10-01", "{}", page.tree());
}

fn time_field() -> Element {
    let mut value = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    rsx! {
        TimeField {
            label: "Start",
            value: value(),
            onchange: move |next| value.set(next),
        }
        span { id: "echo", {value().map(|t| t.to_string()).unwrap_or_default()} }
    }
}

#[test]
fn a_typed_time_commits_on_enter() {
    let mut page = mount(time_field);
    page.click(INPUT);
    clear(&mut page);
    typing(&mut page, "14:45");
    page.press(Key::Enter);
    assert_eq!(page.text("#echo"), "14:45:00", "{}", page.tree());
}

const MINUTES: &str = "[data-column='Minutes']";
const HOURS: &str = "[data-column='Hours']";

fn digital() -> Element {
    let mut value = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    rsx! {
        TimePicker {
            variant: "digital",
            twelve_hour: false,
            step: 5,
            value: value(),
            onchange: move |next: Option<NaiveTime>| value.set(next),
        }
        span { id: "echo", {value().map(|t| t.to_string()).unwrap_or_default()} }
    }
}

#[test]
fn a_click_on_a_digital_option_picks_it() {
    let mut page = mount(digital);
    page.click(&format!("{MINUTES} [data-index='7']"));
    assert_eq!(page.text("#echo"), "09:35:00", "{}", page.tree());
    page.click(&format!("{HOURS} [data-index='10']"));
    assert_eq!(page.text("#echo"), "10:35:00", "{}", page.tree());
}

/// Tab lands on each column's picked option; the keys move from there.
#[test]
fn arrow_down_moves_focus_down_a_digital_column() {
    let mut page = mount(digital);
    page.tab();
    page.tab();
    assert!(
        page.is_focused(&format!("{MINUTES} [data-index='6']")),
        "{}",
        page.focus_owner()
    );
    page.press(Key::ArrowDown);
    assert!(
        page.is_focused(&format!("{MINUTES} [data-index='7']")),
        "{}",
        page.focus_owner()
    );
}

fn analog() -> Element {
    let mut value = use_signal(|| NaiveTime::from_hms_opt(9, 30, 0));
    rsx! {
        TimePicker {
            variant: "analog",
            twelve_hour: false,
            step: 5,
            value: value(),
            onchange: move |next: Option<NaiveTime>| value.set(next),
        }
        span { id: "echo", {value().map(|t| t.to_string()).unwrap_or_default()} }
    }
}

/// Clicks the mark whose text is `label`. A mark is centred by
/// `translate(-50%, -50%)`, which Blitz's client rect ignores: the drawn
/// mark's centre is the rect's top-left corner.
fn click_mark(page: &mut Page, label: &str) {
    let id = page
        .query_all("[data-slot='mark']")
        .into_iter()
        .find(|id| {
            page.doc
                .inner
                .borrow()
                .get_node(*id)
                .is_some_and(|node| node.text_content() == label)
        })
        .unwrap_or_else(|| panic!("no mark {label}\n{}", page.tree()));
    let rect = page.doc.inner.borrow().get_client_bounding_rect(id);
    let rect = rect.expect("a layout box");
    let at = |down| {
        let (x, y) = (rect.x as f32, rect.y as f32);
        BlitzPointerEvent {
            id: BlitzPointerId::Mouse,
            is_primary: true,
            coords: PointerCoords {
                page_x: x,
                page_y: y,
                screen_x: x,
                screen_y: y,
                client_x: x,
                client_y: y,
            },
            button: MouseEventButton::Main,
            buttons: if down {
                MouseEventButtons::Primary
            } else {
                MouseEventButtons::empty()
            },
            mods: Modifiers::empty(),
            details: PointerDetails::default(),
            element: Point { x: 0.0, y: 0.0 },
            active_pointers: Default::default(),
        }
    };
    page.dispatch(UiEvent::PointerDown(at(true)));
    page.dispatch(UiEvent::PointerUp(at(false)));
}

#[test]
fn an_analog_click_takes_the_hour_then_the_minute() {
    let mut page = mount(analog);
    click_mark(&mut page, "11");
    assert_eq!(page.text("#echo"), "11:30:00", "{}", page.tree());
    click_mark(&mut page, "45");
    assert_eq!(page.text("#echo"), "11:45:00", "{}", page.tree());
}
