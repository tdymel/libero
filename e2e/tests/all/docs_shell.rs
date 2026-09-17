//! The docs shell's GitHub star pill and its focus move after a navigation,
//! driven through the docs' own files. `fetch` is mocked: no call leaves the page.

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

/// Replaces `fetch` with one answering `body` (`null`: a network error) and
/// recording each URL, then mounts the link and waits for it to have asked.
async fn mount_with_fetch(page: &Page, body: &str) {
    page.evaluate(format!(
        "(() => {{ sessionStorage.clear(); window.__urls = []; \
         window.fetch = (url) => {{ window.__urls.push(url); const body = {body}; \
         return body === null ? Promise.reject(new TypeError('blocked')) \
         : Promise.resolve(new Response(JSON.stringify(body))); }}; \
         document.querySelector('#mount').click(); }})()"
    ))
    .await
    .unwrap();
    wait::for_js_true(
        page,
        "window.__urls.length > 0",
        "the link to fetch the count",
    )
    .await
    .unwrap();
}

/// The link's name, text, width and height, once the count had time to land.
async fn link(page: &Page) -> (String, String, f64, f64) {
    // The answer is a resolved promise: a few frames settle it.
    page.evaluate("new Promise((r) => setTimeout(r, 300))")
        .await
        .unwrap();
    page.evaluate(
        "(() => { const a = document.querySelector('a[href=\"https://github.com/example/repo\"]'); \
         const r = a.getBoundingClientRect(); \
         return [a.getAttribute('aria-label'), a.textContent.trim(), r.width, r.height]; })()",
    )
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

async fn open_stars() -> Fixture {
    let fixture = Fixture::open("/docs-shell/stars", Viewport::Desktop)
        .await
        .unwrap();
    wait::for_visible(&fixture.page, "#mount").await.unwrap();
    fixture
}

#[test]
fn a_failed_fetch_leaves_the_plain_icon() {
    block_on(async {
        let fixture = open_stars().await;
        let page = &fixture.page;
        mount_with_fetch(page, "null").await;
        let (name, text, width, height) = link(page).await;
        assert_eq!(name, "GitHub (opens in a new tab)");
        assert_eq!(text, "G", "a count showed after an error");
        assert_eq!(width, height, "the icon box changed shape");
        let url: String = page
            .evaluate("window.__urls[0]")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(url, "https://api.github.com/repos/example/repo");
        let cached: bool = page
            .evaluate("sessionStorage.getItem('libero-docs-stars') !== null")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!cached, "an error was cached");
        fixture.close().await.unwrap();
    });
}

#[test]
fn zero_stars_leave_the_plain_icon() {
    block_on(async {
        let fixture = open_stars().await;
        let page = &fixture.page;
        mount_with_fetch(page, "{ stargazers_count: 0 }").await;
        let (name, text, width, height) = link(page).await;
        assert_eq!(name, "GitHub (opens in a new tab)");
        assert_eq!(text, "G");
        assert_eq!(width, height);
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_count_joins_the_icon_and_is_cached_for_the_session() {
    block_on(async {
        let fixture = open_stars().await;
        let page = &fixture.page;
        mount_with_fetch(page, "{ stargazers_count: 1234 }").await;
        let (name, text, width, height) = link(page).await;
        assert_eq!(name, "GitHub, 1.2k stars (opens in a new tab)");
        assert_eq!(text, "G1.2k");
        assert!(width > height, "no pill: {width}x{height}");
        // Unmount and mount again: the count comes from the session, not a second call.
        page.evaluate("document.querySelector('#mount').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('a[href=\"https://github.com/example/repo\"]')",
            "the link to unmount",
        )
        .await
        .unwrap();
        page.evaluate("document.querySelector('#mount').click()")
            .await
            .unwrap();
        let (_, text, _, _) = link(page).await;
        assert_eq!(text, "G1.2k");
        let calls: u32 = page
            .evaluate("window.__urls.length")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(calls, 1, "the count was fetched again");
        fixture.console.assert_clean("the stars fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_navigation_focuses_the_new_heading_but_a_load_does_not() {
    block_on(async {
        let fixture = Fixture::open("/docs-shell/heading", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#to-b").await.unwrap();
        let focused: String = page
            .evaluate("document.activeElement.tagName")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(focused, "BODY", "the first render moved focus");
        page.evaluate("document.querySelector('#to-b').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.tagName === 'H1' && document.activeElement.textContent === 'Page B'",
            "focus on the new heading",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the heading fixture").unwrap();
        fixture.close().await.unwrap();
    });
}
