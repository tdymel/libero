//! `Pagination`: an arrow that disables under its own press hands focus to the
//! current page, whether the caller answers at once or late (todo 406).

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

/// Sync, then a caller that sets the page 150 ms after `onchange`.
const ROUTES: [&str; 2] = ["/pagination", "/pagination-async"];

const PREVIOUS: &str = "[aria-label=\"Go to previous page\"]";
const LAST: &str = "[aria-label=\"Go to last page\"]";
const CURRENT: &str = "[aria-current=page]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("pagination", "/pagination")
        .focusable(PREVIOUS)
        .focusable(CURRENT)
        .targets("nav button")
        .run();
}

/// Todo 631: the current page was a fill only; it carries the house ring now.
#[test]
fn the_current_page_shows_the_on_state_ring() {
    block_on(async {
        let fixture = Fixture::open("/pagination", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        crate::button::assert_on_marker(page, CURRENT, "[aria-label=\"Go to page 3\"]").await;
        fixture.close().await.unwrap();
    });
}

async fn wait_for_page(page: &Page, number: u32, route: &str) {
    wait::for_js_true(
        page,
        &format!(
            "document.querySelector('#page').dataset.page === '{number}' \
             && document.querySelector('{CURRENT}').textContent === '{number}'"
        ),
        &format!("page {number} on {route}"),
    )
    .await
    .unwrap();
}

async fn is_disabled(page: &Page, selector: &str) -> bool {
    page.evaluate(format!("document.querySelector({selector:?}).disabled"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

#[test]
fn a_disabling_arrow_hands_focus_to_the_current_page() {
    block_on(async {
        for route in ROUTES {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;

            // A page button stays the same node when it becomes current, so it
            // keeps focus without any repair.
            let three = "[aria-label=\"Go to page 3\"]";
            keyboard::tab_to(page, three, 8).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait_for_page(page, 3, route).await;
            focus::assert_focused(page, CURRENT, &format!("choosing page 3 on {route}"))
                .await
                .unwrap();

            // Previous twice: the second press lands on page 1 and disables it.
            keyboard::tab_to(page, PREVIOUS, 12).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait_for_page(page, 2, route).await;
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait_for_page(page, 1, route).await;
            assert!(
                is_disabled(page, PREVIOUS).await,
                "{route}: previous at page 1"
            );
            focus::wait_for_focus(page, CURRENT, &format!("previous to page 1 on {route}"))
                .await
                .unwrap();

            pointer::click(page, LAST).await.unwrap();
            wait_for_page(page, 10, route).await;
            assert!(is_disabled(page, LAST).await, "{route}: last at page 10");
            focus::wait_for_focus(page, CURRENT, &format!("clicking last on {route}"))
                .await
                .unwrap();

            fixture
                .console
                .assert_clean(&format!("paging on {route}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 526: the current page is no change, so its click emits nothing, like
/// Select's same-value pick. Page 3 after it proves the first click was seen.
#[test]
fn clicking_the_current_page_emits_no_change() {
    block_on(async {
        let fixture = Fixture::open("/pagination", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, CURRENT).await.unwrap();
        pointer::click(page, "[aria-label=\"Go to page 3\"]")
            .await
            .unwrap();
        wait_for_page(page, 3, "/pagination").await;
        let changes: String = page
            .evaluate("document.querySelector('#page').dataset.changes")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(changes, "1", "only page 3 is a change");
        fixture.close().await.unwrap();
    });
}

/// `[id, :disabled, opacity, cursor]` of every button in the nav `id`.
async fn looks(page: &Page, id: &str) -> Vec<String> {
    page.evaluate(format!(
        "[...document.querySelectorAll('#{id} button')].map((b) => {{ \
         const s = getComputedStyle(b); \
         return `${{b.textContent || b.getAttribute('aria-label')}} ${{b.matches(':disabled')}} \
         ${{s.opacity}} ${{s.cursor}}`; }})"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// The page buttons carry no `disabled` state, so `disabled: true` dimmed only
/// the arrows; a disabled `Fieldset` dimmed nothing but the arrows (todo 514).
#[test]
fn every_control_of_a_disabled_pagination_looks_disabled() {
    block_on(async {
        let fixture = Fixture::open("/pagination/states", Viewport::Desktop)
            .await
            .unwrap();
        for id in ["pg-disabled", "pg-fieldset"] {
            let looks = looks(&fixture.page, id).await;
            assert_eq!(looks.len(), 9, "{id}: {looks:?}");
            let enabled: Vec<_> = looks
                .iter()
                .filter(|look| !look.ends_with(" true 0.5 default"))
                .collect();
            assert!(enabled.is_empty(), "{id} looks enabled: {enabled:?}");
        }
        fixture.close().await.unwrap();
    });
}

/// Forced colours paint every background `Canvas`, so the current page's fill
/// vanished and nothing but `aria-current` told it apart.
#[test]
fn the_current_page_shows_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/pagination/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        // The page itself is `Canvas`; a fill the same colour is no fill.
        let [current, canvas]: [String; 2] = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 return [getComputedStyle(document.querySelector('#pg-xs [aria-current=page]')) \
                 .backgroundColor, canvas]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(current, canvas, "the current page's fill is the page's own");
        fixture.close().await.unwrap();
    });
}

/// Eleven `md` controls are 353px, so the row ran out of a phone's nav (1.4.10).
#[test]
fn a_long_row_wraps_inside_a_phone() {
    block_on(async {
        let fixture = Fixture::open("/pagination/states", Viewport::Mobile)
            .await
            .unwrap();
        let overflowing: Vec<String> = fixture
            .page
            .evaluate(
                "[...document.querySelectorAll('nav')].filter((nav) => \
                 [...nav.querySelectorAll('button')].some((b) => \
                 b.getBoundingClientRect().right > nav.getBoundingClientRect().right + 0.5)) \
                 .map((nav) => nav.id)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(overflowing.is_empty(), "overflowing: {overflowing:?}");
        fixture.close().await.unwrap();
    });
}
