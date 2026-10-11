//! `LineChart`: the pointer, the keys and the legend over the drawn lines. Web and
//! Android: charts are parked on Blitz.

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::keyboard;

const LAYER: &str = "#chart [data-chart-layer]";
const TOOLTIP: &str = "#chart [data-slot=tooltip]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("line_chart", "/line-chart").run();
}

async fn reads<D: Driver>(d: &mut D, id: &str, want: &str, what: &str) -> Result<()> {
    let label = format!("{what}: {id} to read {want:?}");
    eventually(d, &label, async |d| Ok(d.text(id).await? == want)).await
}

/// The pointer over the plot's centre picks Wed, where only "This week" has a point.
async fn hovers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let plot = d.rect(LAYER).await?;
    ensure!(!d.exists(TOOLTIP).await?, "a tooltip before any hover");
    d.hover(LAYER).await?;
    reads(d, "#active", "This week Wed 101", "a hover over the gap").await?;
    eventually(d, "the tooltip", async |d| d.exists(TOOLTIP).await).await?;
    let tip = d.text(TOOLTIP).await?;
    ensure!(
        tip.contains("Wed") && tip.contains("101") && !tip.contains("Last week"),
        "tooltip {tip:?}"
    );
    let tip_box = d.rect(TOOLTIP).await?;
    ensure!(
        tip_box.x >= plot.x - 0.5 && tip_box.x + tip_box.width <= plot.x + plot.width + 0.5,
        "the tooltip left the plot: {tip_box:?} in {plot:?}"
    );
    ensure!(
        d.exists("#chart [data-chart-dot]").await?,
        "no active marker"
    );
    let crosshair = d.rect("#chart [data-chart-crosshair]").await?;
    // It spans the plot, not the x axis' labels under it.
    let labels = d.rects("#chart [data-chart-drawing] text").await?;
    let axis_labels = labels.iter().map(|t| t.y).fold(f64::MIN, f64::max);
    ensure!(
        crosshair.y > plot.y + 0.5 && crosshair.y + crosshair.height <= axis_labels,
        "the crosshair left the plot: {crosshair:?}, x labels at {axis_labels}"
    );
    Ok(())
}

e2e::scenario!(
    hovering_shows_the_point_s_tooltip_and_marker,
    "/line-chart",
    hovers,
    android: skip("a touch screen has no hover"),
    native: skip("charts are parked on Blitz")
);

/// Arrows walk a line and cross to the other at the same x; Enter clicks; Escape clears.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "This week Mon 120", "the first ArrowRight").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "This week Tue 132", "ArrowRight").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "Last week Tue 30", "ArrowDown").await?;
    // "Last week" has no point at Wed.
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "Last week Thu 25", "ArrowRight over the gap").await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "Last week Fri 40", "End").await?;
    d.press(keyboard::ARROW_UP).await?;
    reads(d, "#active", "This week Fri 90", "ArrowUp").await?;
    eventually(d, "the announcement", async |d| {
        Ok(d.text("#chart [role=status]").await? == "This week, Fri: 90")
    })
    .await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#clicked", "This week Fri 90", "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#active", "", "Escape").await?;
    ensure!(!d.exists(TOOLTIP).await?, "Escape left the tooltip");
    Ok(())
}

e2e::scenario!(
    the_keys_walk_the_points,
    "/line-chart",
    keys,
    native: skip("charts are parked on Blitz")
);

/// A click picks the point under it. A tap pins it, a second tap on it clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let plot = d.rect(LAYER).await?;
    let (x, y) = (plot.x + plot.width * 0.01, plot.y + plot.height * 0.05);
    d.click_at(x, y).await?;
    if d.platform() == Platform::Android {
        reads(d, "#active", "This week Mon 120", "a tap").await?;
        ensure!(
            d.text("#clicked").await?.is_empty(),
            "the first tap clicked"
        );
        d.click_at(x, y).await?;
    }
    reads(d, "#clicked", "This week Mon 120", "a click on Mon").await
}

e2e::scenario!(
    a_click_on_a_point_reports_it,
    "/line-chart",
    clicks,
    native: skip("charts are parked on Blitz")
);

/// A legend button hides its line from the drawing; the last shown one stays.
async fn legend<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let this_week = "#chart [data-slot=legend-item]:nth-child(1) button";
    let last_week = "#chart [data-slot=legend-item]:nth-child(2) button";
    ensure!(
        d.exists("#chart svg [stroke-dasharray]").await?,
        "the dashed line is not drawn"
    );
    d.click(last_week).await?;
    reads(d, "#hidden", "Last week", "a click on Last week").await?;
    eventually(d, "the dashed line to leave", async |d| {
        Ok(!d.exists("#chart svg [stroke-dasharray]").await?)
    })
    .await?;
    let pressed = d.attr(last_week, "aria-pressed").await?;
    ensure!(pressed.as_deref() == Some("false"), "pressed {pressed:?}");
    let held = d.attr(this_week, "aria-disabled").await?;
    ensure!(held.as_deref() == Some("true"), "the last line is not held");
    d.click(this_week).await?;
    d.click(last_week).await?;
    reads(d, "#hidden", "", "a second click on Last week").await
}

e2e::scenario!(
    the_legend_hides_and_shows_a_line,
    "/line-chart",
    legend,
    native: skip("charts are parked on Blitz")
);

/// `[background, text fills, series strokes]` of the drawing, and `Canvas` and `CanvasText`.
const INK: &str = "(() => { const probe = document.createElement('i'); document.body.append(probe); \
     probe.style.cssText = 'color: CanvasText; background: Canvas'; const p = getComputedStyle(probe); \
     const system = [p.backgroundColor, p.color]; probe.remove(); \
     const drawing = document.querySelector('#chart [data-chart-drawing]'); \
     const fills = [...drawing.querySelectorAll('svg text')].map(t => getComputedStyle(t).fill); \
     const strokes = [...drawing.querySelectorAll('svg [stroke^=\"#\"]')].map(s => getComputedStyle(s).stroke); \
     return JSON.stringify([getComputedStyle(drawing).backgroundColor, fills, strokes, system, drawing.innerHTML.includes('#0201')]); })()";

/// Forced colours put the drawing on `Canvas` with `CanvasText` labels; the lines keep their hex.
#[test]
fn forced_colours_draw_the_ink_in_system_colours() {
    use e2e::browser::{block_on, force_colours};
    use e2e::{Fixture, Viewport, wait};
    block_on(async {
        let fixture = Fixture::open("/line-chart", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        force_colours(page).await.unwrap();
        // The chart hears the media change and redraws a moment later.
        wait::for_js_true(
            page,
            "!!document.querySelector('#chart [data-chart-drawing] svg')?.outerHTML.includes('CanvasText')",
            "the drawing redrawn in system colours",
        )
        .await
        .unwrap();
        let ink: String = page.evaluate(INK).await.unwrap().into_value().unwrap();
        let (background, fills, strokes, [canvas, text], stand_in): (
            String,
            Vec<String>,
            Vec<String>,
            [String; 2],
            bool,
        ) = serde_json::from_str(&ink).unwrap();
        assert_eq!(background, canvas, "the drawing's background");
        assert!(
            !fills.is_empty() && fills.iter().all(|fill| *fill == text),
            "label fills {fills:?}, CanvasText {text}"
        );
        assert!(
            !strokes.is_empty()
                && strokes
                    .iter()
                    .all(|stroke| *stroke != text && *stroke != canvas),
            "series strokes {strokes:?}"
        );
        assert!(!stand_in, "a stand-in ink colour is left in the drawing");
        fixture.close().await.unwrap();
    });
}

/// A vertical swipe over the plot scrolls the page: the plot takes only sideways moves.
async fn swipes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.drag(LAYER, 0.0, -120.0).await?;
    eventually(d, "the swipe to scroll the page", async |d| {
        Ok(d.scroll_pos("html").await?.1 > 0.0)
    })
    .await
}

e2e::scenario!(
    a_vertical_swipe_over_the_plot_scrolls_the_page,
    "/line-chart",
    swipes,
    web: skip("a mouse drag scrolls nothing"),
    desktop: skip("a mouse drag scrolls nothing"),
    native: skip("charts are parked on Blitz")
);
