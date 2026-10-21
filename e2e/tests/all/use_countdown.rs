//! `use_countdown`: fifty countdowns run on one held 1 s interval, reach zero
//! together and stop it.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};

/// The shared ticker's period.
const SECOND_MS: u32 = 1000;
const LAST: &str = "#c49";

async fn armed_is<D: Driver>(d: &mut D, count: usize, what: &str) -> Result<()> {
    eventually(d, what, async |d| Ok(d.armed(SECOND_MS).await? == count)).await
}

async fn fifty_share_one_ticker<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        d.hold_timers(&[SECOND_MS]).await?,
        "the driver cannot hold timers"
    );
    d.click("#start").await?;
    eventually(d, "the countdowns to mount", async |d| d.exists(LAST).await).await?;
    // The timeout up to the next whole second runs on the real clock, then the interval is held.
    armed_is(d, 1, "one ticker for fifty countdowns").await?;
    ensure!(
        d.text(LAST).await? != "0:00",
        "a countdown read zero at once"
    );

    // Each fire reads the real clock: fire until the ends have passed.
    eventually(d, "every label to read 0:00", async |d| {
        d.fire_timers(SECOND_MS).await?;
        d.settle().await?;
        Ok(!d.exists("[data-zero=false]").await?)
    })
    .await?;
    ensure!(d.text(LAST).await? == "0:00", "{LAST} did not read 0:00");
    armed_is(d, 0, "the ticker to stop once all are at zero").await
}

e2e::scenario!(
    fifty_countdowns_tick_on_one_timer_and_stop_at_zero,
    "/use-countdown",
    fifty_share_one_ticker
);
