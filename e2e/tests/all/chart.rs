//! The docs Chart page's type switch: one control over a line, a bar, a pie, a funnel chart,
//! a heatmap and a candlestick chart. Each chart has its own unit (`line_chart`, `bar_chart`,
//! `pie_chart`, `funnel_chart`, `heatmap_chart`, `candlestick_chart`); here only what the switch adds. Web and Android: charts are parked on Blitz.

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, eventually};

#[test]
fn it_meets_the_baseline() {
    Suite::new("chart", "/chart").run();
}

/// What the chart's hidden summary names: its series or its slices.
const NAMES: &str = "#chart [data-chart-summary]";

async fn shows<D: Driver>(d: &mut D, what: &str, name: &str) -> Result<()> {
    let label = format!("{what}: the summary to name {name}");
    eventually(d, &label, async |d| Ok(d.text(NAMES).await?.contains(name))).await
}

/// Each segment swaps the chart for its type: one chart at a time, the old one gone.
async fn switches<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    shows(d, "the first chart", "This week").await?;
    for (segment, name, gone) in [
        (1, "2025", "This week"),
        (2, "Chrome", "2025"),
        (3, "Visits", "Chrome"),
        (4, "Heatmap of 2 rows", "Visits"),
        (5, "Candlestick chart of 2", "Heatmap"),
        (0, "This week", "Candlestick"),
    ] {
        d.click(&format!(
            "[role=radiogroup] label[for$=\"-segment-{segment}\"]"
        ))
        .await?;
        shows(d, &format!("segment {segment}"), name).await?;
        ensure!(
            !d.text(NAMES).await?.contains(gone),
            "{gone} stayed after segment {segment}"
        );
        let charts = d.rects("#chart").await?.len();
        ensure!(charts == 1, "{charts} charts after segment {segment}");
    }
    Ok(())
}

e2e::scenario!(
    the_segments_swap_the_chart,
    "/chart",
    switches,
    native: skip("charts are parked on Blitz")
);
