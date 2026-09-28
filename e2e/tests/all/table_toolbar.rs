//! `Table` toolbar pieces (1416): Export hands over every page's rows in the
//! shown columns, Columns hides one, Density sets the row height.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_text};
use e2e::passes::keyboard;

/// Clicks the open menu's entry reading `label`.
async fn pick<D: Driver>(d: &mut D, label: &str) -> Result<()> {
    for index in 0..12 {
        let item = format!("[role^=menuitem][data-menu-index=\"{index}\"]");
        if d.exists(&item).await? && d.text(&item).await?.trim() == label {
            return d.click(&item).await;
        }
    }
    bail!("no {label} entry")
}

async fn open<D: Driver>(d: &mut D, tool: &str) -> Result<()> {
    d.click(&format!("[data-table-tool={tool}]")).await?;
    eventually(d, "the menu to open", async |d| {
        d.exists("[role^=menuitem]").await
    })
    .await
}

async fn the_pieces_work<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("[data-table-tool=export]").await?;
    eventually_text(
        d,
        "#csv",
        "Name,Stock|Apple,12|Banana,0|Cherry,3|",
        "an export over both pages",
    )
    .await?;

    open(d, "columns").await?;
    pick(d, "Stock").await?;
    eventually(d, "Stock to hide", async |d| {
        Ok(d.text("thead").await?.contains("Name") && !d.text("thead").await?.contains("Stock"))
    })
    .await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the menu to close", async |d| {
        Ok(!d.exists("[role^=menuitem]").await?)
    })
    .await?;
    d.click("[data-table-tool=export]").await?;
    eventually_text(
        d,
        "#csv",
        "Name|Apple|Banana|Cherry|",
        "an export without Stock",
    )
    .await?;

    let before = d.rect("tbody tr").await?.height;
    open(d, "density").await?;
    pick(d, "Comfortable").await?;
    eventually_text(d, "#density", "Lg", "a density pick").await?;
    eventually(d, "taller rows", async |d| {
        Ok(d.rect("tbody tr").await?.height > before + 2.0)
    })
    .await
}

e2e::scenario!(
    the_toolbar_pieces_export_hide_columns_and_set_the_density,
    "/table-toolbar",
    the_pieces_work
);
