//! `HeatmapChart`: its cells with their values, an empty cell and the scale; the pointer
//! and the keys over the grid. Web and Android: charts are parked on Blitz.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const LAYER: &str = "#chart [data-chart-layer]";
const TOOLTIP: &str = "#chart [data-slot=tooltip]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("heatmap_chart", "/heatmap-chart").run();
}

/// Each cell writes its value; the empty one is outlined and keyed under the scale; the
/// summary reads every row.
#[test]
fn the_cells_write_their_values_and_the_scale_keys_an_empty_one() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/heatmap-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg rect")
            .await
            .unwrap();
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart [data-chart-drawing] text')].map(t => t.textContent.trim())",
        )
        .await;
        for value in ["10", "40", "25", "5", "50", "No value"] {
            assert!(labels.contains(&value.to_string()), "{labels:?}");
        }
        let dashed: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart [data-chart-drawing] rect[stroke-dasharray]').length",
        )
        .await;
        assert_eq!(dashed, 2, "the empty cell and its key");
        let summary: String = e2e::js(
            &fixture.page,
            "document.querySelector('#chart [data-chart-summary]').textContent",
        )
        .await;
        assert!(
            summary.contains("Heatmap of 2 rows by 3 columns, values 5 to 50.")
                && summary.contains("Highest: Tue, 3 pm: 50. Lowest: Tue, 9 am: 5.")
                && summary.contains("Tue: 9 am 5, 12 pm no value, 3 pm 50."),
            "{summary}"
        );
        fixture.close().await.unwrap();
    });
}

async fn reads<D: Driver>(d: &mut D, id: &str, want: &str, what: &str) -> Result<()> {
    let label = format!("{what}: {id} to read {want:?}");
    eventually(d, &label, async |d| Ok(d.text(id).await? == want)).await
}

/// Left and right walk a row, up and down the rows; a cell is announced and tipped alone.
async fn keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LAYER).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "Mon 0 10", "the first ArrowRight").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    reads(d, "#active", "Mon 1 40", "ArrowRight").await?;
    d.press(keyboard::END).await?;
    reads(d, "#active", "Mon 2 25", "End").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    reads(d, "#active", "Tue 2 50", "ArrowDown").await?;
    eventually(d, "the announcement", async |d| {
        Ok(d.text("#chart [role=status]").await? == "Tue, 3 pm: 50")
    })
    .await?;
    eventually(d, "the tooltip", async |d| {
        if !d.exists(TOOLTIP).await? {
            return Ok(false);
        }
        let tip = d.text(TOOLTIP).await?;
        Ok(tip.contains("3 pm") && tip.contains("50") && !tip.contains("25"))
    })
    .await?;
    d.press(keyboard::ARROW_LEFT).await?;
    reads(d, "#active", "Tue 0 5", "ArrowLeft over the empty cell").await?;
    d.press(keyboard::ARROW_UP).await?;
    reads(d, "#active", "Mon 0 10", "ArrowUp").await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#clicked", "Mon 0 10", "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    reads(d, "#active", "", "Escape").await?;
    ensure!(!d.exists(TOOLTIP).await?, "Escape left the tooltip");
    Ok(())
}

e2e::scenario!(the_keys_walk_the_grid, "/heatmap-chart", keys);

/// A click on a cell reports it. A tap pins it, a second tap clicks.
async fn clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // The renderer draws the bottom row first: the first cell is Tue's at 9 am.
    let cell = d.rect("#chart [data-chart-drawing] svg > rect").await?;
    let (x, y) = (cell.x + cell.width / 2.0, cell.y + cell.height / 2.0);
    d.click_at(x, y).await?;
    if d.platform().is_touch() {
        reads(d, "#active", "Tue 0 5", "a tap").await?;
        ensure!(
            d.text("#clicked").await?.is_empty(),
            "the first tap clicked"
        );
        d.click_at(x, y).await?;
    }
    reads(d, "#clicked", "Tue 0 5", "a click on Tue at 9 am").await
}

e2e::scenario!(
    a_click_on_a_cell_reports_it,
    "/heatmap-chart",
    clicks,
    native: skip("a click on a chart mark keeps the vdom busy past 200 polls on Blitz")
);
