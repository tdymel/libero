//! `use_accessibility().get()` follows the browser's own answers, live (954).

use e2e::browser::{Scheme, block_on, emulate_media};
use e2e::{Fixture, Viewport, wait};

async fn wait_for_motion(page: &chromiumoxide::Page, motion: &str) {
    wait::for_js_true(
        page,
        &format!("document.querySelector('#motion').textContent === '{motion}'"),
        &format!("the page to read {motion:?}"),
    )
    .await
    .unwrap();
}

#[test]
fn get_follows_the_browsers_reduced_motion() {
    block_on(async {
        let fixture = Fixture::open("/use-accessibility", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        emulate_media(page, Scheme::Light, Some(false))
            .await
            .unwrap();
        wait_for_motion(page, "no-preference").await;
        emulate_media(page, Scheme::Light, Some(true))
            .await
            .unwrap();
        wait_for_motion(page, "reduce").await;
        emulate_media(page, Scheme::Light, Some(false))
            .await
            .unwrap();
        wait_for_motion(page, "no-preference").await;
        fixture.close().await.unwrap();
    });
}
