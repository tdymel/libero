//! Responsive layout and typography props (todo 5651): below `md` each element draws like its
//! plain `-narrow` twin, from `md` like its `-wide` twin; wrap, clamp and template switch too.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// `(selector with {} for the id, property)` per responsive element.
const LOOK: &str = r#"(mode) => {
    const pairs = [
        ['#{}', 'min-height', 'rl-header'],
        ['#{}', 'font-size', 'rl-text'],
        ['#{}', 'font-size', 'rl-title'],
        ['#{}', 'font-size', 'rl-kbd'],
        ['#{}', 'font-size', 'rl-anchor'],
        ['#{} > blockquote', 'padding-left', 'rl-quote'],
        ['#{}', 'border-bottom-width', 'rl-divider'],
        ['#{}', 'margin-top', 'rl-divider'],
        ['#{}', 'max-width', 'rl-container'],
        ['#{}', 'padding-left', 'rl-container'],
        ['#{}', 'width', 'rl-sidebar'],
        ['#{}', 'row-gap', 'rl-zone'],
    ];
    const value = (selector, property) =>
        getComputedStyle(document.querySelector(selector)).getPropertyValue(property);
    const off = pairs
        .filter(([selector, property, id]) =>
            value(selector.replace('{}', id), property) !==
            value(selector.replace('{}', `${id}-${mode}`), property))
        .map(([, property, id]) => `${id} ${property}`);
    const own = (id, property) => value(`#${id}`, property);
    return [
        off.join(',') || 'twins',
        own('rl-flex', 'flex-wrap'),
        own('rl-clamp', '-webkit-line-clamp'),
        own('rl-grid', 'grid-template-areas'),
    ].join('|');
}"#;

async fn look(page: &Page, mode: &str) -> String {
    page.evaluate_bounded(format!("({LOOK})('{mode}')"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Wide twins and a side-by-side grid from 62rem, narrow twins and a stacked one under it.
#[test]
fn layout_and_type_props_follow_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/responsive-layout", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rl-text").await.unwrap();
        assert_eq!(
            look(page, "wide").await,
            r#"twins|nowrap|2|"nav main main main""#
        );

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let narrow = r#"twins|wrap|none|"nav" "main""#;
        let reached = wait::for_js_true(
            page,
            &format!("({LOOK})('narrow') === {narrow:?}"),
            "the narrow look",
        )
        .await;
        assert!(reached.is_ok(), "{}", look(page, "narrow").await);

        fixture
            .console
            .assert_clean("responsive layout props")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
