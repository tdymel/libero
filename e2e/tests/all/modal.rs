//! `Modal`: the overlay archetype.
//!
//! The focus-return half of this contract is the one that fails silently
//! everywhere else: nothing looks wrong on screen when focus falls back to
//! `<body>`, and the keyboard user is simply dumped at the top of the document.

use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

const TRIGGER: &str = "#open-modal";
const DIALOG: &str = "[role=dialog]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("modal", "/modal")
        .focusable(TRIGGER)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            DIALOG,
        )
        .run();
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
