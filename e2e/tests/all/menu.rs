//! `Menu`: the overlay archetype, as a non-modal popup.
//!
//! Not a focus trap: Tab closes every level and moves on from the trigger.

use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

/// `a11y_attributes()` gives the trigger a generated id, so it is found by
/// the attribute that makes it a menu button instead.
pub const TRIGGER: &str = "[aria-haspopup=menu]";
const MENU: &str = "[role=menu]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("menu", "/menu")
        // Todo 386: the fixture's disabled "Paste" item is `aria-disabled`, and
        // axe's `color-contrast` rule skips a disabled control because WCAG
        // 1.4.3 exempts inactive components. The coverage guard does not model
        // that skip yet, so it reports the one label as an uncovered hole. A
        // false red, not a menu defect; deleting this line verifies the fix.
        .no_contrast_coverage("todo 386 - axe skips the `aria-disabled` item's label, and the guard does not model that yet")
        .focusable(TRIGGER)
        .targets("[role=menu] [role=menuitem]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            MENU,
        )
        .run();
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/menu", viewport).await.unwrap();

            Overlay {
                trigger: TRIGGER,
                panel: MENU,
                traps_focus: false,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the menu contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
