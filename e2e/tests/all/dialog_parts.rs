//! `DialogBody` and `DialogActions` (todo 5634).

use e2e::browser::block_on;
use e2e::wait;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, passes::keyboard};

const TRIGGER: &str = "#open-modal";

/// 5634: at a phone's width the body of a full-screen dialog scrolls while the actions stay
/// at the foot; under short content they still sit there.
#[test]
fn the_actions_of_a_full_screen_dialog_stay_pinned_at_a_phone() {
    const GEOMETRY: &str = "(() => { const body = document.querySelector('[data-slot=body]'); \
         const actions = document.querySelector('[data-slot=actions]').getBoundingClientRect(); \
         return [body.scrollTop, body.scrollHeight - body.clientHeight, actions.top, actions.bottom, innerHeight]; })()";
    block_on(async {
        for route in ["/modal/dialog-parts", "/modal/dialog-parts-short"] {
            let fixture = Fixture::open(route, Viewport::Mobile).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_visible(page, "[data-slot=actions]")
                .await
                .unwrap();
            let read = async || -> Vec<f64> {
                page.evaluate_bounded(GEOMETRY)
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap()
            };
            let [_, overflow, top, bottom, height] = read().await[..] else {
                panic!("{route}: geometry");
            };
            assert!(
                bottom <= height + 1.0 && bottom >= height - 40.0,
                "{route}: actions at {top}..{bottom} of {height}"
            );
            if route == "/modal/dialog-parts" {
                assert!(overflow > 0.0, "{route}: the body fits, nothing to scroll");
                page.evaluate_bounded("document.querySelector('[data-slot=body]').scrollTop = 300")
                    .await
                    .unwrap();
                wait::for_js_true(
                    page,
                    "document.querySelector('[data-slot=body]').scrollTop > 0",
                    "the body to scroll",
                )
                .await
                .unwrap();
                let [scrolled, _, scrolled_top, scrolled_bottom, _] = read().await[..] else {
                    panic!("{route}: geometry after scrolling");
                };
                assert!(scrolled > 0.0, "{route}: the body did not scroll");
                assert!(
                    (scrolled_top - top).abs() <= 1.0 && (scrolled_bottom - bottom).abs() <= 1.0,
                    "{route}: the actions moved from {top}..{bottom} to {scrolled_top}..{scrolled_bottom}"
                );
            }
            fixture.close().await.unwrap();
        }
    });
}
