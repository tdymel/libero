//! A label with no break opportunity wraps instead of widening the page, in
//! `Tree` rows, `NavLink` and `Menu` items (1.4.10, todo 518), and in
//! `Tabs`, `Timeline`, `Menubar` (todo 543).

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::{Fixture, Viewport, passes::pointer, wait};

/// Every overflowing part as `[what, scrollWidth, clientWidth]`, the page included.
const OVERFLOWS: &str = "(() => {
    const parts = [
        ['page', document.documentElement],
        ...[...document.querySelectorAll('[role=treeitem] > div')].map(e => ['tree row', e]),
        ...[...document.querySelectorAll('#nav a')].map(e => ['nav link', e]),
        ...[...document.querySelectorAll('[data-menu-index]')].map(e => ['menu item', e]),
        ...['#tabs', '#timeline', '#menubar', '#menubar-crowded'].flatMap(root =>
            [...document.querySelectorAll(`${root}, ${root} *`)].map(e => [root + ' ' + e.tagName, e])),
    ];
    const menu = document.querySelector('[role=menu]');
    // A crowded tab strip scrolls inside itself by design; each tab must still fit it.
    const out = parts.filter(([, e]) => e.scrollWidth > e.clientWidth && e.getAttribute('role') !== 'tablist')
        .map(([what, e]) => [what, e.scrollWidth, e.clientWidth]);
    const strip = document.querySelector('#tabs [role=tablist]');
    for (const tab of strip.querySelectorAll('[role=tab]'))
        if (tab.offsetWidth > strip.clientWidth) out.push(['tab', tab.offsetWidth, strip.clientWidth]);
    if (menu.getBoundingClientRect().right > innerWidth) out.push(['menu box', menu.getBoundingClientRect().right, innerWidth]);
    return out;
})()";

#[test]
fn a_long_label_wraps_instead_of_widening_the_page() {
    block_on(async {
        let fixture = Fixture::open("/long-labels", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        for width in [390, 320] {
            page.execute(SetDeviceMetricsOverrideParams::new(width, 800, 1.0, true))
                .await
                .unwrap();
            pointer::click(page, "[aria-haspopup=menu]").await.unwrap();
            wait::for_visible(page, "[role=menu]").await.unwrap();
            let overflows: Vec<serde_json::Value> = page
                .evaluate(OVERFLOWS)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(overflows.is_empty(), "{width}px: {overflows:?}");
            pointer::click(page, "[aria-haspopup=menu]").await.unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('[role=menu]')",
                "the menu closed",
            )
            .await
            .unwrap();
        }
        fixture.console.assert_clean("long labels").unwrap();
        fixture.close().await.unwrap();
    });
}
