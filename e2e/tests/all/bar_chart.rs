//! `BarChart`: drawn at its box's size, categories top down when lying, values on bars;
//! the pointer, the keys and the legend over the bars. Web and Android: charts are parked
//! on Blitz.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const LAYER: &str = "#chart [data-chart-layer]";
const TOOLTIP: &str = "#chart [data-slot=tooltip]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("bar_chart", "/bar-chart").run();
}

/// Once measured, the SVG is drawn again at the plot's width: text keeps its size.
#[test]
fn the_bars_are_drawn_at_the_plot_s_width() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#chart svg')?.getAttribute('viewBox')?.startsWith('0 0 480 ')",
            "the chart drawn at 480 px",
        )
        .await
        .unwrap();
        // A gap in a series draws no bar.
        let bars: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart svg rect').length",
        )
        .await;
        assert_eq!(bars, 5);
        let rows: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart tbody tr').length",
        )
        .await;
        assert_eq!(rows, 3);
        fixture.close().await.unwrap();
    });
}

/// Lying, the first category is the top row, a stack's bars share it end to end, and
/// each value is written on its bar.
#[test]
fn stacked_lying_bars_read_top_down() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart/stacked", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg rect")
            .await
            .unwrap();
        let [q1, q3]: [f64; 2] = e2e::js(
            &fixture.page,
            "['Q1', 'Q3'].map(q => [...document.querySelectorAll('#chart svg text')] \
             .find(t => t.textContent.trim() === q).getBBox().y)",
        )
        .await;
        assert!(q1 < q3, "Q1 at {q1}, Q3 at {q3}");
        // The renderer draws the rows bottom up: Q1 is each series' last bar.
        let [a, b]: [[f64; 3]; 2] = e2e::js(
            &fixture.page,
            "(() => { const r = [...document.querySelectorAll('#chart svg rect')] \
             .map(r => [r.x.baseVal.value, r.y.baseVal.value, r.width.baseVal.value]); \
             return [r[2], r[4]]; })()",
        )
        .await;
        assert_eq!(a[1], b[1], "{a:?} {b:?}");
        assert!((a[0] + a[2] - b[0]).abs() < 1.0, "{a:?} {b:?}");
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg text')].map(t => t.textContent.trim())",
        )
        .await;
        assert!(labels.contains(&"140".to_string()), "{labels:?}");
        fixture.close().await.unwrap();
    });
}

async fn reads<D: Driver>(d: &mut D, id: &str, want: &str, what: &str) -> Result<()> {
    let label = format!("{what}: {id} to read {want:?}");
    eventually(d, &label, async |d| Ok(d.text(id).await? == want)).await
}

/// The pointer over the plot's centre picks Q2, where only 2025 has a bar.
async fn hovers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(!d.exists(TOOLTIP).await?, "a tooltip before any hover");
    d.hover(LAYER).await?;
    reads(d, "#active", "2025 Q2 132", "a hover over Q2").await?;
    eventually(d, "the tooltip", async |d| d.exists(TOOLTIP).await).await?;
    let tip = d.text(TOOLTIP).await?;
    ensure!(
        tip.contains("Q2") && tip.contains("132") && !tip.contains("2026"),
        "tooltip {tip:?}"
    );
    // The marker's stroke stays inside its bar, off the x axis.
    let marker = d.rect("#chart [data-chart-marker]").await?;
    let bars = d.rects("#chart [data-chart-drawing] rect").await?;
    ensure!(
        bars.iter().any(|bar| marker.x >= bar.x
            && marker.y >= bar.y
            && marker.x + marker.width <= bar.x + bar.width + 0.01
            && marker.y + marker.height <= bar.y + bar.height + 0.01),
        "the marker {marker:?} is not inside a bar"
    );
    Ok(())
}

e2e::scenario!(
    hovering_shows_the_bar_s_tooltip_and_marker,
    "/bar-chart",
    hovers,
    android: skip("a touch screen has no hover"),
    native: skip("charts are parked on Blitz")
);

/// Arrows walk a series and cross to the other in the same category; Enter clicks.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "2025 Q1 120", "the first ArrowRight").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "2025 Q2 132", "ArrowRight").await?;
    // 2026 has no bar at Q2: the nearest, the first of two.
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "2026 Q1 140", "ArrowDown").await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "2026 Q3 150", "End").await?;
    eventually(d, "the announcement", async |d| {
        Ok(d.text("#chart [role=status]").await? == "2026, Q3: 150")
    })
    .await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#clicked", "2026 Q3 150", "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#active", "", "Escape").await?;
    ensure!(!d.exists(TOOLTIP).await?, "Escape left the tooltip");
    Ok(())
}

e2e::scenario!(
    the_keys_walk_the_bars,
    "/bar-chart",
    keys,
    native: skip("charts are parked on Blitz")
);

/// Lying, up and down walk the categories top down; left and right cross the series.
async fn lying_keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "2025 Q1 120", "the first ArrowDown").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "2025 Q2 132", "ArrowDown").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "2026 Q1 140", "ArrowRight").await
}

e2e::scenario!(
    the_keys_follow_lying_bars,
    "/bar-chart/stacked",
    lying_keys,
    native: skip("charts are parked on Blitz")
);

/// A click picks the bar under it. A tap pins it, a second tap on it clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let bar = d.rect("#chart [data-chart-drawing] rect").await?;
    let (x, y) = (bar.x + bar.width / 2.0, bar.y + bar.height / 2.0);
    d.click_at(x, y).await?;
    if d.platform() == Platform::Android {
        reads(d, "#active", "2025 Q1 120", "a tap").await?;
        ensure!(
            d.text("#clicked").await?.is_empty(),
            "the first tap clicked"
        );
        d.click_at(x, y).await?;
    }
    reads(d, "#clicked", "2025 Q1 120", "a click on Q1").await
}

e2e::scenario!(
    a_click_on_a_bar_reports_it,
    "/bar-chart",
    clicks,
    native: skip("charts are parked on Blitz")
);

/// A legend button hides its series' bars; the last shown one stays.
async fn legend<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let first = "#chart [data-slot=legend-item]:nth-child(1) button";
    let second = "#chart [data-slot=legend-item]:nth-child(2) button";
    let bars = "#chart [data-chart-drawing] rect";
    d.click(second).await?;
    reads(d, "#hidden", "2026", "a click on 2026").await?;
    eventually(d, "2026's bars to leave", async |d| {
        Ok(d.rects(bars).await?.len() == 3)
    })
    .await?;
    let held = d.attr(first, "aria-disabled").await?;
    ensure!(
        held.as_deref() == Some("true"),
        "the last series is not held"
    );
    d.click(second).await?;
    reads(d, "#hidden", "", "a second click on 2026").await
}

e2e::scenario!(
    the_legend_hides_and_shows_a_series,
    "/bar-chart",
    legend,
    native: skip("charts are parked on Blitz")
);
