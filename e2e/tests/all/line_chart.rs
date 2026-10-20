//! `LineChart`: the pointer, the keys and the legend over the drawn lines. Web and
//! Android: charts are parked on Blitz.

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::keyboard;
use e2e::wait::PageBounded;

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
    native: skip("the crosshair reads a 0x0 rect on Blitz"),
    ios: skip("a touch screen has no hover")
);

/// A pen hovers as a mouse does: its tooltip leaves with it.
async fn pen_leaves<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.evaluate(&format!(
        "(() => {{ const layer = document.querySelector({LAYER:?});
           const r = layer.getBoundingClientRect();
           const pen = (type, bubbles) => layer.dispatchEvent(new PointerEvent(type, {{
               bubbles, pointerId: 7, isPrimary: true, pointerType: 'pen', buttons: 0,
               clientX: r.left + r.width / 2, clientY: r.top + r.height / 2 }}));
           pen('pointermove', true);
           window.penLeave = () => pen('pointerleave', false);
           return true; }})()"
    ))
    .await?;
    reads(d, "#active", "This week Wed 101", "a pen hover").await?;
    eventually(d, "the tooltip", async |d| d.exists(TOOLTIP).await).await?;
    d.evaluate("window.penLeave()").await?;
    reads(d, "#active", "", "the pen leaving").await?;
    eventually(d, "the tooltip to leave", async |d| {
        Ok(!d.exists(TOOLTIP).await?)
    })
    .await
}

e2e::scenario!(
    a_pen_leaving_takes_its_tooltip_along,
    "/line-chart",
    pen_leaves,
    android: skip("a synthetic pen hover is a browser check"),
    native: skip("Blitz has no script reads"),
    ios: skip("a synthetic pen hover is a browser check")
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

e2e::scenario!(the_keys_walk_the_points, "/line-chart", keys);

/// Watches the tooltip every frame: a frame whose box is shorter than its text is counted.
const WATCH_TIP: &str = "(() => { window.tipSqueezed = 0; const watch = () => { \
     const tip = document.querySelector('#chart [data-slot=tooltip]'); \
     if (tip && tip.getBoundingClientRect().height + 1 < tip.scrollHeight) window.tipSqueezed += 1; \
     window.tipWatch = requestAnimationFrame(watch); }; watch(); return true; })()";
const SETTLED: &str =
    "document.querySelector('#chart [data-slot=tooltip]').getAnimations().length === 0";
const SQUEEZED: &str =
    "(() => { cancelAnimationFrame(window.tipWatch); return window.tipSqueezed; })()";

/// Todo 3462: walking point to point, above and below the middle in turn, the tooltip
/// keeps the height of its text in every frame.
async fn tooltip_keeps_its_height<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.evaluate(WATCH_TIP).await?;
    let walk = [
        (keyboard::ARROW_RIGHT, "This week Mon 120"),
        (keyboard::ARROW_DOWN, "Last week Mon 20"),
        (keyboard::ARROW_RIGHT, "Last week Tue 30"),
        (keyboard::ARROW_UP, "This week Tue 132"),
        (keyboard::ARROW_RIGHT, "This week Wed 101"),
        (keyboard::ARROW_RIGHT, "This week Thu 134"),
        (keyboard::ARROW_DOWN, "Last week Thu 25"),
        (keyboard::ARROW_RIGHT, "Last week Fri 40"),
        (keyboard::ARROW_UP, "This week Fri 90"),
    ];
    for (key, want) in walk {
        d.press(key).await?;
        reads(d, "#active", want, "the walk").await?;
    }
    eventually(d, "the tooltip to settle at Fri", async |d| {
        Ok(d.text(TOOLTIP).await?.contains("Fri") && d.evaluate(SETTLED).await? == true)
    })
    .await?;
    let squeezed = d.evaluate(SQUEEZED).await?.as_u64().unwrap_or(u64::MAX);
    ensure!(
        squeezed == 0,
        "{squeezed} frames squeezed the tooltip below its text"
    );
    Ok(())
}

e2e::scenario!(
    the_tooltip_keeps_its_height_from_point_to_point,
    "/line-chart",
    tooltip_keeps_its_height,
    native: skip("Blitz has no script reads")
);

/// A click picks the point under it. A tap pins it, a second tap on it clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let plot = d.rect(LAYER).await?;
    let (x, y) = (plot.x + plot.width * 0.01, plot.y + plot.height * 0.05);
    d.click_at(x, y).await?;
    if d.platform().is_touch() {
        reads(d, "#active", "This week Mon 120", "a tap").await?;
        ensure!(
            d.text("#clicked").await?.is_empty(),
            "the first tap clicked"
        );
        d.click_at(x, y).await?;
    }
    reads(d, "#clicked", "This week Mon 120", "a click on Mon").await
}

e2e::scenario!(a_click_on_a_point_reports_it, "/line-chart", clicks);

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

e2e::scenario!(the_legend_hides_and_shows_a_line, "/line-chart", legend);

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
        let ink: String = page
            .evaluate_bounded(INK)
            .await
            .unwrap()
            .into_value()
            .unwrap();
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
    let plot = d.rect(LAYER).await?;
    let (x, y) = (plot.x + plot.width / 2.0, plot.y + plot.height / 2.0);
    d.swipe_from(x, y, 0.0, -120.0).await?;
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
    native: skip("a swipe over the plot scrolls no page on Blitz")
);

const STATUS: &str = "#chart [role=status]";
const RESET: &str = "#reset";
const PLUS: keyboard::Key = keyboard::Key {
    key: "+",
    code: "Equal",
    vk: 187,
    text: Some("+"),
};
const ZERO: keyboard::Key = keyboard::Key {
    key: "0",
    code: "Digit0",
    vk: 48,
    text: Some("0"),
};

async fn says<D: Driver>(d: &mut D, want: &str, what: &str) -> Result<()> {
    reads(d, STATUS, want, what).await
}

/// Plus zooms in around the middle, the arrows carry the window past its edge, 0 resets;
/// Shift with an arrow brushes, Escape clears the brush.
async fn zoom_keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let held = d.attr(RESET, "aria-disabled").await?;
    ensure!(
        held.as_deref() == Some("true"),
        "a reset before a zoom: {held:?}"
    );
    d.focus(LAYER).await?;
    d.press(PLUS).await?;
    says(d, "Showing Mar to Jun", "+").await?;
    let held = d.attr(RESET, "aria-disabled").await?;
    ensure!(held.as_deref() != Some("true"), "no reset after a zoom");
    d.press(keyboard::HOME).await?;
    reads(d, "#active", "Visits Mar 4", "Home in the window").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    reads(d, "#active", "Visits Feb 5", "ArrowLeft past the window").await?;
    says(d, "Visits, Feb: 5", "the point the window moved to").await?;
    d.press_shift(keyboard::ARROW_RIGHT).await?;
    reads(d, "#range", "1 2 Feb Mar", "Shift+ArrowRight").await?;
    ensure!(
        d.exists("#chart [data-chart-brush]").await?,
        "no brush band"
    );
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#range", "", "Escape").await?;
    ensure!(
        !d.exists("#chart [data-chart-brush]").await?,
        "Escape left the band"
    );
    d.press(ZERO).await?;
    says(d, "Showing Jan to Aug", "0").await
}

e2e::scenario!(the_keys_zoom_pan_and_brush, "/line-chart-zoom", zoom_keys);

/// A sideways drag over the plot brushes a band of x values; the hover and click stay.
async fn brushes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let plot = d.rect(LAYER).await?;
    d.drag(LAYER, -plot.width * 0.4, 0.0).await?;
    eventually(d, "the brushed range", async |d| {
        let range = d.text("#range").await?;
        let ends: Vec<usize> = range
            .split(' ')
            .take(2)
            .filter_map(|n| n.parse().ok())
            .collect();
        Ok(matches!(ends[..], [start, end] if start < end))
    })
    .await?;
    ensure!(
        d.exists("#chart [data-chart-brush]").await?,
        "no brush band"
    );
    ensure!(
        d.text(STATUS).await?.starts_with("Selected "),
        "the brush was not announced"
    );
    Ok(())
}

e2e::scenario!(a_sideways_drag_brushes_a_range, "/line-chart-zoom", brushes);

/// Two fingers spread over the plot zoom it in.
async fn pinches<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let plot = d.rect(LAYER).await?;
    d.pinch(LAYER, plot.width * 0.2, plot.width * 0.8).await?;
    eventually(d, "the reset to wake", async |d| {
        Ok(d.attr(RESET, "aria-disabled").await?.as_deref() != Some("true"))
    })
    .await?;
    // Said once the zoom settles.
    eventually(d, "the zoom to be announced", async |d| {
        Ok(d.text(STATUS).await?.starts_with("Showing "))
    })
    .await
}

e2e::scenario!(
    a_pinch_zooms_the_plot,
    "/line-chart-zoom",
    pinches,
    web: skip("a mouse has no pinch"),
    desktop: skip("a mouse has no pinch"),
    native: skip("Blitz takes no multi-touch input"),
    ios: skip("AXe injects no two-finger pinch")
);

/// The handle reads a titled SVG and a PNG, and saves the SVG for a button of the page's own.
async fn exports<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#read-svg").await?;
    reads(d, "#svg", "true true", "the SVG").await?;
    d.click("#read-png").await?;
    reads(d, "#png", "true", "the PNG").await?;
    // Android hands a saved file to the share sheet, which would cover the page.
    if d.platform() == Platform::Web {
        d.click("#save-svg").await?;
        reads(d, "#saved", "Saved", "the SVG save").await?;
    }
    Ok(())
}

/// The handle says the chart is zoomed and its reset button, the page's own, shows it all again.
async fn resets<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(PLUS).await?;
    eventually(d, "the reset to wake", async |d| {
        Ok(d.attr(RESET, "aria-disabled").await?.as_deref() != Some("true"))
    })
    .await?;
    d.click(RESET).await?;
    says(d, "Showing Jan to Aug", "the reset").await?;
    eventually(d, "the reset to sleep", async |d| {
        Ok(d.attr(RESET, "aria-disabled").await?.as_deref() == Some("true"))
    })
    .await
}

e2e::scenario!(
    the_handle_resets_the_zoom_and_says_it_is_zoomed,
    "/line-chart-zoom",
    resets
);

e2e::scenario!(
    the_chart_exports_as_svg_and_png,
    "/line-chart-zoom",
    exports,
    native: skip("the PNG export never lands on Blitz")
);
