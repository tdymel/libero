//! A barrier for checks that something did not happen: wait for what the page would
//! have done, not a fixed span.

use anyhow::Result;
use chromiumoxide::Page;

/// Resolves once what an event already sent queued has run: [`e2e::clock::settle`].
pub async fn painted(page: &Page) -> Result<()> {
    e2e::clock::settle(page).await
}
