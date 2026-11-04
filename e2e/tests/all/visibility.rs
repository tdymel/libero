//! `hidden_from` / `visible_from` on every family (todo 5647): hidden at one width, shown at
//! the other, over Flex's, GridZone's and Dialog's own `display`.

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// The carriers of each word: how many there are, and how many draw (`display` not `none`).
const LOOK: &str = "(() => { const drawn = word => { \
    const all = [...document.querySelectorAll(`[data-state~=\"${word}\"]`)]; \
    return all.length + '/' + all.filter(e => getComputedStyle(e).display !== 'none').length; }; \
    return drawn('hidden-from-md') + '|' + drawn('visible-from-md'); })()";

/// 16 roots carry `hidden-from-md`, 3 `visible-from-md`: each family's flips at 62rem.
#[test]
fn hidden_from_and_visible_from_follow_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/visibility", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let wide = "16/0|3/3";
        let reached =
            wait::for_js_true(page, &format!("{LOOK} === '{wide}'"), "the wide look").await;
        assert!(
            reached.is_ok(),
            "{:?}",
            page.evaluate_bounded(LOOK)
                .await
                .unwrap()
                .into_value::<String>()
        );

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let narrow = "16/16|3/0";
        let reached =
            wait::for_js_true(page, &format!("{LOOK} === '{narrow}'"), "the narrow look").await;
        assert!(
            reached.is_ok(),
            "{:?}",
            page.evaluate_bounded(LOOK)
                .await
                .unwrap()
                .into_value::<String>()
        );

        fixture.console.assert_clean("visibility").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 5703: an open `CascaderDropdown` box carries the caller's `states` and hides under
/// `visible_from: "md"` once the window narrows.
#[test]
fn a_cascader_dropdown_box_takes_states_and_visible_from() {
    const DRAWN: &str = "(() => { const b = document.querySelector('[data-state~=\"ore-box\"][data-state~=\"visible-from-md\"]'); \
        return b ? getComputedStyle(b).display : 'gone'; })()";
    block_on(async {
        let fixture = Fixture::open("/visibility/cascader", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "[aria-haspopup]").await.unwrap();
        e2e::passes::pointer::click(page, "[aria-haspopup]")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("!['none', 'gone'].includes({DRAWN})"),
            "the wide box",
        )
        .await
        .unwrap();

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let hidden =
            wait::for_js_true(page, &format!("{DRAWN} === 'none'"), "the narrow box").await;
        assert!(
            hidden.is_ok(),
            "{:?}",
            page.evaluate_bounded(DRAWN)
                .await
                .unwrap()
                .into_value::<String>()
        );

        fixture.console.assert_clean("cascader visibility").unwrap();
        fixture.close().await.unwrap();
    });
}
