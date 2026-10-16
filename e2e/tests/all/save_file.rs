//! `save_file` (2016): Android hands the bytes to the share sheet. The web
//! download is checked in `table_toolbar`, through the export it saves.

use anyhow::Result;
use e2e::driver::{Driver, eventually, eventually_text};

async fn the_share_sheet_opens<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    shares(d, "#save").await
}

/// Todo 2078: bytes that are no text go to Downloads, then to the sheet as a file.
async fn the_share_sheet_opens_on_a_file<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let shared = shares(d, "#save-binary").await;
    // The app wrote `bytes.bin` to Downloads; the phone keeps it otherwise (todo 3348).
    #[cfg(feature = "android")]
    e2e::android::remove_download("bytes.bin").await?;
    shared
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

/// Todo 3548: Back as soon as frames stall, before the covered mark settles.
async fn back_closes_the_sheet_at_once<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#save-binary").await?;
    eventually(d, "the sheet's first stalled frames", async |d| {
        d.exists("html[data-stalled]").await
    })
    .await?;
    d.press_back().await?;
    let shared = eventually_text(d, "#outcome", "Shared", "the share outcome").await;
    #[cfg(feature = "android")]
    e2e::android::remove_download("bytes.bin").await?;
    shared
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
e2e::scenario!(
    android_reports_a_file_shared_when_back_closes_the_sheet_at_once,
    "/save-file",
    back_closes_the_sheet_at_once,
    android_only("3548: the share sheet is Android's")
);
