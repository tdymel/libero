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
            "document.querySelectorAll('#chart [data-chart-drawing] path').length",
        )
        .await;
        assert_eq!(bars, 5);
        // The data is a Table's job: the chart draws none (3461).
        let tables: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart table').length",
        )
        .await;
        assert_eq!(tables, 0);
        fixture.close().await.unwrap();
    });
}

/// Per bar, the radius of its corners as `[top left, top right, bottom right, bottom left]`.
const CORNERS: &str = "[...document.querySelectorAll('#chart [data-chart-drawing] path')] \
    .map(p => p.getAttribute('d').split('A').slice(1).map(parseFloat)) \
    .map(([tr, br, bl, tl]) => [tl, tr, br, bl])";

/// A bar is rounded at its free end only, the one away from the axis (3469).
#[test]
fn a_bar_rounds_only_its_free_end() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart [data-chart-drawing] path")
            .await
            .unwrap();
        let rising: Vec<[f64; 4]> = e2e::js(&fixture.page, CORNERS).await;
        assert_eq!(rising.len(), 5);
        assert!(
            rising
                .iter()
                .all(|[tl, tr, br, bl]| *tl > 0.0 && *tr > 0.0 && *br == 0.0 && *bl == 0.0),
            "{rising:?}"
        );
        fixture.close().await.unwrap();
        let fixture = Fixture::open("/bar-chart/stacked", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart [data-chart-drawing] path")
            .await
            .unwrap();
        // Lying, only the right end rounds, and only where no bar follows: 3 stacks.
        let lying: Vec<[f64; 4]> = e2e::js(&fixture.page, CORNERS).await;
        assert!(
            lying
                .iter()
                .all(|[tl, tr, br, bl]| *tl == 0.0 && *bl == 0.0 && (*tr > 0.0) == (*br > 0.0)),
            "{lying:?}"
        );
        assert_eq!(lying.iter().filter(|c| c[1] > 0.0).count(), 3, "{lying:?}");
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
        // Every bar is a path, rounded at its free end only (3469).
        wait::for_visible(&fixture.page, "#chart [data-chart-drawing] path")
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
            "(() => { const r = [...document.querySelectorAll('#chart [data-chart-drawing] path')] \
             .map(p => p.getBBox()).map(b => [b.x, b.y, b.width]); \
             return [r[2], r[4]]; })()",
        )
        .await;
        assert!((a[1] - b[1]).abs() < 0.5, "{a:?} {b:?}");
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

/// Half-band bars, a loss in its own colour with its value on it, an error bar per bar
/// centred on it.
#[test]
fn styled_bars_keep_their_width_colour_and_ranges() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart/styled", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart [data-chart-drawing] path")
            .await
            .unwrap();
        let bars: Vec<[f64; 2]> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart [data-chart-drawing] path')] \
             .map(p => p.getBBox()).map(b => [b.x + b.width / 2, b.width])",
        )
        .await;
        assert_eq!(bars.len(), 3);
        let band = bars[1][0] - bars[0][0];
        assert!(
            bars.iter()
                .all(|[_, width]| (width / band - 0.5).abs() < 0.03),
            "{bars:?}"
        );
        let fills: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart [data-chart-drawing] path')] \
             .map(p => p.getAttribute('fill').toLowerCase())",
        )
        .await;
        assert!(fills[1] == "#c92a2a" && fills[0] != fills[1], "{fills:?}");
        let ranges: Vec<f64> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart [data-chart-drawing] line')] \
             .filter(l => l.getAttribute('x1') === l.getAttribute('x2') && +l.getAttribute('y1') !== +l.getAttribute('y2')) \
             .map(l => +l.getAttribute('x1'))",
        )
        .await;
        for [centre, _] in &bars {
            assert!(
                ranges.iter().any(|x| (x - centre).abs() < 0.5),
                "no range on {centre}: {ranges:?}"
            );
        }
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg text')].map(t => t.textContent.trim())",
        )
        .await;
        assert!(labels.contains(&"-40".to_string()), "{labels:?}");
        fixture.close().await.unwrap();
    });
}

/// A percent stack with losses reads shares of its absolute sum: its axis ends at 100%
/// and stops at its deepest loss (3470).
#[test]
fn a_percent_axis_with_losses_ends_at_100() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart/percent", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart [data-chart-drawing] path")
            .await
            .unwrap();
        let ticks: Vec<f64> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg text')] \
             .map(t => t.textContent.trim().replace('\\u2212', '-')) \
             .filter(t => /^-?\\d+%$/.test(t)).map(t => parseInt(t))",
        )
        .await;
        let (low, high) = ticks
            .iter()
            .fold((f64::MAX, f64::MIN), |(l, h), t| (l.min(*t), h.max(*t)));
        assert!(high == 100.0 && (-100.0..0.0).contains(&low), "{ticks:?}");
        fixture.close().await.unwrap();
    });
}

/// A range on a stack or a lying bar is drawn, whole inside the plot (3480).
#[test]
fn stacked_and_lying_bars_draw_their_ranges_inside_the_plot() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart/errors", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#lying [data-chart-drawing] path")
            .await
            .unwrap();
        for (chart, along) in [("chart", "y"), ("lying", "x")] {
            // Per range: its stem, then a cap at each end.
            let [stems, outside]: [usize; 2] = e2e::js(
                &fixture.page,
                &format!(
                    "(() => {{ const svg = document.querySelector('#{chart} svg'); \
                     const lines = [...svg.querySelectorAll('line[stroke-width=\"1.5\"]')]; \
                     const box = svg.getBoundingClientRect(); \
                     const scale = box.{size} / svg.viewBox.baseVal.{size}; \
                     const stems = lines.filter(l => l.getAttribute('{other}1') === l.getAttribute('{other}2')); \
                     const outside = lines.filter(l => ['{along}1', '{along}2'].some(a => \
                       +l.getAttribute(a) < 0 || +l.getAttribute(a) * scale > box.{size})); \
                     return [stems.length, outside.length]; }})()",
                    size = if along == "y" { "height" } else { "width" },
                    other = if along == "y" { "x" } else { "y" },
                ),
            )
            .await;
            assert_eq!(stems, 6, "{chart}");
            assert_eq!(outside, 0, "{chart}");
        }
        fixture.close().await.unwrap();
    });
}

/// The keys' bar is announced and tipped with its error range.
async fn ranges<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "the announcement", async |d| {
        Ok(d.text("#chart [role=status]").await? == "2026, Q1: 120 (110–130)")
    })
    .await?;
    eventually(d, "the tooltip", async |d| {
        Ok(d.exists(TOOLTIP).await? && d.text(TOOLTIP).await?.contains("120 (110–130)"))
    })
    .await
}

e2e::scenario!(the_keys_read_a_bar_s_range, "/bar-chart/styled", ranges);

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
    let bars = d.rects("#chart [data-chart-drawing] path").await?;
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
    ios: skip("a touch screen has no hover")
);

/// Escape closes a hovered tooltip though focus never entered the plot (3428).
async fn escapes_a_hover<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.hover(LAYER).await?;
    eventually(d, "the tooltip", async |d| d.exists(TOOLTIP).await).await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the tooltip to close", async |d| {
        Ok(!d.exists(TOOLTIP).await?)
    })
    .await
}

e2e::scenario!(
    escape_closes_a_hovered_tooltip_without_focus,
    "/bar-chart",
    escapes_a_hover,
    android: skip("a touch screen has no hover"),
    ios: skip("a touch screen has no hover")
);

/// A loading chart says so in a live region, not only in its summary (3488).
async fn loads<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    says(d, "Loading chart", "a loading chart").await
}

e2e::scenario!(a_loading_chart_says_so, "/bar-chart/loading", loads);

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

e2e::scenario!(the_keys_walk_the_bars, "/bar-chart", keys);

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

e2e::scenario!(the_keys_follow_lying_bars, "/bar-chart/stacked", lying_keys);

/// A click picks the bar under it. A tap pins it, a second tap on it clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let bar = d.rect("#chart [data-chart-drawing] path").await?;
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
    native: skip("a click on a chart mark keeps the vdom busy past 200 polls on Blitz"),
    ios: skip("3416: the tap reaches the chart layer as a click, but no mark's click is reported")
);

/// A legend button hides its series' bars; the last shown one stays.
async fn legend<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let first = "#chart [data-slot=legend-item]:nth-child(1) button";
    let second = "#chart [data-slot=legend-item]:nth-child(2) button";
    let bars = "#chart [data-chart-drawing] path";
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

e2e::scenario!(the_legend_hides_and_shows_a_series, "/bar-chart", legend);

/// A line among bars (3463): drawn over them, keyed by a stroke, its highest point pinned.
#[test]
fn a_line_is_drawn_over_the_bars_and_keyed_by_a_stroke() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart/line", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart [data-chart-drawing] path")
            .await
            .unwrap();
        let bars: usize = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart [data-chart-drawing] path')] \
             .filter(p => p.getAttribute('fill') && p.getAttribute('fill') !== 'none').length",
        )
        .await;
        assert_eq!(bars, 6);
        let keys: Vec<bool> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart [data-chart-swatch]')] \
             .map(s => s.hasAttribute('data-line'))",
        )
        .await;
        assert_eq!(keys, [false, true, false]);
        let pins: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart .ct-pin text')].map(t => t.textContent.trim())",
        )
        .await;
        assert!(pins.len() == 1 && pins[0].starts_with("18"), "{pins:?}");
        fixture.close().await.unwrap();
    });
}

/// Up and down cross the series in their written order, the line between the bars.
async fn line_keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "Revenue Q1 120", "the first ArrowRight").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "Margin Q1 12", "ArrowDown onto the line").await?;
    eventually(d, "the line's tooltip", async |d| {
        Ok(d.exists(TOOLTIP).await? && d.text(TOOLTIP).await?.contains("Margin"))
    })
    .await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "Costs Q1 80", "ArrowDown off the line").await
}

e2e::scenario!(
    the_keys_walk_onto_a_line_among_bars,
    "/bar-chart/line",
    line_keys
);

/// A pointer on the line's point picks it over the bar beneath.
async fn line_point<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let pin = d.rect("#chart .ct-pin rect").await?;
    // The pin stands 6 px above its point.
    d.click_at(pin.x + pin.width / 2.0, pin.y + pin.height + 6.0)
        .await?;
    reads(d, "#active", "Margin Q2 18", "a click on the line's point").await
}

e2e::scenario!(
    a_pointer_on_a_line_s_point_picks_it,
    "/bar-chart/line",
    line_point,
    native: skip("a click on a line's point among bars picks nothing on Blitz")
);

const PLUS: keyboard::Key = keyboard::Key {
    key: "+",
    code: "Equal",
    vk: 187,
    text: Some("+"),
};

async fn says<D: Driver>(d: &mut D, want: &str, what: &str) -> Result<()> {
    eventually(
        d,
        &format!("{what}: the chart to say {want:?}"),
        async |d| Ok(d.text("#chart [role=status]").await? == want),
    )
    .await
}

/// Lying bars zoom down their side; a drag down brushes; the reset shows them all.
async fn zooms_down<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(PLUS).await?;
    says(d, "Showing Mar to May", "+").await?;
    let plot = d.rect(LAYER).await?;
    d.drag(LAYER, 0.0, plot.height * 0.4).await?;
    eventually(d, "a brushed range", async |d| {
        let range = d.text("#range").await?;
        Ok(!range.is_empty() && !range.starts_with("Jan"))
    })
    .await?;
    d.click("#reset").await?;
    says(d, "Showing Jan to Jun", "the reset").await
}

e2e::scenario!(
    lying_bars_zoom_and_brush_down_their_side,
    "/bar-chart/zoom",
    zooms_down
);
