//! `Table` toolbar pieces (1416): Export hands over every page's rows in the
//! shown columns, Columns hides one, Density sets the row height.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;

/// Clicks the open menu's entry reading `label`, once it shows (todo 1502).
async fn pick<D: Driver>(d: &mut D, label: &str) -> Result<()> {
    let mut found = None;
    eventually(d, &format!("the {label} entry"), async |d| {
        for index in 0..12 {
            let item = format!("[role^=menuitem][data-menu-index=\"{index}\"]");
            if d.exists(&item).await? && d.text(&item).await?.trim() == label {
                found = Some(item);
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await?;
    d.click(&found.expect("the wait held")).await
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
    // Todo 1460: the export changes nothing on screen, so it is said.
    eventually_text(
        d,
        "[role=status]",
        "Exported 3 rows",
        "the export announcement",
    )
    .await?;

    open(d, "columns").await?;
    pick(d, "Stock").await?;
    eventually(d, "Stock to hide", async |d| {
        Ok(d.text("thead").await?.contains("Name") && !d.text("thead").await?.contains("Stock"))
    })
    .await?;
    // Todo 1460: the last shown column's checkbox is off and says why.
    const NAME: &str = "[role=menuitemcheckbox][aria-disabled=true]";
    eventually(d, "Name's checkbox to turn off", async |d| {
        d.exists(NAME).await
    })
    .await?;
    let reason = d.attr(NAME, "aria-describedby").await?.unwrap_or_default();
    if d.text(&format!("[id=\"{reason}\"]")).await? != "One column stays shown" {
        bail!("the last column's checkbox is described by {reason:?}, not the reason");
    }
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

    // A cell: Blitz lays out no box for a `tr`.
    const CELL: &str = "tbody tr > *";
    let before = d.rect(CELL).await?.height;
    open(d, "density").await?;
    pick(d, "Comfortable").await?;
    eventually_text(d, "#density", "Lg", "a density pick").await?;
    eventually(d, "taller rows", async |d| {
        Ok(d.rect(CELL).await?.height > before + 2.0)
    })
    .await
}

e2e::scenario!(
    the_toolbar_pieces_export_hide_columns_and_set_the_density,
    "/table-toolbar",
    the_pieces_work
);

/// 1448: the Filters piece sits where the toolbar puts it, opens the panel
/// under itself and takes the focus back on Escape.
async fn the_filter_piece_opens_the_panel<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const BUTTON: &str = "[data-filter-panel-button]";
    let (button, density) = (
        d.rect(BUTTON).await?,
        d.rect("[data-table-tool=density]").await?,
    );
    if button.x <= density.x || button.x >= d.rect("[data-table-tool=export]").await?.x {
        bail!("the Filters button is not between Density and Export");
    }
    d.click(BUTTON).await?;
    eventually(d, "the panel under the button", async |d| {
        Ok(d.exists("[data-filter-panel]").await?
            && d.rect("[data-filter-panel]").await?.y >= button.y + button.height)
    })
    .await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the closed panel", async |d| {
        Ok(!d.exists("[data-filter-panel]").await?)
    })
    .await?;
    eventually_focused(d, BUTTON, "Escape").await
}

e2e::scenario!(
    the_filter_piece_opens_the_panel_under_itself,
    "/table-toolbar",
    the_filter_piece_opens_the_panel
);
