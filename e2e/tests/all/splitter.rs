//! `Splitter`: the divider keeps the keyboard after a mouse drag (todo 431).
//!
//! `use_drag` cancels the pointerdown, and with it the browser's own focus, so
//! the component has to focus the divider itself, as `Slider` does its thumb.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const DIVIDER: &str = "[role=separator]";
const VALUE_NOW: &str = "document.querySelector('[role=separator]').getAttribute('aria-valuenow')";

#[test]
fn it_meets_the_baseline() {
    Suite::new("splitter", "/splitter").focusable(DIVIDER).run();
}

/// Focus starts on a button outside, so a divider that never takes focus
/// fails the first assertion rather than passing on focus it already had.
#[test]
fn a_drag_leaves_the_divider_focused_for_the_arrow_keys() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/splitter", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, "#before", 5).await.unwrap();
            let before: String = page
                .evaluate(VALUE_NOW)
                .await
                .unwrap()
                .into_value()
                .unwrap();

            let from = pointer::centre_of(page, DIVIDER).await.unwrap();
            let to = pointer::Point {
                x: from.x + 40.0,
                y: from.y,
            };
            pointer::drag(page, from, to, 10).await.unwrap();
            wait::for_js_change(page, VALUE_NOW, &before, "the drag to move the divider")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));
            let dragged: String = page
                .evaluate(VALUE_NOW)
                .await
                .unwrap()
                .into_value()
                .unwrap();

            wait::for_js_true(
                page,
                &format!("document.activeElement === document.querySelector({DIVIDER:?})"),
                "the divider to hold focus after the drag",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
            wait::for_js_change(page, VALUE_NOW, &dragged, "ArrowRight to move the divider")
                .await
                .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("a splitter drag at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
