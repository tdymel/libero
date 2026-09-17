//! `TimePicker`: a pick redraws the options it moved and leaves the rest, on
//! both variants (todo 29 stopped the unchanged columns and marks redrawing).
//!
//! `/time-picker` shows 09:30 on a 24-hour digital picker and an analog one,
//! each echoing its value into `#<variant>-value`.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

const MINUTES: &str = "#digital [data-column='Minutes']";
const HOURS: &str = "#digital [data-column='Hours']";

#[test]
fn a_digital_pick_moves_the_selection() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/time-picker", viewport).await.unwrap();
            let page = &fixture.page;

            // Minute 35 is the eighth option at a 5-minute step.
            click(page, &format!("{MINUTES} [data-index='7']")).await;
            expect(
                page,
                "digital-value",
                "09:35:00",
                &format!("{MINUTES} [data-index='7']"),
                at,
            )
            .await;
            expect_unselected(page, &format!("{MINUTES} [data-index='6']"), at).await;

            click(page, &format!("{HOURS} [data-index='10']")).await;
            expect(
                page,
                "digital-value",
                "10:35:00",
                &format!("{HOURS} [data-index='10']"),
                at,
            )
            .await;
            expect_unselected(page, &format!("{HOURS} [data-index='9']"), at).await;
            // The minutes column kept its pick through the hour change.
            expect(
                page,
                "digital-value",
                "10:35:00",
                &format!("{MINUTES} [data-index='7']"),
                at,
            )
            .await;

            // The keyboard's focus request finds the option by its index.
            let focused = format!("{MINUTES} [data-index='7']");
            page.evaluate(format!("document.querySelector({focused:?}).focus()"))
                .await
                .unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            let check = format!(
                "document.activeElement === document.querySelector(\"{MINUTES} [data-index='8']\")"
            );
            wait::for_js_true(page, &check, "ArrowDown to focus minute 40")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("the digital picks at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

#[test]
fn an_analog_pick_takes_the_hour_then_the_minute() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/time-picker", viewport).await.unwrap();
            let page = &fixture.page;

            // The hour face: 9 picked; 11 moves the hand on to the minutes.
            expect_mark(page, "09:30:00", "9", at).await;
            click_mark(page, "11").await;
            expect_mark(page, "11:30:00", "30", at).await;
            click_mark(page, "45").await;
            expect_mark(page, "11:45:00", "45", at).await;

            fixture
                .console
                .assert_clean(&format!("the analog picks at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// A time picked before any day lands on today, from the clock, when the
/// picker has no `today` prop (todo 26).
#[test]
fn a_time_picked_first_lands_on_today() {
    block_on(async {
        let fixture = Fixture::open("/time-picker/date-time", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(
            "[...document.querySelectorAll('label')].find(label => label.textContent === 'Time').click()",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("!!document.querySelector(\"{HOURS_ANY} [data-index='10']\")"),
            "the clock to replace the calendar",
        )
        .await
        .unwrap();
        click(page, &format!("{HOURS_ANY} [data-index='10']")).await;
        let today = "(() => { const d = new Date(); \
                     return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`; })()";
        wait::for_js_true(
            page,
            &format!(
                "document.getElementById('moment-value').textContent === {today} + ' 10:00:00'"
            ),
            "the time to land on today",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the date-time pick").unwrap();
        fixture.close().await.unwrap();
    });
}

const HOURS_ANY: &str = "[data-column='Hours']";

/// Ctrl/Alt/Meta chords are the browser's, on a column and on the face.
#[test]
fn modifier_chords_go_to_the_browser() {
    block_on(async {
        let arrows = [
            keyboard::ARROW_DOWN,
            keyboard::ARROW_UP,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_LEFT,
        ];

        let fixture = Fixture::open("/time-picker", Viewport::Desktop)
            .await
            .unwrap();
        let option = format!("{MINUTES} [data-index='6']");
        fixture
            .page
            .evaluate(format!("document.querySelector({option:?}).focus()"))
            .await
            .unwrap();
        keyboard::assert_chords_ignored(
            &fixture.page,
            &[arrows[0], arrows[1], keyboard::HOME, keyboard::END],
            "[document.activeElement.dataset.index, document.getElementById('digital-value').textContent]",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();

        let fixture = Fixture::open("/time-picker/analog", Viewport::Desktop)
            .await
            .unwrap();
        fixture
            .page
            .evaluate("document.querySelector('#fine [data-slot=face]').focus()")
            .await
            .unwrap();
        keyboard::assert_chords_ignored(
            &fixture.page,
            &arrows,
            "[document.activeElement.dataset.slot, document.getElementById('fine-value').textContent]",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("time picker chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A one-minute step and seconds (todo 474): the face picks by the pointer's
/// angle, not only at the marks, an hour keeps its minute inside `min`, and a
/// release moves on to the seconds hand.
#[test]
fn the_analog_face_reaches_every_minute_and_second() {
    block_on(async {
        let fixture = Fixture::open("/time-picker/analog", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let value = |value: &'static str, what: &'static str| async move {
            let check = format!("document.getElementById('fine-value').textContent === {value:?}");
            wait::for_js_true(page, &check, what).await.unwrap();
        };
        // The face's label names the hand it shows; each readout button is
        // named by its shown digits, then the same label (2.5.3).
        let showing = |nth: usize, what: &'static str| async move {
            let check = format!(
                "(() => {{ const b = document.querySelectorAll('#fine [data-slot=readout] button')[{nth}]; \
                 return b.getAttribute('aria-label') === b.textContent + ' ' \
                 + document.querySelector('#fine [data-slot=face]').getAttribute('aria-label'); }})()"
            );
            wait::for_js_true(page, &check, what).await.unwrap();
        };

        press_at(page, mark_centre(page, "#fine", "9").await).await;
        value("09:30:00", "hour 9 to keep :30 inside min").await;
        showing(1, "the release to move on to the minutes").await;
        wait::for_js_true(
            page,
            "!!document.querySelector('#fine [data-slot=ticks]')",
            "a tick per minute",
        )
        .await
        .unwrap();

        press_at(page, face_point(page, 37.0 / 60.0).await).await;
        value("09:37:00", "a press between the marks to pick minute 37").await;
        showing(2, "the release to move on to the seconds").await;

        press_at(page, face_point(page, 13.0 / 60.0).await).await;
        value("09:37:13", "a press to pick second 13").await;
        wait::for_js_true(
            page,
            "!!document.querySelector('#fine [data-slot=hand]')",
            "a seconds hand",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        value("09:37:14", "ArrowUp to step a second").await;

        pointer::click(page, "#fine [data-slot=readout] button:nth-of-type(2)")
            .await
            .unwrap();
        showing(1, "the minutes button to show the minutes").await;
        page.evaluate("document.querySelector('#fine [data-slot=face]').focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        value("09:38:14", "ArrowUp to step a minute").await;

        let (from, to) = (
            face_point(page, 40.0 / 60.0).await,
            face_point(page, 50.0 / 60.0).await,
        );
        pointer::drag(page, from, to, 8).await.unwrap();
        value("09:50:14", "a drag to follow the pointer to minute 50").await;

        fixture
            .console
            .assert_clean("the fine analog face")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The face is an APG slider: Home/End go to the hand's first and last open
/// value, Page Up/Down a quarter of the face (todo 533).
#[test]
fn the_face_takes_home_end_and_page_keys() {
    block_on(async {
        let fixture = Fixture::open("/time-picker/analog", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let focus_face =
            || page.evaluate("document.querySelector('#fine [data-slot=face]').focus()");
        let press = |key: keyboard::Key, value: &'static str, what: &'static str| async move {
            keyboard::press(page, key).await.unwrap();
            let check = format!("document.getElementById('fine-value').textContent === {value:?}");
            wait::for_js_true(page, &check, what).await.unwrap();
        };

        focus_face().await.unwrap();
        press(keyboard::END, "23:00:00", "End to the last hour").await;
        press(
            keyboard::HOME,
            "09:30:00",
            "Home to the first hour inside min",
        )
        .await;
        press(keyboard::PAGE_UP, "12:30:00", "PageUp three hours on").await;

        pointer::click(page, "#fine [data-slot=readout] button:nth-of-type(2)")
            .await
            .unwrap();
        focus_face().await.unwrap();
        press(keyboard::PAGE_UP, "12:45:00", "PageUp fifteen minutes on").await;
        press(keyboard::END, "12:59:00", "End to the last minute").await;
        press(keyboard::HOME, "12:00:00", "Home to the first minute").await;

        fixture
            .console
            .assert_clean("the face's page keys")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A press and release at one point.
async fn press_at(page: &chromiumoxide::Page, at: pointer::Point) {
    pointer::drag(page, at, at, 1).await.unwrap();
}

/// The point on `#fine`'s outer ring a `turn` clockwise from 12.
async fn face_point(page: &chromiumoxide::Page, turn: f64) -> pointer::Point {
    page.evaluate(format!(
        "(() => {{ const r = document.querySelector('#fine [data-slot=face]').getBoundingClientRect(); \
         const a = {turn} * 2 * Math.PI; \
         return {{ x: r.x + r.width / 2 + 0.4 * r.width * Math.sin(a), \
                   y: r.y + r.height / 2 - 0.4 * r.height * Math.cos(a) }}; }})()"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// The centre of the mark labelled `label` under `root`.
async fn mark_centre(page: &chromiumoxide::Page, root: &str, label: &str) -> pointer::Point {
    page.evaluate(format!(
        "(() => {{ const r = [...document.querySelectorAll('{root} [data-slot=mark]')] \
         .find(mark => mark.textContent === {label:?}).getBoundingClientRect(); \
         return {{ x: r.x + r.width / 2, y: r.y + r.height / 2 }}; }})()"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

const MARKS: &str = "[...document.querySelectorAll(\"#analog [data-slot='mark']\")]";

async fn click_mark(page: &chromiumoxide::Page, label: &str) {
    press_at(page, mark_centre(page, "#analog", label).await).await;
}

/// Waits until the analog picker reads `value` and `label` is its only picked
/// mark.
async fn expect_mark(page: &chromiumoxide::Page, value: &str, label: &str, at: &str) {
    let check = format!(
        "document.getElementById('analog-value').textContent === {value:?} \
         && {MARKS}.filter(mark => mark.hasAttribute('data-selected')) \
              .map(mark => mark.textContent).join() === {label:?}"
    );
    wait::for_js_true(page, &check, &format!("{value} with mark {label} picked"))
        .await
        .unwrap_or_else(|e| panic!("at {at}: {e}"));
}

async fn click(page: &chromiumoxide::Page, selector: &str) {
    page.evaluate(format!("document.querySelector({selector:?}).click()"))
        .await
        .unwrap();
}

/// Waits until `#id` reads `value` and `selected` is the picked option.
async fn expect(page: &chromiumoxide::Page, id: &str, value: &str, selected: &str, at: &str) {
    let check = format!(
        "document.getElementById({id:?}).textContent === {value:?} \
         && document.querySelector({selected:?}).hasAttribute('data-selected')"
    );
    wait::for_js_true(
        page,
        &check,
        &format!("{id} to read {value} with {selected} picked"),
    )
    .await
    .unwrap_or_else(|e| panic!("at {at}: {e}"));
}

/// Forced colours paint every fill `Canvas`: the picked option and mark, the
/// hand the readout is on, and the hand itself must not vanish (1.4.1, 1.4.11).
#[test]
fn picks_and_the_hand_show_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/time-picker", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        let canvas_filled: Vec<String> = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 return ['#digital [data-selected]', \"#analog [data-slot='mark'][data-selected]\", \
                   \"#analog [data-slot='readout'] [data-active]\", \"#analog [data-slot='hand']\", \
                   \"#analog [data-slot='pivot']\"] \
                   .filter(s => { const e = document.querySelector(s); \
                     return !e || getComputedStyle(e).backgroundColor === canvas; }); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            canvas_filled.is_empty(),
            "painted the page's own colour: {canvas_filled:?}"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 744: the hand the readout sets carries the house on-state ring, not a
/// tint. Todo 745: a disabled mark or option turns `GrayText` in forced colours.
#[test]
fn the_readout_hand_is_ringed_and_disabled_parts_gray_out() {
    use crate::button::{assert_gray_in_forced_colours, assert_on_marker};
    const ACTIVE: &str = r#"#fine [data-slot="readout"] [data-active]"#;
    const IDLE: &str = r#"#fine [data-slot="readout"] button:not([data-active])"#;
    block_on(async {
        let fixture = Fixture::open("/time-picker/analog", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        assert_on_marker(page, ACTIVE, IDLE).await;
        let [active, idle]: [String; 2] = page
            .evaluate(format!(
                "[{ACTIVE:?}, {IDLE:?}].map(q => getComputedStyle(document.querySelector(q)).backgroundColor)"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(active, idle, "the active hand keeps a tint under its ring");

        crate::calendar::force_colours(page).await;
        assert_gray_in_forced_colours(page, r#"#fine [data-slot="mark"][data-disabled]"#).await;
        assert_gray_in_forced_colours(page, r#"#limited [data-column="Hours"] button:disabled"#)
            .await;
        fixture.close().await.unwrap();
    });
}

async fn expect_unselected(page: &chromiumoxide::Page, selector: &str, at: &str) {
    let check = format!("!document.querySelector({selector:?}).hasAttribute('data-selected')");
    wait::for_js_true(page, &check, &format!("{selector} to lose its pick"))
        .await
        .unwrap_or_else(|e| panic!("at {at}: {e}"));
}
