//! `use_local_storage` and `use_session_storage`: writes, a second handle on the
//! key and remove on every backend; on the web a reload, another tab, a full
//! store and unparsable text. Test pages share one profile, so each test has
//! its own keys and removes them at the end.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually_text};
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, wait};

/// Both keys removed: a past run may have left values.
async fn cleared<D: Driver>(d: &mut D) -> Result<()> {
    d.click("#remove-local").await?;
    d.click("#remove-session").await?;
    for (selector, expected) in [
        ("#local", "0"),
        ("#twin", "0"),
        ("#stored", "false"),
        ("#session", "0"),
        ("#error", "None"),
    ] {
        eventually_text(d, selector, expected, "removing both keys").await?;
    }
    Ok(())
}

async fn writes_and_removes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    cleared(d).await?;
    d.click("#add-local").await?;
    d.click("#add-local").await?;
    eventually_text(d, "#local", "2", "two adds").await?;
    eventually_text(d, "#twin", "2", "two adds").await?;
    eventually_text(d, "#stored", "true", "two adds").await?;
    eventually_text(d, "#session", "0", "local adds").await?;

    d.click("#add-session").await?;
    eventually_text(d, "#session", "1", "a session add").await?;
    eventually_text(d, "#local", "2", "a session add").await?;

    d.click("#remove-local").await?;
    eventually_text(d, "#local", "0", "remove").await?;
    eventually_text(d, "#twin", "0", "remove").await?;
    eventually_text(d, "#stored", "false", "remove").await?;
    eventually_text(d, "#error", "None", "remove").await?;
    d.click("#remove-session").await?;
    eventually_text(d, "#session", "0", "a session remove").await
}

e2e::scenario!(
    writes_reach_every_handle_and_remove_falls_back,
    "/use-local-storage",
    writes_and_removes
);

async fn reads(page: &Page, selector: &str, text: &str) -> Result<()> {
    let expression = format!("document.querySelector({selector:?})?.textContent === {text:?}");
    wait::for_js_true(page, &expression, &format!("{selector} to read {text}")).await
}

async fn click(page: &Page, selector: &str) {
    pointer::click(page, selector).await.unwrap();
}

/// The raw text under `key` in `area` (`localStorage` or `sessionStorage`).
async fn raw(page: &Page, area: &str, key: &str) -> Option<String> {
    // In an array: chromiumoxide reads a bare `null` as no value.
    let [raw]: [Option<String>; 1] = (page.evaluate(format!("[{area}.getItem({key:?})]")).await)
        .unwrap()
        .into_value()
        .unwrap();
    raw
}

/// Both areas survive a reload in the tab, stored as JSON.
#[test]
fn a_reload_keeps_both_areas() {
    block_on(async {
        let fixture = Fixture::open("/use-local-storage/reload", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        click(page, "#remove-local").await;
        click(page, "#remove-session").await;
        reads(page, "#stored", "false").await.unwrap();
        click(page, "#add-local").await;
        click(page, "#add-session").await;
        reads(page, "#local", "1").await.unwrap();
        reads(page, "#session", "1").await.unwrap();
        let local = raw(page, "localStorage", "e2e-storage-reload-local").await;
        assert_eq!(local.as_deref(), Some("1"));

        page.reload().await.unwrap();
        reads(page, "#local", "1").await.unwrap();
        reads(page, "#twin", "1").await.unwrap();
        reads(page, "#session", "1").await.unwrap();
        reads(page, "#stored", "true").await.unwrap();

        click(page, "#remove-local").await;
        click(page, "#remove-session").await;
        reads(page, "#stored", "false").await.unwrap();
        let session = raw(page, "sessionStorage", "e2e-storage-reload-session").await;
        assert_eq!(session, None);
        fixture.console.assert_clean("a reload").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A local write in one tab shows in another at once; session storage stays per tab.
#[test]
fn another_tab_follows_local_but_not_session() {
    block_on(async {
        let first = Fixture::open("/use-local-storage/tabs", Viewport::Desktop)
            .await
            .unwrap();
        let second = Fixture::open("/use-local-storage/tabs", Viewport::Desktop)
            .await
            .unwrap();
        let (a, b) = (&first.page, &second.page);
        click(a, "#remove-local").await;
        click(a, "#remove-session").await;
        reads(b, "#local", "0").await.unwrap();

        click(a, "#add-local").await;
        reads(b, "#local", "1").await.unwrap();
        reads(b, "#twin", "1").await.unwrap();
        reads(b, "#stored", "true").await.unwrap();

        // The local add after it arrives later than any session event could.
        click(a, "#add-session").await;
        click(a, "#add-local").await;
        reads(a, "#session", "1").await.unwrap();
        reads(b, "#local", "2").await.unwrap();
        reads(b, "#session", "0").await.unwrap();

        click(a, "#remove-local").await;
        reads(b, "#local", "0").await.unwrap();
        reads(b, "#stored", "false").await.unwrap();
        click(a, "#remove-session").await;
        reads(a, "#session", "0").await.unwrap();

        first.console.assert_clean("the first tab").unwrap();
        second.console.assert_clean("the second tab").unwrap();
        first.close().await.unwrap();
        second.close().await.unwrap();
    });
}

const KEY: &str = "e2e-storage-errors-local";

/// Every `setItem` throws as a full store does, until `__unfill()`.
const FILL: &str = r#"(() => {
    const real = Storage.prototype.setItem;
    Storage.prototype.setItem = function () {
        throw new DOMException('The quota has been exceeded.', 'QuotaExceededError');
    };
    window.__unfill = () => { Storage.prototype.setItem = real; };
})()"#;

/// Unparsable text shows the default and stays; a full store keeps the value
/// for the session; the next good write clears the error.
#[test]
fn bad_text_and_a_full_store_report_and_keep_the_value() {
    block_on(async {
        let fixture = Fixture::open("/use-local-storage/errors", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(format!("localStorage.setItem({KEY:?}, '{{bad')"))
            .await
            .unwrap();
        page.reload().await.unwrap();
        reads(page, "#error", "Some(Invalid)").await.unwrap();
        reads(page, "#local", "0").await.unwrap();
        reads(page, "#stored", "true").await.unwrap();
        assert_eq!(
            raw(page, "localStorage", KEY).await.as_deref(),
            Some("{bad")
        );

        page.evaluate(FILL).await.unwrap();
        click(page, "#add-local").await;
        reads(page, "#error", "Some(Full)").await.unwrap();
        reads(page, "#local", "1").await.unwrap();
        reads(page, "#twin", "1").await.unwrap();
        assert_eq!(
            raw(page, "localStorage", KEY).await.as_deref(),
            Some("{bad"),
            "a refused write changed the store"
        );

        page.evaluate("window.__unfill()").await.unwrap();
        click(page, "#add-local").await;
        reads(page, "#error", "None").await.unwrap();
        reads(page, "#local", "2").await.unwrap();
        assert_eq!(raw(page, "localStorage", KEY).await.as_deref(), Some("2"));

        click(page, "#remove-local").await;
        reads(page, "#stored", "false").await.unwrap();
        fixture.console.settle().await.unwrap();
        let warned = fixture.console.drain();
        for kind in ["does not parse", "is full"] {
            let count = warned.iter().filter(|line| line.contains(kind)).count();
            assert_eq!(count, 1, "one warning that the store {kind}: {warned:?}");
        }
        assert_eq!(warned.len(), 2, "{warned:?}");
        fixture.close().await.unwrap();
    });
}
