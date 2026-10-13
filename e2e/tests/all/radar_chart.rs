//! `RadarChart`: the pointer, the keys and the legend round the spokes, the summary with
//! each series' highest and lowest axis. Web and Android: charts are parked on Blitz.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const LAYER: &str = "#chart [data-chart-layer]";
const TOOLTIP: &str = "#chart [data-slot=tooltip]";
const STATUS: &str = "#chart [role=status]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("radar_chart", "/radar-chart").run();
}

async fn reads<D: Driver>(d: &mut D, id: &str, want: &str, what: &str) -> Result<()> {
    let label = format!("{what}: {id} to read {want:?}");
    eventually(d, &label, async |d| Ok(d.text(id).await? == want)).await
}

/// The summary names the axes, each series' highest and lowest, and a missing value.
#[test]
fn the_summary_says_each_series_highest_and_lowest() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/radar-chart", Viewport::Desktop)
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
            "Radar chart over 4 axes: Speed, Power, Range, Cost.",
            "X: highest on Speed, 8; lowest on Cost, 5.",
            "Y: highest on Range, 18; lowest on Cost, 4.",
            "Y: no value on Power, drawn at the centre.",
        ] {
            assert!(summary.contains(want), "{want:?} in {summary}");
        }
        fixture.close().await.unwrap();
    });
}

/// The pointer picks the nearest point; the tooltip lists every series on its axis.
async fn hovers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.hover(LAYER).await?;
    eventually(d, "a point under the pointer", async |d| {
        Ok(!d.text("#active").await?.is_empty())
    })
    .await?;
    eventually(d, "the tooltip", async |d| d.exists(TOOLTIP).await).await?;
    let rows = d
        .rects("#chart [data-slot=tooltip] [data-chart-row]")
        .await?;
    ensure!(!rows.is_empty(), "no tooltip rows");
    let crosshairs = d.rects("#chart [data-chart-crosshair]").await?;
    ensure!(crosshairs.is_empty(), "{} crosshairs", crosshairs.len());
    Ok(())
}

e2e::scenario!(
    hovering_picks_the_nearest_point,
    "/radar-chart",
    hovers,
    android: skip("a touch screen has no hover"),
    ios: skip("a touch screen has no hover")
);

/// Left and right walk a series round the axes, up and down cross to the other series.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "X Speed 8", "the first ArrowRight").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "X Power 6", "ArrowRight").await?;
    reads(d, STATUS, "X, Power: 6", "the announcement").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(
        d,
        "#active",
        "Y Speed 6",
        "ArrowDown past Y's missing Power",
    )
    .await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "Y Cost 4", "End").await?;
    d.press(keyboard::ARROW_UP).await?;
    reads(d, "#active", "X Cost 5", "ArrowUp").await?;
    d.press(keyboard::HOME).await?;
    reads(d, "#active", "X Speed 8", "Home").await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#clicked", "X Speed 8", "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#active", "", "Escape").await?;
    ensure!(!d.exists(TOOLTIP).await?, "Escape left the tooltip");
    Ok(())
}

e2e::scenario!(the_keys_walk_the_spokes, "/radar-chart", keys);

/// A legend button hides a series from the drawing; the last shown one stays.
async fn legend<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let x = "#chart [data-slot=legend-item]:nth-child(1) button";
    let y = "#chart [data-slot=legend-item]:nth-child(2) button";
    const DRAWING: &str = "document.querySelector('#chart [data-chart-drawing]').innerHTML";
    let before = d.evaluate(DRAWING).await?;
    d.click(y).await?;
    reads(d, "#hidden", "Y", "a click on Y").await?;
    eventually(d, "Y to leave the drawing", async |d| {
        Ok(d.evaluate(DRAWING).await? != before)
    })
    .await?;
    let held = d.attr(x, "aria-disabled").await?;
    ensure!(
        held.as_deref() == Some("true"),
        "the last series is not held"
    );
    d.focus(LAYER).await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "X Cost 5", "End with Y hidden").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "X Cost 5", "ArrowDown with no other series").await?;
    d.click(y).await?;
    reads(d, "#hidden", "", "a second click on Y").await
}

e2e::scenario!(
    the_legend_hides_and_shows_a_series,
    "/radar-chart",
    legend,
    native: skip("Blitz has no script reads")
);
