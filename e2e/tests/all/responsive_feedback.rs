//! Responsive feedback, navigation and overlay props (todo 5655): below `md` each element draws
//! like its plain `-narrow` twin, from `md` like its `-wide` twin.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// The elements that draw unlike their `mode` twin, or `twins`.
const LOOK: &str = r#"(mode) => {
    const pairs = [
        ['#{}', 'width', 'rf-loader'],
        ['#{}', 'height', 'rf-progress'],
        ['#{}', 'border-top-left-radius', 'rf-progress'],
        ['#{}', 'width', 'rf-circular'],
        ['#{}', 'height', 'rf-skeleton'],
        ['#{}', 'width', 'rf-skeleton'],
        ['#{}', 'border-top-left-radius', 'rf-skeleton'],
        ['#{} [aria-current]', 'height', 'rf-pagination'],
        ['#{} [aria-current]', 'border-top-left-radius', 'rf-pagination'],
        ['#{} [data-slot="arrow"]', 'width', 'rf-pagination'],
        ['#{} [data-slot="arrow"]', 'border-top-left-radius', 'rf-pagination'],
        ['#{} [role="menuitem"]', 'font-size', 'rf-menubar'],
        ['#{} [role="menuitem"]', 'border-top-left-radius', 'rf-menubar'],
        ['#{}', 'width', 'rf-burger'],
        ['#{} [data-slot="glyph"]', 'width', 'rf-burger'],
        ['#{}', 'width', 'rf-dialog'],
        ['#{}', 'border-top-left-radius', 'rf-dialog'],
    ];
    const value = (selector, property) =>
        getComputedStyle(document.querySelector(selector)).getPropertyValue(property);
    const off = pairs
        .filter(([selector, property, id]) =>
            value(selector.replace('{}', id), property) !==
            value(selector.replace('{}', `${id}-${mode}`), property))
        .map(([selector, property, id]) => `${selector.replace('{}', id)} ${property}`);
    return off.join(',') || 'twins';
}"#;

async fn look(page: &Page, mode: &str) -> String {
    page.evaluate_bounded(format!("({LOOK})('{mode}')"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Wide twins from 62rem, narrow twins under it.
#[test]
fn feedback_navigation_and_overlay_props_follow_the_viewport_width() {
    block_on(async {
        let fixture = Fixture::open("/responsive-feedback", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rf-loader").await.unwrap();
        assert_eq!(look(page, "wide").await, "twins");

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(600, 800, 1.0, false))
            .await
            .unwrap();
        let reached = wait::for_js_true(
            page,
            &format!("({LOOK})('narrow') === 'twins'"),
            "the narrow look",
        )
        .await;
        assert!(reached.is_ok(), "{}", look(page, "narrow").await);

        fixture
            .console
            .assert_clean("responsive feedback props")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
