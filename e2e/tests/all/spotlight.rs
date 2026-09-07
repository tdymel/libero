//! `Spotlight`: the overlay archetype, as a modal command palette.
//!
//! Focus goes to the search box and stays there while the highlight moves, so
//! the palette is a trap with a single stop in it.
//!
//! `contrast_covers` on the dialog is not decoration. Until todo 327 was found
//! axe judged every row below the search box off-screen - `Modal`'s
//! `body { overflow: hidden }` scroll lock against a content-sized body box -
//! so `color-contrast` was inapplicable to the whole result list and a `#ddd`
//! description planted there stayed green. The lock is lifted for the axe run
//! now (`passes/contrast.rs`), and this line is what stops that regressing
//! into silence again.

use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const TRIGGER: &str = "#open-spotlight";
const DIALOG: &str = "[role=dialog]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("spotlight", "/spotlight")
        .focusable(TRIGGER)
        .targets("[role=dialog] [role=option]")
        .contrast_covers(DIALOG)
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
            let fixture = Fixture::open("/spotlight", viewport).await.unwrap();

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
                .assert_clean(&format!("the spotlight contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
