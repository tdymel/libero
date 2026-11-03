//! Responsive form props (todo 5652): the field's frame and the card look follow the viewport,
//! from the server's copies and from copies inserted after mount.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Frame heights of the small, large, responsive and late fields, then the card's border width.
const LOOK: &str = "(() => { const h = id => { const f = document.querySelector(`#${id} [data-slot='frame']`); \
    return f ? getComputedStyle(f).height : '-'; }; \
    const card = getComputedStyle(document.querySelector('#rspf-card > *')).borderTopWidth; \
    return [h('rspf-sm'), h('rspf-lg'), h('rspf-text'), h('rspf-late'), card].join('|'); })()";

async fn look(page: &Page) -> Vec<String> {
    let text: String = page
        .evaluate_bounded(LOOK)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    text.split('|').map(str::to_string).collect()
}

#[test]
fn form_props_follow_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/responsive-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rspf-text").await.unwrap();
        pointer::click(page, "#rspf-add").await.unwrap();
        wait::for_visible(page, "#rspf-late").await.unwrap();
        let wide = look(page).await;
        assert_ne!(wide[0], wide[1], "{wide:?}");
        assert_eq!((&wide[2], &wide[3]), (&wide[1], &wide[1]), "{wide:?}");
        assert_eq!(wide[4], "1px", "{wide:?}");

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let narrow = format!("{0}|{1}|{0}|{0}|0px", wide[0], wide[1]);
        let reached =
            wait::for_js_true(page, &format!("{LOOK} === '{narrow}'"), "the narrow look").await;
        assert!(reached.is_ok(), "{:?}", look(page).await);

        fixture
            .console
            .assert_clean("responsive form props")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
