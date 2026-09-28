//! `Table` loading, empty states and toolbar (1156-3d): a bar over the kept
//! rows, skeleton rows without any, and the toolbar's quick filter at the end.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_text};

const TABLE: &str = "table[aria-label=Fruit]";

async fn loading_overlays<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let top = d.rect(TABLE).await?.y;
    d.click("#toggle-loading").await?;
    eventually(d, "the loading bar", async |d| {
        d.exists("[data-loading-bar] [role=progressbar]").await
    })
    .await?;
    // Over the table's top edge: the table did not move.
    let bar = d.rect("[data-loading-bar] [role=progressbar]").await?;
    let moved = d.rect(TABLE).await?.y;
    if (moved - top).abs() > 1.0 || (bar.y - top).abs() > 2.0 {
        bail!("table top {top} -> {moved}, bar top {}", bar.y);
    }
    if d.attr(TABLE, "aria-busy").await?.is_some() {
        bail!("busy while its rows are still shown");
    }
    eventually_text(d, "tbody tr:first-child th", "Cherry", "the kept rows").await?;

    d.click("#toggle-rows").await?;
    eventually(d, "skeleton rows", async |d| {
        d.exists("tbody tr[data-skeleton]").await
    })
    .await?;
    if d.exists("[data-loading-bar]").await? {
        bail!("the bar stays over skeleton rows");
    }
    if d.attr(TABLE, "aria-busy").await?.as_deref() != Some("true") {
        bail!("not busy while only placeholders show");
    }

    d.click("#toggle-loading").await?;
    eventually_text(d, "tbody tr[data-empty]", "No rows", "the no-rows row").await
}

e2e::scenario!(
    a_loading_table_shows_a_bar_over_its_rows_or_skeleton_rows,
    "/table-overlay",
    loading_overlays
);

async fn toolbar_layout<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let toolbar = d.rect("[data-toolbar]").await?;
    let first = d.rect("#toggle-loading").await?;
    let search = d.rect("[data-toolbar-end]").await?;
    let (start, end) = (toolbar.x, toolbar.x + toolbar.width);
    if (first.x - start).abs() > 2.0 || (search.x + search.width - end).abs() > 2.0 {
        bail!(
            "toolbar {start}..{end}: button at {}, search ends at {}",
            first.x,
            search.x + search.width
        );
    }
    d.focus("[data-toolbar-end] input").await?;
    d.type_text("kiwi").await?;
    eventually_text(d, "#no-results", "No fruit matches", "the no-results slot").await
}

e2e::scenario!(
    the_toolbar_leads_with_its_controls_and_ends_with_the_quick_filter,
    "/table-overlay",
    toolbar_layout
);
