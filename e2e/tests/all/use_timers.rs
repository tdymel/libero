//! `use_timeout`, `use_interval` and the debounce hooks: typing into a field
//! with a debounced value, and an interval that starts and stops.

use std::time::{Duration, Instant};

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::clock;
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, js, wait};

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
    ensure!(ticks(d).await? == 0, "ticked before start");
    let held = d.hold_timers(&[TICK_MS]).await?;

    d.click("#toggle").await?;
    text_is(d, "#toggle", "Stop").await?;
    if held {
        armed_is(d, TICK_MS, 1, "start to arm the interval").await?;
        for tick in 1..=2 {
            ensure!(d.fire_timers(TICK_MS).await? == 1, "no interval armed");
            text_is(d, "#ticks", &tick.to_string()).await?;
        }
    } else {
        eventually(d, "two ticks", async |d| Ok(ticks(d).await? >= 2)).await?;
    }

    d.click("#toggle").await?;
    text_is(d, "#toggle", "Start").await?;
    if held {
        // Stop clears the interval: nothing is left to fire.
        armed_is(d, TICK_MS, 0, "stop to clear the interval").await?;
        ensure!(ticks(d).await? == 2, "ticked without a fired timer");
        return Ok(());
    }
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
    ensure!(ticks(d).await? == at_stop, "kept ticking after stop");
    Ok(())
}

async fn timeout_fires_once<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    text_is(d, "#flashing", "off").await?;
    let held = d.hold_timers(&[FLASH_MS]).await?;
    d.click("#flash").await?;
    text_is(d, "#flashing", "on").await?;
    if held {
        armed_is(d, FLASH_MS, 1, "the click to arm the timeout").await?;
        d.settle().await?;
        ensure!(d.text("#flashing").await? == "on", "ended before its timer");
        ensure!(d.fire_timers(FLASH_MS).await? == 1, "no timeout armed");
        armed_is(d, FLASH_MS, 0, "the timeout to fire once").await?;
    }
    text_is(d, "#flashing", "off").await
}

async fn armed_is<D: Driver>(d: &mut D, ms: u32, count: usize, what: &str) -> Result<()> {
    eventually(d, what, async |d| Ok(d.armed(ms).await? == count)).await
}

/// The `/use-timers/interval` fixture's period and timeout.
const TICK_MS: u32 = 100;
const FLASH_MS: u32 = 150;

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

async fn text(page: &chromiumoxide::Page, id: &str) -> String {
    js(
        page,
        &format!("document.getElementById('{id}').textContent"),
    )
    .await
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
        clock::hold(page, &[DEBOUNCE_MS, SAVE_MS, THROTTLE_MS])
            .await
            .unwrap();
        let _: bool = js(page, "(document.getElementById('field').focus(), true)").await;
        keyboard::type_text(page, "abc").await.unwrap();

        reads(page, "typed", "abc").await;
        clock::until_armed(page, DEBOUNCE_MS, 1, "typing to arm the debounce")
            .await
            .unwrap();
        clock::until_armed(page, SAVE_MS, 1, "typing to arm the debounced callback")
            .await
            .unwrap();
        // Armed means held: a settle, not a wait past the delay, shows nothing ran early.
        clock::settle(page).await.unwrap();
        assert_eq!(text(page, "settled").await, "", "settled before its timer");
        assert_eq!(text(page, "saved").await, "", "saved before its timer");
        assert!(
            !text(page, "throttled").await.is_empty(),
            "the leading key waited"
        );

        clock::fire(page, DEBOUNCE_MS).await.unwrap();
        reads(page, "settled", "abc").await;
        assert_eq!(text(page, "saved").await, "");
        clock::fire(page, SAVE_MS).await.unwrap();
        reads(page, "saved", "abc").await;
        clock::fire(page, THROTTLE_MS).await.unwrap();
        reads(page, "throttled", "abc").await;

        fixture
            .console
            .assert_clean("typing into a debounced field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
