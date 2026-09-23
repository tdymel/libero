//! `use_in_viewport` and `use_intersection` against a real `IntersectionObserver`:
//! scrolling a target into view and out again.

use std::time::Duration;

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

async fn js<T: serde::de::DeserializeOwned>(page: &chromiumoxide::Page, expression: &str) -> T {
    page.evaluate(expression)
        .await
        .unwrap_or_else(|e| panic!("evaluate {expression}: {e}"))
        .into_value()
        .unwrap_or_else(|e| panic!("read {expression}: {e}"))
}

async fn state_reads(page: &chromiumoxide::Page, expected: &str) {
    wait::for_js_true(
        page,
        &format!("document.getElementById('state').textContent === '{expected}'"),
        &format!("#state to read {expected}"),
    )
    .await
    .unwrap();
}

/// Scrolls `target` to the middle of the viewport (`true`) or the page to the top.
async fn scroll(page: &chromiumoxide::Page, to_target: bool) {
    let expression = match to_target {
        true => "(document.getElementById('target').scrollIntoView({block: 'center'}), true)",
        false => "(window.scrollTo(0, 0), true)",
    };
    let _: bool = js(page, expression).await;
}

#[test]
fn a_target_is_in_the_viewport_only_while_scrolled_to() {
    block_on(async {
        let fixture = Fixture::open("/use-intersection/viewport", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        state_reads(page, "out").await;
        scroll(page, true).await;
        state_reads(page, "in").await;
        scroll(page, false).await;
        state_reads(page, "out").await;

        fixture
            .console
            .assert_clean("scrolling a target in and out")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn once_keeps_the_first_sighting() {
    block_on(async {
        let fixture = Fixture::open("/use-intersection/once", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        state_reads(page, "unseen").await;
        scroll(page, true).await;
        state_reads(page, "seen").await;
        scroll(page, false).await;
        // The control is the viewport test: there the same scroll flips it back.
        tokio::time::sleep(Duration::from_millis(400)).await;
        state_reads(page, "seen").await;

        fixture.close().await.unwrap();
    });
}

#[test]
fn a_scroller_is_the_root_and_the_ratio_follows_the_threshold() {
    block_on(async {
        let fixture = Fixture::open("/use-intersection/root", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        state_reads(page, "0").await;
        // The target sits 300px down a 100px scroller: all of it shows at 280.
        let _: bool = js(
            page,
            "(document.getElementById('scroller').scrollTop = 280, true)",
        )
        .await;
        state_reads(page, "100").await;
        let _: bool = js(
            page,
            "(document.getElementById('scroller').scrollTop = 0, true)",
        )
        .await;
        state_reads(page, "0").await;

        fixture.close().await.unwrap();
    });
}
