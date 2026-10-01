//! Android's Back key on an open layer (1275, 1289, 1300): it closes the layer, not the
//! app, and the trigger still opens it. Only Android has the key, so a scenario using
//! this skips every other arm (`android_only`).

use anyhow::Result;

use crate::driver::{Driver, eventually};

/// Opens through `trigger` until `open` holds, presses Back, waits for `open` to drop,
/// then opens it again: the app stayed.
pub async fn back_closes<D: Driver>(
    d: &mut D,
    trigger: &str,
    mut open: impl AsyncFnMut(&mut D) -> Result<bool>,
) -> Result<()> {
    d.click(trigger).await?;
    eventually(d, "the layer to open", async |d| open(d).await).await?;
    d.press_back().await?;
    eventually(d, "Back to close the layer", async |d| Ok(!open(d).await?)).await?;
    d.click(trigger).await?;
    eventually(d, "the app to stay and open it again", async |d| {
        open(d).await
    })
    .await
}
