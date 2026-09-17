//! `Badge`, `Indicator`, `Kbd` and `List`: contrast of small text in both
//! schemes, list semantics that survive `list-style: none`, and an `Indicator`
//! dot in forced colours.

use e2e::browser::block_on;
use e2e::passes::contrast;
use e2e::suite::Suite;
use e2e::{Fixture, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("badge", "/badge")
        .waive(contrast::TODO_297)
        .run();
}

/// Forced colours paint every fill `Canvas`: a bare `Indicator` dot, which is
/// nothing but its fill, must not vanish into the page.
#[test]
fn a_bare_indicator_dot_shows_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/badge", Viewport::Desktop).await.unwrap();
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
        let [dot, canvas]: [String; 2] = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 return [getComputedStyle(document.querySelector('#dot')).backgroundColor, canvas]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(dot, canvas, "the dot is filled with the page's own colour");
        fixture.close().await.unwrap();
    });
}
