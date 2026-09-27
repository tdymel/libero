//! `Table` master-detail (1156-4a): a toggle opens a full-width detail row under
//! its row, which follows the row through a sort; the toggle is no row click.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text, linger};
use e2e::passes::keyboard;

const CHERRY: &str = "button[aria-label=\"Details for Cherry\"]";
const DETAIL: &str = "tbody tr[data-detail]";

async fn opens_under_its_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.exists("button[aria-label=\"Details for Banana\"]")
        .await?
    {
        bail!("a row without a detail has a toggle");
    }
    d.click(CHERRY).await?;
    eventually_text(d, "#expanded", "Cherry", "a toggle click").await?;
    eventually(d, "the detail row", async |d| d.exists(DETAIL).await).await?;
    let id = d.attr(DETAIL, "id").await?;
    if d.attr(CHERRY, "aria-controls").await? != id {
        bail!("the toggle does not point at its detail row {id:?}");
    }
    if d.attr(CHERRY, "aria-expanded").await?.as_deref() != Some("true") {
        bail!("the open toggle is not expanded");
    }
    // Right under its row, as wide as the table.
    let row = d.rect("tbody tr:first-child th").await?;
    let detail = d.rect(&format!("{DETAIL} td")).await?;
    let table = d.rect("table").await?;
    if (detail.y - (row.y + row.height)).abs() > 2.0 || detail.width < table.width - 2.0 {
        bail!("the detail is not a full-width row under Cherry: {detail:?}, {row:?}, {table:?}");
    }
    linger(d, 5).await;
    if !d.text("#clicked").await?.is_empty() {
        bail!("the toggle also clicked its row");
    }
    d.click(CHERRY).await?;
    eventually(d, "the detail row to go", async |d| {
        Ok(!d.exists(DETAIL).await?)
    })
    .await
}

e2e::scenario!(
    a_toggle_opens_the_detail_under_its_row,
    "/table-detail",
    opens_under_its_row
);

async fn follows_the_sort<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(CHERRY).await?;
    eventually_text(d, "#expanded", "Cherry", "a toggle click").await?;
    // Name ascending: Apple, Banana, Cherry, then Cherry's detail.
    d.click("th:nth-child(2) [data-sort-button]").await?;
    eventually_text(d, "tbody tr:first-child th", "Apple", "the sort").await?;
    eventually_text(d, "tbody tr:nth-child(3) th", "Cherry", "the sort").await?;
    if d.attr("tbody tr:nth-child(4)", "data-detail")
        .await?
        .is_none()
    {
        bail!("the detail did not follow Cherry");
    }
    // Stripes count rows, not the detail.
    if d.attr("tbody tr:nth-child(2)", "data-stripe")
        .await?
        .is_none()
    {
        bail!("Banana, the second row, is not striped");
    }
    Ok(())
}

e2e::scenario!(
    an_open_detail_follows_its_row_through_a_sort,
    "/table-detail",
    follows_the_sort
);

/// Enter opens, Tab goes into the detail, which follows its toggle.
async fn keyboard_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(CHERRY).await?;
    eventually_focused(d, CHERRY, "focus").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#expanded", "Cherry", "Enter").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "[data-order=\"Cherry\"]", "Tab").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, CHERRY, "Shift+Tab").await?;
    d.press(keyboard::SPACE).await?;
    eventually(d, "Space to close the detail", async |d| {
        Ok(!d.exists(DETAIL).await?)
    })
    .await
}

e2e::scenario!(
    the_keyboard_opens_a_detail_and_tabs_into_it,
    "/table-detail",
    keyboard_opens,
    android: skip("958: element identity on the WebView")
);
