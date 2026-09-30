//! `use_long_press`: a held touch fires once and swallows its click, a short
//! touch and a plain click stay taps.

use std::time::{Duration, Instant};

use anyhow::Result;
use e2e::driver::{Driver, eventually};

const TARGET: &str = "#target";

async fn text_is<D: Driver>(d: &mut D, selector: &str, expected: &str) -> Result<()> {
    eventually(d, &format!("{selector} to read {expected:?}"), async |d| {
        Ok(d.text(selector).await? == expected)
    })
    .await
}

/// The hold cue is off once a press has fired or ended.
async fn pressing_is<D: Driver>(d: &mut D, expected: &str) -> Result<()> {
    eventually(d, &format!("data-pressing to be {expected}"), async |d| {
        Ok(d.attr(TARGET, "data-pressing").await?.as_deref() == Some(expected))
    })
    .await
}

/// The press timer (`LongPressOptions::default().ms`).
const PRESS_MS: u32 = 400;

/// Until a wrong late callback or click would have landed: on a held clock fires every
/// pending press timer and settles, elsewhere waits 500 ms.
async fn past_the_delay<D: Driver>(d: &mut D, held: bool) -> Result<()> {
    if held {
        d.fire_timers(PRESS_MS).await?;
        return d.settle().await;
    }
    let started = Instant::now();
    while started.elapsed() < Duration::from_millis(500) {
        d.idle().await;
    }
    Ok(())
}

async fn a_held_touch_fires_once<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.long_press(TARGET, 900).await?;
    text_is(d, "#holds", "1").await?;
    // Held only now: the first press timer has to run on the real clock.
    let held = d.hold_timers(&[PRESS_MS]).await?;
    past_the_delay(d, held).await?;
    text_is(d, "#holds", "1").await?;
    text_is(d, "#taps", "0").await?;
    pressing_is(d, "false").await
}

async fn a_short_touch_is_a_tap<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let held = d.hold_timers(&[PRESS_MS]).await?;
    d.long_press(TARGET, 100).await?;
    text_is(d, "#taps", "1").await?;
    past_the_delay(d, held).await?;
    text_is(d, "#holds", "0").await?;
    pressing_is(d, "false").await
}

async fn a_click_is_a_tap<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let held = d.hold_timers(&[PRESS_MS]).await?;
    d.click(TARGET).await?;
    text_is(d, "#taps", "1").await?;
    past_the_delay(d, held).await?;
    text_is(d, "#holds", "0").await
}

e2e::scenario!(
    a_held_touch_fires_the_callback_once_and_swallows_the_click,
    "/use-long-press/basic",
    a_held_touch_fires_once,
    desktop: skip("1126: no touch input under Xvfb")
);
e2e::scenario!(
    a_short_touch_still_clicks,
    "/use-long-press/basic",
    a_short_touch_is_a_tap,
    desktop: skip("1126: no touch input under Xvfb")
);
e2e::scenario!(
    a_plain_click_never_counts_as_a_press,
    "/use-long-press/basic",
    a_click_is_a_tap
);
