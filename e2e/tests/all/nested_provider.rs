//! A nested `LiberoProvider` with German words and formats: its date field and
//! its portaled calendar speak German, the outer one stays English.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, wait};

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

/// Todo 2073: the root's `lang` is the outer provider's, and follows its switch.
#[test]
fn the_root_lang_follows_the_outer_provider_only() {
    block_on(async {
        let fixture = Fixture::open("/nested-provider", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, INNER).await.unwrap();
        wait::for_js_true(
            page,
            "document.documentElement.lang === 'en'",
            "the root lang to stay the outer English",
        )
        .await
        .unwrap();
        pointer::click(page, "#outer-german").await.unwrap();
        wait::for_js_true(
            page,
            "document.documentElement.lang === 'de'",
            "the root lang to follow the outer switch",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
