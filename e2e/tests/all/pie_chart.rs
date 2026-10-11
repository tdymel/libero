//! `PieChart`: its shares beside the slices, a donut's centre in the ring's middle; the
//! pointer, the keys and the legend over the slices. Web and Android: charts are parked
//! on Blitz.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, Platform, eventually};
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
        let shares: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart tbody td:last-child')].map(t => t.textContent)",
        )
        .await;
        assert_eq!(shares, ["60%", "25%", "15%"]);
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
        let texts: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart svg text').length",
        )
        .await;
        assert_eq!(texts, 0);
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

e2e::scenario!(
    the_keys_walk_the_slices,
    "/pie-chart",
    keys,
    native: skip("charts are parked on Blitz")
);

/// A click right of the middle lands on Chrome, from 12 to past 6 o'clock. A tap pins
/// it, a second tap on it clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let plot = d.rect(LAYER).await?;
    let (x, y) = (
        plot.x + plot.width / 2.0 + plot.height * 0.2,
        plot.y + plot.height / 2.0,
    );
    d.click_at(x, y).await?;
    if d.platform() == Platform::Android {
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

e2e::scenario!(
    a_click_on_a_slice_reports_it,
    "/pie-chart",
    clicks,
    native: skip("charts are parked on Blitz")
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

e2e::scenario!(
    the_legend_hides_and_shows_a_slice,
    "/pie-chart",
    legend,
    native: skip("charts are parked on Blitz")
);
