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
