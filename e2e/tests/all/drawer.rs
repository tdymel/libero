//! `Drawer`: the overlay archetype, as a modal docked to an edge.

use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const TRIGGER: &str = "#open-drawer";
const DIALOG: &str = "[role=dialog]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("drawer", "/drawer")
        .focusable(TRIGGER)
        .targets(TRIGGER)
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
            let fixture = Fixture::open("/drawer", viewport).await.unwrap();

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
                .assert_clean(&format!("the drawer contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
