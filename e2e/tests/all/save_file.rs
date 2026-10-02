//! `save_file` (2016): Android hands the bytes to the share sheet. The web
//! download is checked in `table_toolbar`, through the export it saves.

use anyhow::Result;
use e2e::driver::{Driver, eventually, eventually_text};

async fn the_share_sheet_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#save").await?;
    eventually(d, "the share sheet over the page", async |d| {
        d.exists("html[data-covered]").await
    })
    .await?;
    // The covered activity applies no edit until Back closes the share sheet.
    d.press_back().await?;
    eventually_text(d, "#outcome", "Shared", "the share outcome").await
}

e2e::scenario!(
    android_shares_text_through_the_share_sheet,
    "/save-file",
    the_share_sheet_opens,
    android_only("2016: the share sheet is Android's")
);
