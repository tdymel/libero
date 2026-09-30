//! `Repository`: the star count joins the icon once it lands, and a failed or
//! zero answer leaves the icon alone. `fetch` is stubbed: no call leaves the page.

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{focus, keyboard};
use e2e::{Fixture, Suite, Viewport, wait};

const GITHUB: &str = "#github";
const GITLAB: &str = "#gitlab";

/// With a count seeded for the session, so the tree shows the pill.
#[test]
fn it_meets_the_baseline() {
    Suite::new("repository", "/repository/stubbed")
        .focusable(GITHUB)
        .targets(GITHUB)
        .run();
}

/// A filled or gradient count takes the fill's label colour, not the page's ink.
#[test]
fn a_count_on_a_fill_takes_the_label_colour() {
    block_on(async {
        let fixture = Fixture::open("/repository/stubbed", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#gradient [data-slot='count']")
            .await
            .unwrap();
        for id in ["#filled", "#gradient"] {
            let (label, count): (String, String) = page
                .evaluate(format!(
                    "(() => {{ const a = document.querySelector('{id}'); \
                     const c = a.querySelector('[data-slot=\"count\"]'); \
                     return [getComputedStyle(a).color, getComputedStyle(c).color]; }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(count, label, "{id}'s count left the label colour");
        }
        fixture.close().await.unwrap();
    });
}

/// Replaces `fetch` with one answering `body` (`null`: a network error) and
/// recording each URL, then mounts both buttons and waits for both to have asked.
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
        "window.__urls.length === 2",
        "both buttons to fetch their count",
    )
    .await
    .unwrap();
}

/// The link's name, text, width and height, once the count had time to land.
async fn link(page: &Page, selector: &str) -> (String, String, f64, f64) {
    // The answer is a resolved promise: a few frames settle it.
    page.evaluate("new Promise((r) => setTimeout(r, 300))")
        .await
        .unwrap();
    page.evaluate(format!(
        "(() => {{ const a = document.querySelector('{selector}'); \
         const r = a.getBoundingClientRect(); \
         return [a.getAttribute('aria-label'), a.textContent.trim(), r.width, r.height]; }})()"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

async fn open() -> Fixture {
    let fixture = Fixture::open("/repository", Viewport::Desktop)
        .await
        .unwrap();
    wait::for_visible(&fixture.page, "#mount").await.unwrap();
    fixture
}

#[test]
fn a_failed_fetch_leaves_the_plain_icon() {
    block_on(async {
        let fixture = open().await;
        let page = &fixture.page;
        mount_with_fetch(page, "null").await;
        let (name, text, width, height) = link(page, GITHUB).await;
        assert_eq!(name, "GitHub example/repo (opens in a new tab)");
        assert_eq!(text, "", "a count showed after an error");
        assert_eq!(width, height, "the icon box changed shape");
        let cached: bool = page
            .evaluate("sessionStorage.length > 0")
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
        let fixture = open().await;
        let page = &fixture.page;
        mount_with_fetch(page, "{ stargazers_count: 0, star_count: 0 }").await;
        let (name, text, width, height) = link(page, GITHUB).await;
        assert_eq!(name, "GitHub example/repo (opens in a new tab)");
        assert_eq!(text, "");
        assert_eq!(width, height);
        fixture.close().await.unwrap();
    });
}

/// Each host asks its own endpoint, reads its own field and leads to its own page.
#[test]
fn each_host_asks_its_endpoint_and_links_its_page() {
    block_on(async {
        let fixture = open().await;
        let page = &fixture.page;
        mount_with_fetch(page, "{ stargazers_count: 12, star_count: 3 }").await;
        let (github, _, _, _) = link(page, GITHUB).await;
        let (gitlab, _, _, _) = link(page, GITLAB).await;
        assert_eq!(github, "GitHub example/repo, 12 stars (opens in a new tab)");
        assert_eq!(
            gitlab,
            "GitLab group/sub/repo, 3 stars (opens in a new tab)"
        );
        let (mut urls, hrefs): (Vec<String>, Vec<String>) = page
            .evaluate(
                "[window.__urls, [...document.querySelectorAll('#github, #gitlab')].map(a => a.href + ' ' + a.target)]",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        urls.sort();
        assert_eq!(
            urls,
            [
                "https://api.github.com/repos/example/repo",
                "https://gitlab.com/api/v4/projects/group%2Fsub%2Frepo",
            ]
        );
        assert_eq!(
            hrefs,
            [
                "https://github.com/example/repo _blank",
                "https://gitlab.com/group/sub/repo _blank",
            ]
        );
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_count_joins_the_icon_and_is_cached_for_the_session() {
    block_on(async {
        let fixture = open().await;
        let page = &fixture.page;
        mount_with_fetch(page, "{ stargazers_count: 1234, star_count: 1234 }").await;
        let (name, text, width, height) = link(page, GITHUB).await;
        assert_eq!(name, "GitHub example/repo, 1.2k stars (opens in a new tab)");
        assert_eq!(text, "1.2k");
        assert!(width > height, "no pill: {width}x{height}");
        // Unmount and mount again: the count comes from the session, not a second call.
        page.evaluate("document.querySelector('#mount').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#github')",
            "the link to unmount",
        )
        .await
        .unwrap();
        page.evaluate("document.querySelector('#mount').click()")
            .await
            .unwrap();
        let (_, text, _, _) = link(page, GITHUB).await;
        assert_eq!(text, "1.2k");
        let calls: u32 = page
            .evaluate("window.__urls.length")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(calls, 2, "the count was fetched again");
        fixture.console.assert_clean("the repositories").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn aria_label_replaces_the_name_while_the_count_shows() {
    block_on(async {
        let fixture = Fixture::open("/repository/named", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#named").await.unwrap();
        let (name, text, _, _) = link(page, "#named").await;
        assert_eq!(name, "Example source (new tab)");
        assert_eq!(text, "1.2k", "the seeded count did not show");
        fixture.close().await.unwrap();
    });
}

/// It is a link: a tab stop, and Enter follows it. The click is caught before
/// it opens a tab.
#[test]
fn enter_follows_the_link() {
    block_on(async {
        let fixture = open().await;
        let page = &fixture.page;
        mount_with_fetch(page, "null").await;
        page.evaluate(
            "document.addEventListener('click', (e) => { const a = e.target.closest('a'); \
             if (a) { window.__followed = a.href; e.preventDefault(); } }, true)",
        )
        .await
        .unwrap();
        keyboard::tab_to(page, GITHUB, 5).await.unwrap();
        focus::assert_focused(page, GITHUB, "tabbing")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "window.__followed === 'https://github.com/example/repo'",
            "Enter to follow the link",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
