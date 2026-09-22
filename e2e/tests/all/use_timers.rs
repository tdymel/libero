//! `use_timeout`, `use_interval` and the debounce hooks: typing into a field
//! with a debounced value, and an interval that starts and stops.

use std::time::{Duration, Instant};

use anyhow::Result;
use e2e::browser::block_on;
use e2e::clock::HELD_CLOCK;
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

async fn text_is<D: Driver>(d: &mut D, selector: &str, expected: &str) -> Result<()> {
    eventually(d, &format!("{selector} to read {expected:?}"), async |d| {
        Ok(d.text(selector).await? == expected)
    })
    .await
}

async fn settles_after_typing<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#field").await?;
    d.type_text("abc").await?;

    text_is(d, "#typed", "abc").await?;
    text_is(d, "#settled", "abc").await?;
    text_is(d, "#throttled", "abc").await?;
    text_is(d, "#saved", "abc").await
}

async fn ticks<D: Driver>(d: &mut D) -> Result<u32> {
    Ok(d.text("#ticks").await?.trim().parse()?)
}

async fn interval_starts_and_stops<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    assert_eq!(ticks(d).await?, 0, "ticked before start");

    d.click("#toggle").await?;
    eventually(d, "two ticks", async |d| Ok(ticks(d).await? >= 2)).await?;
    text_is(d, "#toggle", "Stop").await?;

    d.click("#toggle").await?;
    text_is(d, "#toggle", "Start").await?;
    // One tick may already be in flight.
    let stopped = Instant::now();
    while stopped.elapsed() < Duration::from_millis(150) {
        d.idle().await;
    }
    let at_stop = ticks(d).await?;
    let quiet = Instant::now();
    while quiet.elapsed() < Duration::from_millis(400) {
        d.idle().await;
    }
    assert_eq!(ticks(d).await?, at_stop, "kept ticking after stop");
    Ok(())
}

async fn timeout_fires_once<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    text_is(d, "#flashing", "off").await?;
    d.click("#flash").await?;
    text_is(d, "#flashing", "on").await?;
    text_is(d, "#flashing", "off").await
}

e2e::scenario!(
    typing_settles_a_debounced_value_and_callback,
    "/use-timers/quick",
    settles_after_typing
);
e2e::scenario!(
    an_interval_ticks_until_it_is_stopped,
    "/use-timers/interval",
    interval_starts_and_stops
);
e2e::scenario!(
    a_timeout_ends_what_it_started,
    "/use-timers/interval",
    timeout_fires_once
);

const DEBOUNCE_MS: u32 = 717;
const SAVE_MS: u32 = 727;
const THROTTLE_MS: u32 = 737;

async fn js<T: serde::de::DeserializeOwned>(page: &chromiumoxide::Page, expression: &str) -> T {
    page.evaluate(expression)
        .await
        .unwrap_or_else(|e| panic!("evaluate {expression}: {e}"))
        .into_value()
        .unwrap_or_else(|e| panic!("read {expression}: {e}"))
}

async fn text(page: &chromiumoxide::Page, id: &str) -> String {
    js(
        page,
        &format!("document.getElementById('{id}').textContent"),
    )
    .await
}

async fn armed(page: &chromiumoxide::Page, ms: u32, what: &str) {
    wait::for_js_true(page, &format!("window.__heldClock.armed({ms}) === 1"), what)
        .await
        .unwrap();
}

async fn fire(page: &chromiumoxide::Page, ms: u32) {
    let fired: usize = js(page, &format!("window.__heldClock.fire({ms})")).await;
    assert_eq!(fired, 1, "fired {fired} timers of {ms}ms");
}

async fn reads(page: &chromiumoxide::Page, id: &str, expected: &str) {
    wait::for_js_true(
        page,
        &format!("document.getElementById('{id}').textContent === '{expected}'"),
        &format!("#{id} to read {expected}"),
    )
    .await
    .unwrap();
}

/// Each delay sits on a timer the test holds: nothing follows the typing until
/// it fires, the whole burst is one update, and a throttle shows the first key.
#[test]
fn the_delays_hold_until_their_timers_fire() {
    block_on(async {
        let fixture = Fixture::open("/use-timers/held", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let _: bool = js(
            page,
            &format!("(({HELD_CLOCK})([{DEBOUNCE_MS}, {SAVE_MS}, {THROTTLE_MS}]), true)"),
        )
        .await;
        let _: bool = js(page, "(document.getElementById('field').focus(), true)").await;
        keyboard::type_text(page, "abc").await.unwrap();

        reads(page, "typed", "abc").await;
        armed(page, DEBOUNCE_MS, "typing to arm the debounce").await;
        armed(page, SAVE_MS, "typing to arm the debounced callback").await;
        // The control on the clock: were a delay not held, the real one would
        // have settled the value while this sleeps.
        tokio::time::sleep(Duration::from_millis(u64::from(DEBOUNCE_MS) + 300)).await;
        assert_eq!(text(page, "settled").await, "", "settled before its timer");
        assert_eq!(text(page, "saved").await, "", "saved before its timer");
        assert!(
            !text(page, "throttled").await.is_empty(),
            "the leading key waited"
        );

        fire(page, DEBOUNCE_MS).await;
        reads(page, "settled", "abc").await;
        assert_eq!(text(page, "saved").await, "");
        fire(page, SAVE_MS).await;
        reads(page, "saved", "abc").await;
        fire(page, THROTTLE_MS).await;
        reads(page, "throttled", "abc").await;

        fixture
            .console
            .assert_clean("typing into a debounced field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
