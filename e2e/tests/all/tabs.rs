//! `Tabs`: the roving-tabindex archetype.
//!
//! Added as the second archetype's first consumer. The point of this unit is
//! less about `Tabs` than about whether the archetype holds up when it meets a
//! component it was not written against - `RovingTabindex` was written from
//! APG, not from this component.

use e2e::archetypes::{Orientation, RovingTabindex};
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const TAB: &str = "[role=tab]";
/// What the "second" state waits on. Not `TAB`: that is visible at rest, so
/// waiting on it returned at once and the snapshot raced the re-render.
const SECOND_SELECTED: &str = "[role=tablist] > [role=tab]:nth-child(2)[aria-selected=true]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("tabs", "/tabs")
        .focusable(TAB)
        .targets(TAB)
        // The second tab selected, because an "is the right one marked" bug is
        // invisible in the resting state where the first is selected anyway.
        .state(
            "second",
            &[Step::TabTo(TAB), Step::Press(keyboard::ARROW_RIGHT)],
            SECOND_SELECTED,
        )
        .run();
}

#[test]
fn it_honours_the_roving_tabindex_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/tabs", viewport).await.unwrap();

            RovingTabindex {
                items: TAB,
                orientation: Orientation::Horizontal,
                // `TabsCore` wraps: `if at == last { 0 }`.
                wraps: true,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the tabs contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
