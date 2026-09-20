//! `use_accessibility()` follows the browser's own answers, live, and a forced
//! reduced motion wins over them in libero's CSS (954).

use e2e::browser::{Scheme, block_on, emulate_media};
use e2e::{Fixture, Viewport, wait};

async fn wait_for(page: &chromiumoxide::Page, motion: &str, width: &str) {
    wait::for_js_true(
        page,
        &format!(
            "document.querySelector('#motion').textContent === '{motion}' && \
             getComputedStyle(document.querySelector('#motion-box')).width === '{width}'"
        ),
        &format!("the page to read {motion:?} at width {width}"),
    )
    .await
    .unwrap();
}

async fn click(page: &chromiumoxide::Page, id: &str) {
    page.evaluate(format!("document.querySelector('#{id}').click()"))
        .await
        .unwrap();
}

#[test]
fn reduced_motion_follows_the_browser_until_forced() {
    block_on(async {
        let fixture = Fixture::open("/use-accessibility", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        emulate_media(page, Scheme::Light, Some(false))
            .await
            .unwrap();
        wait_for(page, "no-preference", "10px").await;
        emulate_media(page, Scheme::Light, Some(true))
            .await
            .unwrap();
        wait_for(page, "reduce", "20px").await;

        click(page, "moving").await;
        wait_for(page, "no-preference", "10px").await;
        emulate_media(page, Scheme::Light, Some(false))
            .await
            .unwrap();
        click(page, "still").await;
        wait_for(page, "reduce", "20px").await;

        click(page, "system").await;
        wait_for(page, "no-preference", "10px").await;
        fixture.close().await.unwrap();
    });
}
