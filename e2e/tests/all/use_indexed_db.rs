//! `use_indexed_db`: writes, a second handle on the key and remove on every
//! backend; on the web a reload, another tab, a refused write and blocked
//! IndexedDB. Test pages share one profile, so each test has its own key and
//! removes it at the end.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually_text};
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, wait};

async fn writes_and_removes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#loaded", "true", "the load").await?;
    d.click("#remove").await?;
    for (selector, expected) in [("#count", "0"), ("#twin", "0"), ("#stored", "false")] {
        eventually_text(d, selector, expected, "removing the key").await?;
    }
    d.click("#add").await?;
    d.click("#add").await?;
    eventually_text(d, "#count", "2", "two adds").await?;
    eventually_text(d, "#twin", "2", "two adds").await?;
    eventually_text(d, "#stored", "true", "two adds").await?;
    eventually_text(d, "#error", "None", "two adds").await?;

    d.click("#remove").await?;
    eventually_text(d, "#count", "0", "remove").await?;
    eventually_text(d, "#twin", "0", "remove").await?;
    eventually_text(d, "#stored", "false", "remove").await?;
    eventually_text(d, "#error", "None", "remove").await
}

e2e::scenario!(
    writes_reach_every_handle_and_remove_falls_back,
    "/use-indexed-db",
    writes_and_removes
);

async fn reads(page: &Page, selector: &str, text: &str) -> Result<()> {
    let expression = format!("document.querySelector({selector:?})?.textContent === {text:?}");
    wait::for_js_true(page, &expression, &format!("{selector} to read {text}")).await
}

async fn click(page: &Page, selector: &str) {
    pointer::click(page, selector).await.unwrap();
}

/// The text under `key` in the origin's database, read behind the page's back.
async fn stored(page: &Page, key: &str) -> Option<String> {
    // In an array: chromiumoxide reads a bare `null` as no value.
    let [raw]: [Option<String>; 1] = (page
        .evaluate(format!(
            r#"new Promise((resolve) => {{
            const open = indexedDB.open('libero', 1);
            open.onerror = () => resolve([null]);
            open.onsuccess = () => {{
                const get = open.result.transaction('kv').objectStore('kv').get({key:?});
                get.onsuccess = () => {{ open.result.close(); resolve([get.result ?? null]); }};
                get.onerror = () => resolve([null]);
            }};
        }})"#
        ))
        .await)
        .unwrap()
        .into_value()
        .unwrap();
    raw
}

async fn stored_is(page: &Page, key: &str, expected: Option<&str>) {
    let what = format!("{key} to hold {expected:?} in IndexedDB");
    wait::until(&what, || async {
        Ok(stored(page, key).await.as_deref() == expected)
    })
    .await
    .unwrap();
}

/// A value survives a reload, stored as JSON, and `is_loaded` showed false before true.
#[test]
fn a_reload_keeps_the_value_and_the_load_flips() {
    block_on(async {
        let fixture = Fixture::open("/use-indexed-db/reload", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        reads(page, "#shown", "FT").await.unwrap();
        click(page, "#remove").await;
        reads(page, "#stored", "false").await.unwrap();
        click(page, "#add").await;
        click(page, "#add").await;
        reads(page, "#count", "2").await.unwrap();
        stored_is(page, "e2e-idb-reload-count", Some("2")).await;

        page.reload().await.unwrap();
        reads(page, "#loaded", "true").await.unwrap();
        reads(page, "#count", "2").await.unwrap();
        reads(page, "#twin", "2").await.unwrap();
        reads(page, "#stored", "true").await.unwrap();
        reads(page, "#shown", "FT").await.unwrap();

        click(page, "#remove").await;
        reads(page, "#stored", "false").await.unwrap();
        stored_is(page, "e2e-idb-reload-count", None).await;
        fixture.console.assert_clean("a reload").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A write or remove in one tab shows in another at once.
#[test]
fn another_tab_follows_a_write_and_a_remove() {
    block_on(async {
        let first = Fixture::open("/use-indexed-db/tabs", Viewport::Desktop)
            .await
            .unwrap();
        let second = Fixture::open("/use-indexed-db/tabs", Viewport::Desktop)
            .await
            .unwrap();
        let (a, b) = (&first.page, &second.page);
        reads(a, "#loaded", "true").await.unwrap();
        reads(b, "#loaded", "true").await.unwrap();
        click(a, "#remove").await;
        reads(b, "#count", "0").await.unwrap();

        click(a, "#add").await;
        reads(b, "#count", "1").await.unwrap();
        reads(b, "#twin", "1").await.unwrap();
        reads(b, "#stored", "true").await.unwrap();
        click(a, "#add").await;
        reads(b, "#count", "2").await.unwrap();

        click(a, "#remove").await;
        reads(b, "#count", "0").await.unwrap();
        reads(b, "#stored", "false").await.unwrap();

        first.console.assert_clean("the first tab").unwrap();
        second.console.assert_clean("the second tab").unwrap();
        first.close().await.unwrap();
        second.close().await.unwrap();
    });
}

/// Every `put` throws as a full store does, until `__unfill()`.
const FILL: &str = r#"(() => {
    const real = IDBObjectStore.prototype.put;
    IDBObjectStore.prototype.put = function () {
        throw new DOMException('The quota has been exceeded.', 'QuotaExceededError');
    };
    window.__unfill = () => { IDBObjectStore.prototype.put = real; };
})()"#;

/// A refused write keeps the value for the session and says why; the next good one clears it.
#[test]
fn a_full_store_reports_and_keeps_the_value() {
    block_on(async {
        let fixture = Fixture::open("/use-indexed-db/errors", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        reads(page, "#loaded", "true").await.unwrap();
        click(page, "#remove").await;
        reads(page, "#stored", "false").await.unwrap();

        page.evaluate(FILL).await.unwrap();
        click(page, "#add").await;
        reads(page, "#error", "Some(Full)").await.unwrap();
        reads(page, "#count", "1").await.unwrap();
        reads(page, "#twin", "1").await.unwrap();
        assert_eq!(stored(page, "e2e-idb-errors-count").await, None);

        page.evaluate("window.__unfill()").await.unwrap();
        click(page, "#add").await;
        reads(page, "#error", "None").await.unwrap();
        reads(page, "#count", "2").await.unwrap();
        stored_is(page, "e2e-idb-errors-count", Some("2")).await;

        click(page, "#remove").await;
        stored_is(page, "e2e-idb-errors-count", None).await;
        fixture.console.settle().await.unwrap();
        let warned = fixture.console.drain();
        assert_eq!(warned.len(), 1, "{warned:?}");
        assert!(warned[0].contains("is full"), "{warned:?}");
        fixture.close().await.unwrap();
    });
}

/// IndexedDB throws on access when site data is blocked.
const BLOCK: &str = r#"Object.defineProperty(window, 'indexedDB', {
    configurable: true,
    get() { throw new DOMException('Site data is blocked.', 'SecurityError'); },
})"#;

/// Blocked IndexedDB reports `Unavailable`; the value lasts the session.
#[test]
fn blocked_indexed_db_reports_unavailable_and_keeps_the_value() {
    block_on(async {
        let fixture = Fixture::open("/use-indexed-db/blocked", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_on_new_document(BLOCK).await.unwrap();
        page.reload().await.unwrap();
        reads(page, "#loaded", "true").await.unwrap();
        reads(page, "#error", "Some(Unavailable)").await.unwrap();
        reads(page, "#count", "0").await.unwrap();

        click(page, "#add").await;
        reads(page, "#count", "1").await.unwrap();
        reads(page, "#twin", "1").await.unwrap();
        reads(page, "#error", "Some(Unavailable)").await.unwrap();

        fixture.console.settle().await.unwrap();
        let warned = fixture.console.drain();
        assert_eq!(warned.len(), 1, "{warned:?}");
        assert!(
            warned[0].contains("nothing to keep values in"),
            "{warned:?}"
        );
        fixture.close().await.unwrap();
    });
}
