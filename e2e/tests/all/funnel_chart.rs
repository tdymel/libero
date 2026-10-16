//! `FunnelChart`: its stages narrowing top down with shares of the first on them; the
//! pointer, the keys and the legend over the stages. Web and Android: charts are parked
//! on Blitz.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const LAYER: &str = "#chart [data-chart-layer]";
const TOOLTIP: &str = "#chart [data-slot=tooltip]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("funnel_chart", "/funnel-chart").run();
}

/// Each stage is narrower than the one above it and carries its share of the first.
#[test]
fn the_stages_narrow_and_carry_shares_of_the_first() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/funnel-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg polygon")
            .await
            .unwrap();
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg text')].map(t => t.textContent.trim())",
        )
        .await;
        assert_eq!(labels, ["100%", "40%", "10%"]);
        let boxes: Vec<[f64; 2]> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg polygon')]\
             .map(p => { const b = p.getBBox(); return [b.y, b.width]; })",
        )
        .await;
        assert!(
            boxes
                .windows(2)
                .all(|pair| pair[0][0] < pair[1][0] && pair[0][1] > pair[1][1]),
            "{boxes:?}"
        );
        let summary: String = e2e::js(
            &fixture.page,
            "document.querySelector('#chart [data-chart-summary]').textContent",
        )
        .await;
        assert!(
            summary.contains("Funnel chart from Visits to Orders.")
                && summary.contains("Carts: 400, 40% of the first."),
            "{summary}"
        );
        fixture.close().await.unwrap();
    });
}

/// In a right-to-left page the labels stay on their stages, as drawn.
#[test]
fn the_labels_keep_their_place_right_to_left() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/funnel-chart", Viewport::Desktop)
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

async fn reads<D: Driver>(d: &mut D, id: &str, want: &str, what: &str) -> Result<()> {
    let label = format!("{what}: {id} to read {want:?}");
    eventually(d, &label, async |d| Ok(d.text(id).await? == want)).await
}

/// Every arrow walks the stages top down; the keys' stage is announced with its share of
/// the first.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "Visits 0 1000", "the first ArrowRight").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "Carts 1 400", "ArrowDown").await?;
    eventually(d, "the announcement", async |d| {
        Ok(d.text("#chart [role=status]").await? == "Carts: 400, 40% of the first.")
    })
    .await?;
    eventually(d, "the tooltip", async |d| {
        Ok(d.exists(TOOLTIP).await? && d.text(TOOLTIP).await?.contains("400 (40%)"))
    })
    .await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "Orders 2 100", "End").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    reads(d, "#active", "Carts 1 400", "ArrowLeft").await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#clicked", "Carts 1 400", "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#active", "", "Escape").await?;
    ensure!(!d.exists(TOOLTIP).await?, "Escape left the tooltip");
    Ok(())
}

e2e::scenario!(the_keys_walk_the_stages, "/funnel-chart", keys);

/// A click near the top middle lands on the first stage. A tap pins it, a second tap
/// clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let plot = d.rect(LAYER).await?;
    let (x, y) = (plot.x + plot.width / 2.0, plot.y + plot.height * 0.15);
    d.click_at(x, y).await?;
    if d.platform().is_touch() {
        reads(d, "#active", "Visits 0 1000", "a tap").await?;
        ensure!(
            d.text("#clicked").await?.is_empty(),
            "the first tap clicked"
        );
        d.click_at(x, y).await?;
    }
    reads(d, "#clicked", "Visits 0 1000", "a click on Visits").await
}

e2e::scenario!(a_click_on_a_stage_reports_it, "/funnel-chart", clicks);

/// A legend button takes its stage out of the funnel; the shares stay of the first stage.
async fn legend<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let visits = "#chart [data-slot=legend-item]:nth-child(1) button";
    d.click(visits).await?;
    reads(d, "#hidden", "Visits", "a click on Visits").await?;
    eventually(d, "two stages left", async |d| {
        let polygon = |n: usize| format!("#chart [data-chart-drawing] polygon:nth-of-type({n})");
        Ok(d.exists(&polygon(2)).await? && !d.exists(&polygon(3)).await?)
    })
    .await?;
    let drawing = d.text("#chart [data-chart-drawing]").await?;
    ensure!(
        drawing.contains("40%") && !drawing.contains("100%"),
        "{drawing}"
    );
    let pressed = d.attr(visits, "aria-pressed").await?;
    ensure!(pressed.as_deref() == Some("false"), "pressed {pressed:?}");
    d.click(visits).await?;
    reads(d, "#hidden", "", "a second click on Visits").await
}

e2e::scenario!(the_legend_hides_and_shows_a_stage, "/funnel-chart", legend);
