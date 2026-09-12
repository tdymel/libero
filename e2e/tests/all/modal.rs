//! `Modal`: the overlay archetype.
//!
//! `contrast_covers` is load-bearing here. Todo 327: the scroll lock this
//! component injects, `body { overflow: hidden }`, made axe judge **every**
//! element in the dialog off-screen, so the open-state axe run checked nothing
//! at all. Fixed in `passes/contrast.rs`; this line keeps it fixed.
//!
//! The focus-return half of this contract is the one that fails silently
//! everywhere else: nothing looks wrong on screen when focus falls back to
//! `<body>`, and the keyboard user is simply dumped at the top of the document.

use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::wait;
use e2e::{
    Fixture, Suite, Viewport,
    passes::{focus, keyboard, pointer},
};

const TRIGGER: &str = "#open-modal";
const DIALOG: &str = "[role=dialog]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("modal", "/modal")
        .focusable(TRIGGER)
        .contrast_covers(DIALOG)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            DIALOG,
        )
        .run();
}

/// APG: a dialog with nothing focusable takes focus itself. Before, focus
/// stayed on the trigger behind the backdrop, so Tab walked the page and
/// Escape never reached the modal.
#[test]
fn a_dialog_with_nothing_focusable_takes_focus_itself() {
    block_on(async {
        let fixture = Fixture::open("/modal/static", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.matches('[role=dialog]') === true",
            "focus on the dialog",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::assert_focused(page, DIALOG, "Tab in a dialog with nothing to focus")
            .await
            .unwrap();

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]')",
            "Escape to close the dialog",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape")
            .await
            .unwrap();
        fixture.console.assert_clean("the static modal").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A click on the dialog's text used to drop focus to `<body>`, outside the
/// trap: Escape then did nothing and Tab reached the page behind.
#[test]
fn a_click_on_the_dialog_text_keeps_escape_and_the_trap() {
    block_on(async {
        let fixture = Fixture::open("/modal", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();

        pointer::click(page, "#modal-text").await.unwrap();
        for _ in 0..4 {
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            let inside: bool = page
                .evaluate(
                    "document.querySelector('[role=dialog]').contains(document.activeElement)",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                inside,
                "Shift+Tab after a click on the text left the dialog"
            );
        }

        pointer::click(page, "#modal-text").await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[role=dialog]')",
            "Escape after a click on the text to close the dialog",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, TRIGGER, "Escape")
            .await
            .unwrap();
        fixture.console.assert_clean("the clicked modal").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/modal", viewport).await.unwrap();

            Overlay {
                trigger: TRIGGER,
                panel: DIALOG,
                traps_focus: true,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the modal contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
