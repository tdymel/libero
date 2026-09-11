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
        // The face's label names the hand it shows; the readout buttons carry
        // the same three labels.
        let showing = |nth: usize, what: &'static str| async move {
            let check = format!(
                "document.querySelector('#fine [data-slot=face]').getAttribute('aria-label') \
                 === document.querySelectorAll('#fine [data-slot=readout] button')[{nth}].getAttribute('aria-label')"
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

async fn expect_unselected(page: &chromiumoxide::Page, selector: &str, at: &str) {
    let check = format!("!document.querySelector({selector:?}).hasAttribute('data-selected')");
    wait::for_js_true(page, &check, &format!("{selector} to lose its pick"))
        .await
        .unwrap_or_else(|e| panic!("at {at}: {e}"));
}
