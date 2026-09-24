//! A nested `LiberoProvider` with German words and formats: its date field and
//! its portaled calendar speak German, the outer one stays English.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};

const OUTER: &str = "#outer input[data-controlled]";
const INNER: &str = "#inner input[data-controlled]";

/// The live value on the web; Blitz has none, but its `value` attribute is the DOM's.
async fn field_text<D: Driver>(d: &mut D, selector: &str) -> Result<String> {
    match d.value(selector).await {
        Ok(text) => Ok(text),
        Err(_) => Ok(d.attr(selector, "value").await?.unwrap_or_default()),
    }
}

async fn inner_speaks_german<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let outer = field_text(d, OUTER).await?;
    let inner = field_text(d, INNER).await?;
    ensure!(
        outer != inner,
        "the inner field writes the day its own way: {outer:?} vs {inner:?}"
    );
    ensure!(
        inner == "14. März 2026",
        "German words and formats: {inner:?}"
    );

    d.click(INNER).await?;
    eventually(d, "the inner calendar names March in German", async |d| {
        Ok(d.exists("[role=dialog]").await? && d.text("[role=dialog]").await?.contains("März"))
    })
    .await
}

e2e::scenario!(
    an_inner_provider_localizes_its_subtree,
    "/nested-provider",
    inner_speaks_german
);
