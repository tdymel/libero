//! `ScatterChart`: the pointer, the keys and the legend over free points, the summary with
//! its trends and bubbles. Web and Android: charts are parked on Blitz.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const LAYER: &str = "#chart [data-chart-layer]";
const TOOLTIP: &str = "#chart [data-slot=tooltip]";
const STATUS: &str = "#chart [role=status]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("scatter_chart", "/scatter-chart").run();
}

async fn reads<D: Driver>(d: &mut D, id: &str, want: &str, what: &str) -> Result<()> {
    let label = format!("{what}: {id} to read {want:?}");
    eventually(d, &label, async |d| Ok(d.text(id).await? == want)).await
}

/// The summary names the kind and x range, each series' count and ranges and its trend.
#[test]
fn the_summary_says_each_series_and_its_trend() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/scatter-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg")
            .await
            .unwrap();
        let summary: String = e2e::js(
            &fixture.page,
            "document.querySelector('#chart [data-chart-summary]').textContent",
        )
        .await;
        for want in [
            "Scatter chart from 10 to 30.",
            "A: 3 points, x from 10 to 30, y from 30 to 45.",
            "A: linear trend, rising.",
            "B: 2 points, x from 13 to 28, y from 20 to 25.",
        ] {
            assert!(summary.contains(want), "{want:?} in {summary}");
        }
        fixture.close().await.unwrap();
    });
}

/// Bubbles say their sizes in the summary and the announcement.
async fn bubbles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let summary = d.text("#chart [data-chart-summary]").await?;
    ensure!(
        summary.contains("Cities: sizes from 300 to 1,200."),
        "summary {summary:?}"
    );
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, STATUS, "Cities: x 1, y 20, size 300", "the first bubble").await
}

e2e::scenario!(bubbles_say_their_sizes, "/scatter-chart/bubbles", bubbles);

/// The pointer picks the nearest point; the tooltip gives its own x and y, not a column.
async fn hovers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.hover(LAYER).await?;
    eventually(d, "a point under the pointer", async |d| {
        Ok(!d.text("#active").await?.is_empty())
    })
    .await?;
    eventually(d, "the tooltip", async |d| d.exists(TOOLTIP).await).await?;
    let tip = d.text(TOOLTIP).await?;
    ensure!(tip.contains("x ") && tip.contains("y "), "tooltip {tip:?}");
    let crosshairs = d.rects("#chart [data-chart-crosshair]").await?;
    ensure!(crosshairs.len() == 2, "{} crosshairs", crosshairs.len());
    Ok(())
}

e2e::scenario!(
    hovering_picks_the_nearest_point,
    "/scatter-chart",
    hovers,
    android: skip("a touch screen has no hover"),
    ios: skip("a touch screen has no hover")
);

/// Left and right walk a series in x order, up and down cross to the next series' nearest x.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "A 10 30", "the first ArrowRight").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "A 20 45", "ArrowRight").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "B 13 20", "ArrowDown to the nearest x").await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "B 28 25", "End").await?;
    d.press(keyboard::ARROW_UP).await?;
    reads(d, "#active", "A 30 40", "ArrowUp").await?;
    reads(d, STATUS, "A: x 30, y 40", "the announcement").await?;
    d.press(keyboard::HOME).await?;
    reads(d, "#active", "A 10 30", "Home").await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#clicked", "A 10 30", "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#active", "", "Escape").await?;
    ensure!(!d.exists(TOOLTIP).await?, "Escape left the tooltip");
    Ok(())
}

e2e::scenario!(the_keys_walk_the_points, "/scatter-chart", keys);

/// A legend button hides a series from the drawing; the last shown one stays.
async fn legend<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let a = "#chart [data-slot=legend-item]:nth-child(1) button";
    let b = "#chart [data-slot=legend-item]:nth-child(2) button";
    const DRAWING: &str = "document.querySelector('#chart [data-chart-drawing]').innerHTML";
    let before = d.evaluate(DRAWING).await?;
    d.click(b).await?;
    reads(d, "#hidden", "B", "a click on B").await?;
    eventually(d, "B to leave the drawing", async |d| {
        Ok(d.evaluate(DRAWING).await? != before)
    })
    .await?;
    let held = d.attr(a, "aria-disabled").await?;
    ensure!(
        held.as_deref() == Some("true"),
        "the last series is not held"
    );
    d.focus(LAYER).await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "A 30 40", "End with B hidden").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "A 30 40", "ArrowDown with no other series").await?;
    d.click(b).await?;
    reads(d, "#hidden", "", "a second click on B").await
}

e2e::scenario!(
    the_legend_hides_and_shows_a_series,
    "/scatter-chart",
    legend,
    native: skip("Blitz has no script reads")
);
