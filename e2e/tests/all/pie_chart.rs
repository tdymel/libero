//! `PieChart`: its shares beside the slices, a donut's centre in the ring's middle; the
//! pointer, the keys and the legend over the slices. Web and Android: charts are parked
//! on Blitz.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const LAYER: &str = "#chart [data-chart-layer]";
const TOOLTIP: &str = "#chart [data-slot=tooltip]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("pie_chart", "/pie-chart").run();
}

#[test]
fn the_slices_carry_their_shares() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg path")
            .await
            .unwrap();
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg text')].map(t => t.textContent.trim())",
        )
        .await;
        assert_eq!(labels, ["60%", "25%", "15%"]);
        let summary: String = e2e::js(
            &fixture.page,
            "document.querySelector('#chart [data-chart-summary]').textContent",
        )
        .await;
        assert!(
            ["60%", "25%", "15%"]
                .iter()
                .all(|share| summary.contains(share)),
            "{summary}"
        );
        fixture.close().await.unwrap();
    });
}

/// A plot that narrows after its first measure is drawn again at its new width, so its text
/// keeps its size.
#[test]
fn a_narrowed_plot_is_drawn_again_at_its_width() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#chart svg')?.getAttribute('viewBox')?.startsWith('0 0 480 ')",
            "the pie drawn at 480 px",
        )
        .await
        .unwrap();
        e2e::js::<String>(
            &fixture.page,
            "document.querySelector('#chart').parentElement.style.maxWidth = '320px'",
        )
        .await;
        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#chart svg')?.getAttribute('viewBox')?.startsWith('0 0 320 ')",
            "the pie drawn again at 320 px",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A loading chart in a flex row keeps the row's width, its placeholder the plot's size,
/// rather than shrink to its title.
#[test]
fn a_loading_chart_keeps_its_width_in_a_flex_row() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart/loading", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#chart [data-slot=plot]')?.getBoundingClientRect().width === 480",
            "the loading plot 480 px wide",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// On a narrow chart a side legend leaves the plot its width: the end's goes under the
/// plot, the start's over it, as the DOM and Tab order them.
#[test]
fn a_narrow_chart_stacks_its_side_legend() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart/narrow", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#start svg path")
            .await
            .unwrap();
        const PLACES: &str = "['#chart', '#start'].map(id => { \
             const plot = document.querySelector(id + ' [data-slot=plot]').getBoundingClientRect(); \
             const legend = document.querySelector(id + ' [data-slot=legend]').getBoundingClientRect(); \
             return [plot.width, legend.top >= plot.bottom, legend.bottom <= plot.top]; })";
        let places: Vec<(f64, bool, bool)> = e2e::js(&fixture.page, PLACES).await;
        assert_eq!(places, [(280.0, true, false), (280.0, false, true)]);
        fixture.close().await.unwrap();
    });
}

/// In a right-to-left page the labels stay where they are drawn left to right, beside
/// their slices rather than flipped over them.
#[test]
fn the_labels_keep_their_place_right_to_left() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg text")
            .await
            .unwrap();
        const BOXES: &str = "[...document.querySelectorAll('#chart svg text')]\
             .map(t => { const b = t.getBBox(); return [b.x, b.width]; })";
        let ltr: Vec<[f64; 2]> = e2e::js(&fixture.page, BOXES).await;
        let rtl: Vec<[f64; 2]> = e2e::js(
            &fixture.page,
            &format!("(() => {{ document.documentElement.dir = 'rtl'; return {BOXES}; }})()"),
        )
        .await;
        assert_eq!(ltr, rtl);
        fixture.close().await.unwrap();
    });
}

/// The centre sits over the ring's hole, which is the plot's middle.
#[test]
fn a_donut_s_center_sits_in_its_middle() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart/donut", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#center").await.unwrap();
        let [dx, dy]: [f64; 2] = e2e::js(
            &fixture.page,
            "(() => { const mid = r => [r.x + r.width / 2, r.y + r.height / 2]; \
             const [cx, cy] = mid(document.querySelector('#center').getBoundingClientRect()); \
             const [px, py] = mid(document.querySelector('#chart svg').getBoundingClientRect()); \
             return [cx - px, cy - py]; })()",
        )
        .await;
        assert!(dx.abs() < 2.0 && dy.abs() < 2.0, "{dx} {dy}");
        // The centre's circle keeps a gap to the ring: the hole is the slices' inner arc.
        let [half, hole]: [f64; 2] = e2e::js(
            &fixture.page,
            "(() => { const svg = document.querySelector('#chart svg'); \
             const scale = svg.getBoundingClientRect().width / svg.viewBox.baseVal.width; \
             const radii = [...svg.querySelectorAll('path')].flatMap(p => \
               [...p.getAttribute('d').matchAll(/A([\\d.]+) /g)].map(m => +m[1])).filter(r => r > 0); \
             const slot = document.querySelector('#chart [data-slot=center]').getBoundingClientRect(); \
             return [Math.max(slot.width, slot.height) / 2, Math.min(...radii) * scale]; })()",
        )
        .await;
        assert!(
            half > 0.0 && half <= hole - 3.0,
            "centre {half} in a hole of {hole}"
        );
        let texts: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart svg text').length",
        )
        .await;
        assert_eq!(texts, 0);
        fixture.close().await.unwrap();
    });
}

/// A half donut's centre sits in the half of the hole under the arc; its labels are on it.
#[test]
fn a_gauge_s_center_sits_under_its_arc() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart/gauge", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#center").await.unwrap();
        // The first slice starts at 9 o'clock on the hole: its middle is a hole right of it.
        let [cx, cy, hole, slot_x, slot_y, half]: [f64; 6] = e2e::js(
            &fixture.page,
            "(() => { const svg = document.querySelector('#chart svg'); \
             const box = svg.getBoundingClientRect(); \
             const scale = box.width / svg.viewBox.baseVal.width; \
             const d = svg.querySelector('path').getAttribute('d'); \
             const [x, y] = d.slice(1).split(' ')[0].split(',').map(Number); \
             const hole = Math.min(...[...d.matchAll(/A([\\d.]+) /g)].map(m => +m[1]).filter(r => r > 0)); \
             const slot = document.querySelector('#chart [data-slot=center]').getBoundingClientRect(); \
             return [box.x + (x + hole) * scale, box.y + y * scale, hole * scale, \
               slot.x + slot.width / 2, slot.y + slot.height / 2, slot.width / 2]; })()",
        )
        .await;
        assert!(
            (slot_x - cx).abs() < 2.0 && (slot_y - (cy - hole / 2.0)).abs() < 2.0,
            "centre at {slot_x},{slot_y}, middle {cx},{cy}, hole {hole}"
        );
        assert!(half > 0.0 && half <= hole / 2.0, "{half} in {hole}");
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg text')].map(t => t.textContent.trim())",
        )
        .await;
        assert_eq!(labels, ["60%", "25%", "15%"]);
        fixture.close().await.unwrap();
    });
}

/// Each slice's corners are filleted arcs of one radius, none of the renderer's knobs (3479).
#[test]
fn a_slice_radius_rounds_each_corner() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/pie-chart/rounded", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#ring svg path")
            .await
            .unwrap();
        for (chart, fillets) in [("chart", 2), ("ring", 4)] {
            let corners: Vec<usize> = e2e::js(
                &fixture.page,
                &format!(
                    "[...document.querySelectorAll('#{chart} [data-chart-drawing] path[fill]:not([fill=none])')].map(p => \
                     [...p.getAttribute('d').matchAll(/A([\\d.]+),/g)].filter(m => +m[1] > 0 && +m[1] < 30).length)"
                ),
            )
            .await;
            assert_eq!(corners, [fillets; 3], "{chart}");
            let knobs: usize = e2e::js(
                &fixture.page,
                &format!(
                    "[...document.querySelectorAll('#{chart} svg path')] \
                     .filter(p => p.getAttribute('d').includes('A0 0')).length"
                ),
            )
            .await;
            assert_eq!(knobs, 0, "{chart}");
        }
        fixture.close().await.unwrap();
    });
}

async fn reads<D: Driver>(d: &mut D, id: &str, want: &str, what: &str) -> Result<()> {
    let label = format!("{what}: {id} to read {want:?}");
    eventually(d, &label, async |d| Ok(d.text(id).await? == want)).await
}

/// Every arrow walks the slices clockwise; the keys' slice is announced with its share.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "Chrome 0 60", "the first ArrowRight").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "Safari 1 25", "ArrowDown").await?;
    eventually(d, "the announcement", async |d| {
        Ok(d.text("#chart [role=status]").await? == "Safari: 25, 25%.")
    })
    .await?;
    eventually(d, "the tooltip", async |d| {
        Ok(d.exists(TOOLTIP).await? && d.text(TOOLTIP).await?.contains("25 (25%)"))
    })
    .await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "Other 2 15", "End").await?;
    d.press(keyboard::ARROW_UP).await?;
    reads(d, "#active", "Safari 1 25", "ArrowUp").await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#clicked", "Safari 1 25", "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#active", "", "Escape").await?;
    ensure!(!d.exists(TOOLTIP).await?, "Escape left the tooltip");
    Ok(())
}

e2e::scenario!(the_keys_walk_the_slices, "/pie-chart", keys);

/// A click right of the middle lands on Chrome, from 12 to past 6 o'clock. A tap pins
/// it, a second tap on it clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let plot = d.rect(LAYER).await?;
    let (x, y) = (
        plot.x + plot.width / 2.0 + plot.height * 0.2,
        plot.y + plot.height / 2.0,
    );
    d.click_at(x, y).await?;
    if d.platform().is_touch() {
        reads(d, "#active", "Chrome 0 60", "a tap").await?;
        ensure!(
            d.text("#clicked").await?.is_empty(),
            "the first tap clicked"
        );
        d.click_at(x, y).await?;
    }
    reads(d, "#clicked", "Chrome 0 60", "a click on Chrome").await?;
    ensure!(
        d.exists("#chart [data-chart-marker]").await?,
        "no active marker"
    );
    Ok(())
}

e2e::scenario!(a_click_on_a_slice_reports_it, "/pie-chart", clicks);

/// With a slice hidden, the tooltip and the keys' announcement give the share the labels
/// show (of the shown), the summary the share of all (3429, documented).
async fn hidden_share<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#chart [data-slot=legend-item]:nth-child(3) button")
        .await?;
    reads(d, "#hidden", "Other", "a click on Other").await?;
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "the announcement", async |d| {
        Ok(d.text("#chart [role=status]").await? == "Chrome: 60, 70.6%.")
    })
    .await?;
    eventually(d, "the tooltip", async |d| {
        Ok(d.exists(TOOLTIP).await? && d.text(TOOLTIP).await?.contains("60 (70.6%)"))
    })
    .await?;
    let summary = d.text("#chart [data-chart-summary]").await?;
    ensure!(
        summary.contains("Chrome: 60, 60%.") && summary.contains("Shares count the hidden"),
        "{summary}"
    );
    Ok(())
}

e2e::scenario!(
    a_hidden_slice_grows_the_shares_the_eye_reads_not_the_summary,
    "/pie-chart",
    hidden_share
);

/// A legend button takes its slice out of the pie: the others' shares grow.
async fn legend<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let other = "#chart [data-slot=legend-item]:nth-child(3) button";
    d.click(other).await?;
    reads(d, "#hidden", "Other", "a click on Other").await?;
    eventually(d, "Chrome's share to grow", async |d| {
        Ok(d.text("#chart [data-chart-drawing]")
            .await?
            .contains("70.6%"))
    })
    .await?;
    let pressed = d.attr(other, "aria-pressed").await?;
    ensure!(pressed.as_deref() == Some("false"), "pressed {pressed:?}");
    d.click(other).await?;
    reads(d, "#hidden", "", "a second click on Other").await
}

e2e::scenario!(the_legend_hides_and_shows_a_slice, "/pie-chart", legend);
