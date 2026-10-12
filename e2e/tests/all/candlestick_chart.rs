//! `CandlestickChart`: hollow rising and filled falling candles with their key; the pointer
//! and the keys over the candles. Web and Android: charts are parked on Blitz.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const LAYER: &str = "#chart [data-chart-layer]";
const TOOLTIP: &str = "#chart [data-slot=tooltip]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("candlestick_chart", "/candlestick-chart").run();
}

/// The key names the hollow and the filled candle; the summary reads the change, the
/// extremes and the directions.
#[test]
fn the_key_and_the_summary_tell_rising_from_falling() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/candlestick-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg rect")
            .await
            .unwrap();
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart [data-chart-drawing] text')].map(t => t.textContent.trim())",
        )
        .await;
        for key in ["Up", "Down"] {
            assert!(labels.contains(&key.to_string()), "{labels:?}");
        }
        let summary: String = e2e::js(
            &fixture.page,
            "document.querySelector('#chart [data-chart-summary]').textContent",
        )
        .await;
        assert!(
            summary.contains("Candlestick chart of 3 periods from Mon to Wed.")
                && summary.contains("Opened at 20, closed at 30: +10 (+50%).")
                && summary.contains("Highest: 40 on Tue. Lowest: 10 on Mon.")
                && summary.contains("1 up, 1 down, 1 unchanged."),
            "{summary}"
        );
        fixture.close().await.unwrap();
    });
}

async fn reads<D: Driver>(d: &mut D, id: &str, want: &str, what: &str) -> Result<()> {
    let label = format!("{what}: {id} to read {want:?}");
    eventually(d, &label, async |d| Ok(d.text(id).await? == want)).await
}

/// Left and right walk the candles; each is announced and tipped with its four prices.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "Mon 0 34", "the first ArrowRight").await?;
    eventually(d, "the announcement", async |d| {
        let said = d.text("#chart [role=status]").await?;
        Ok(said == "Price, Mon: open 20, high 38, low 10, close 34, up")
    })
    .await?;
    eventually(d, "the tooltip", async |d| {
        if !d.exists(TOOLTIP).await? {
            return Ok(false);
        }
        Ok(d.text(TOOLTIP).await?.contains("close 34"))
    })
    .await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "Tue 1 30", "ArrowRight").await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "Wed 2 30", "End").await?;
    d.press(keyboard::HOME).await?;
    reads(d, "#active", "Mon 0 34", "Home").await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#clicked", "Mon 0 34", "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#active", "", "Escape").await?;
    ensure!(!d.exists(TOOLTIP).await?, "Escape left the tooltip");
    Ok(())
}

e2e::scenario!(
    the_keys_walk_the_candles,
    "/candlestick-chart",
    keys,
    native: skip("charts are parked on Blitz")
);

/// A click on a candle reports it. A tap pins it, a second tap clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // The first rect drawn is Monday's body; the key's swatches come last.
    let candle = d.rect("#chart [data-chart-drawing] svg > rect").await?;
    let (x, y) = (
        candle.x + candle.width / 2.0,
        candle.y + candle.height / 2.0,
    );
    d.click_at(x, y).await?;
    if d.platform() == Platform::Android {
        reads(d, "#active", "Mon 0 34", "a tap").await?;
        ensure!(
            d.text("#clicked").await?.is_empty(),
            "the first tap clicked"
        );
        d.click_at(x, y).await?;
    }
    reads(d, "#clicked", "Mon 0 34", "a click on Monday's candle").await
}

e2e::scenario!(
    a_click_on_a_candle_reports_it,
    "/candlestick-chart",
    clicks,
    native: skip("charts are parked on Blitz"),
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);
