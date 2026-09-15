//! `use_popover` with `PopoverOptions::dismiss` (todo 522): Escape anywhere
//! and a press outside close the box; Escape hands focus to the trigger.

use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

const TRIGGER: &str = "#trigger";
const BOX: &str = "#box";
const BOX_TEXT: &str = "#box-text";
const IN_BOX: &str = "#in-box";
const BLANK: &str = "#blank";
const AFTER: &str = "#after";

async fn open(page: &chromiumoxide::Page) {
    pointer::click(page, TRIGGER).await.unwrap();
    wait::for_visible(page, BOX).await.unwrap();
}

/// A press on nothing focusable, or on another control, closes the box; a
/// press on the box's own text does not.
#[test]
fn a_press_outside_closes_it() {
    block_on(async {
        let fixture = Fixture::open("/popover", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        open(page).await;
        pointer::click(page, BLANK).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();

        open(page).await;
        pointer::click(page, BOX_TEXT).await.unwrap();
        pointer::click(page, IN_BOX).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        assert!(
            wait::is_visible(page, BOX).await.unwrap(),
            "a press inside the box closed it"
        );
        pointer::click(page, AFTER).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();
        focus::assert_focused(page, AFTER, "a press on another control")
            .await
            .unwrap();

        // The trigger's own click still toggles it shut.
        open(page).await;
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();

        fixture.console.assert_clean("pressing outside").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Escape closes it with focus on the trigger or inside the box, and focus
/// ends on the trigger.
#[test]
fn escape_closes_it_and_returns_focus_to_the_trigger() {
    block_on(async {
        let fixture = Fixture::open("/popover", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        open(page).await;
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();
        focus::assert_focused(page, TRIGGER, "Escape on the trigger")
            .await
            .unwrap();

        open(page).await;
        pointer::click(page, IN_BOX).await.unwrap();
        focus::assert_focused(page, IN_BOX, "a click on the box's control")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, BOX).await.unwrap();
        let _ = wait::for_js_true(
            page,
            "document.activeElement === document.querySelector('#trigger')",
            "focus back on the trigger",
        )
        .await;
        focus::assert_focused(page, TRIGGER, "Escape inside the box")
            .await
            .unwrap();

        fixture.console.assert_clean("Escape").unwrap();
        fixture.close().await.unwrap();
    });
}
