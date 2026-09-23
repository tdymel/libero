//! `use_media_query` and `use_is_mobile`: the web follows the viewport live, native
//! Blitz keeps the documented `false` default.

use anyhow::Result;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually};
use e2e::{Fixture, Viewport, wait};

async fn answers<D: Driver>(d: &mut D) -> Result<(String, String)> {
    Ok((d.text("#wide").await?, d.text("#mobile").await?))
}

/// What the page reads for its own viewport: Blitz has no media queries.
async fn answers_for_the_viewport<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (width, _) = d.viewport().await?;
    let measures = d.platform() != Platform::Native;
    let yes = |on: bool| if on && measures { "yes" } else { "no" }.to_string();
    let expected = (yes(width >= 1024.0), yes(width < 768.0));
    eventually(d, &format!("answers to read {expected:?}"), async |d| {
        Ok(answers(d).await? == expected)
    })
    .await
}

e2e::scenario!(
    the_hooks_answer_for_the_viewport,
    "/use-media-query",
    answers_for_the_viewport
);

async fn reads(page: &chromiumoxide::Page, selector: &str, text: &str) -> Result<()> {
    let expression = format!("document.querySelector({selector:?})?.textContent === {text:?}");
    wait::for_js_true(page, &expression, &format!("{selector} to read {text}")).await
}

#[test]
fn the_web_follows_a_resize() {
    block_on(async {
        let fixture = Fixture::open("/use-media-query", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#wide").await.unwrap();
        reads(page, "#wide", "yes").await.unwrap();
        reads(page, "#mobile", "no").await.unwrap();

        for (width, wide, mobile) in [(390, "no", "yes"), (800, "no", "no"), (1280, "yes", "no")] {
            page.execute(SetDeviceMetricsOverrideParams::new(width, 800, 1.0, false))
                .await
                .unwrap();
            reads(page, "#wide", wide)
                .await
                .unwrap_or_else(|e| panic!("#wide at {width}px: {e}"));
            reads(page, "#mobile", mobile)
                .await
                .unwrap_or_else(|e| panic!("#mobile at {width}px: {e}"));
        }

        fixture
            .console
            .assert_clean("the media query hooks")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
