//! `save_file` (2016): Android hands the bytes to the share sheet. The web
//! download is checked in `table_toolbar`, through the export it saves.

use anyhow::Result;
use e2e::driver::{Driver, eventually, eventually_text};

async fn the_share_sheet_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    shares(d, "#save").await
}

/// Todo 2078: bytes that are no text go to Downloads, then to the sheet as a file.
async fn the_share_sheet_opens_on_a_file<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    shares(d, "#save-binary").await
}

async fn shares<D: Driver>(d: &mut D, button: &str) -> Result<()> {
    d.click(button).await?;
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
e2e::scenario!(
    android_shares_binary_bytes_as_a_file,
    "/save-file",
    the_share_sheet_opens_on_a_file,
    android_only("2078: the share sheet is Android's")
);
