//! The date and time pickers' flows: a time picked before any day (todo 3111). `today` is
//! the 9th and `min` the 10th at 09:00, so any time on `today` lands on `min`.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

const INPUT: &str = "input[data-controlled]";
const SECOND_TAB: &str = "[role=dialog] [role=tablist] [role=tab]:nth-child(2)";
const HOURS: &str = "[role=dialog] [data-column='Hours']";

/// Opens the field, goes to its second tab (the time, or the start's time), and turns the hours.
async fn pick_a_time_first(route: &str) -> Fixture {
    let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
    let page = &fixture.page;

    pointer::click(page, INPUT).await.unwrap();
    wait::for_js_true(page, &selector_exists(SECOND_TAB), "the dropdown's tabs")
        .await
        .unwrap();
    pointer::click(page, SECOND_TAB).await.unwrap();
    wait::for_js_true(page, &selector_exists(HOURS), "the clock's hours")
        .await
        .unwrap();
    page.evaluate(format!("document.querySelector({HOURS:?}).focus()"))
        .await
        .unwrap();
    keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
    fixture
}

fn selector_exists(selector: &str) -> String {
    format!("!!document.querySelector({selector:?})")
}

async fn readout_is(page: &chromiumoxide::Page, text: &str, what: &str) {
    let check = format!("document.querySelector('#readout').textContent.trim() === {text:?}");
    wait::for_js_true(page, &check, what).await.unwrap();
}

/// A time turned on the clock before any day was picked lands on `today`, below `min`: it is
/// pulled up to `min`, not emitted below it.
#[test]
fn a_time_picked_before_any_day_is_pulled_into_min() {
    block_on(async {
        let fixture = pick_a_time_first("/chrono-field/time-first").await;

        readout_is(
            &fixture.page,
            "2026-10-10 09:00:00",
            "the time pulled up to min",
        )
        .await;

        fixture.console.assert_clean("a time before a day").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The same for the start of a range of moments.
#[test]
fn a_start_time_picked_before_any_day_is_pulled_into_min() {
    block_on(async {
        let fixture = pick_a_time_first("/chrono-field/time-first-range").await;

        readout_is(
            &fixture.page,
            "2026-10-10 09:00:00",
            "the start time pulled up to min",
        )
        .await;

        fixture
            .console
            .assert_clean("a start time before a day")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
