//! Responsive props of the data_display family (todo 5654): each `rsp-*` element draws like its
//! plain `wide-*` twin from `md` up and like `narrow-*` below.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// `selector-suffix property` per probe; the `{}` takes `rsp`, `narrow` or `wide`.
const PROBES: &str = "[['#{}-icon', 'width'], ['#{}-icon svg', 'width'], ['#{}-icon', 'background-color'], \
    ['#{}-indicator', 'height'], ['#{}-badge', 'border-top-left-radius'], \
    ['#{}-image', 'border-top-left-radius'], ['#{}-list', 'row-gap'], ['#{}-timeline', 'row-gap'], \
    ['#{}-timeline [data-slot=bullet]', 'width'], ['#{}-timeline [data-slot=bullet]', 'right'], \
    ['#{}-timeline > li', 'padding-left']]";

/// The probes of `who`, one `selector property=value` per line, the selector left unfilled.
fn look(who: &str) -> String {
    format!(
        "{PROBES}.map(([s, p]) => {{ const e = document.querySelector(s.replace('{{}}', '{who}')); \
         return s + ' ' + p + '=' + (e ? getComputedStyle(e)[p.replace(/-./g, m => m[1].toUpperCase())] : 'missing'); }}).join('\\n')"
    )
}

async fn values(page: &Page, who: &str) -> String {
    page.evaluate_bounded(look(who))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Waits until `rsp` draws like `twin`.
async fn follows(page: &Page, twin: &str) -> Result<(), String> {
    let same = format!("({}) === ({})", look("rsp"), look(twin));
    match wait::for_js_true(page, &same, twin).await {
        Ok(_) => Ok(()),
        Err(_) => Err(format!(
            "{}\n-- {twin} --\n{}",
            values(page, "rsp").await,
            values(page, twin).await
        )),
    }
}

#[test]
fn data_display_props_follow_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/responsive-data", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rsp-timeline").await.unwrap();
        let wide = values(page, "wide").await;
        assert!(!wide.contains("missing"), "{wide}");
        follows(page, "wide").await.unwrap();

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        follows(page, "narrow").await.unwrap();
        let narrow = values(page, "narrow").await;
        // Every probe moves, or a twin pair tests nothing.
        for (narrow, wide) in narrow.lines().zip(wide.lines()) {
            assert_ne!(narrow, wide, "the twins draw alike");
        }

        fixture
            .console
            .assert_clean("responsive data props")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
