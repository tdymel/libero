//! `SegmentedControl`: the `RadioSet` archetype.
//!
//! Each segment is a native radio beside its `<label>`, so it is a radio group
//! to assistive technology and to the keyboard - not a tab strip, whatever it
//! looks like.

use e2e::archetypes::RadioSet;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const RADIOS: &str = "[role=radiogroup] input[type=radio]";
const CHECKED: &str = "[role=radiogroup] input[type=radio]:checked";
/// What the "right" state waits on: the last segment's label, checked. Not
/// `CHECKED`, since the radio is visually hidden and never counts as visible,
/// and not any checked label, since the middle one is checked at rest.
const RIGHT_CHECKED: &str = "[role=radiogroup] label[for$=\"-segment-2\"][data-state~=checked]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("segmented_control", "/segmented-control")
        .focusable(CHECKED)
        .targets("[role=radiogroup] label")
        .state(
            "right",
            &[Step::TabTo(CHECKED), Step::Press(keyboard::ARROW_RIGHT)],
            RIGHT_CHECKED,
        )
        .run();
}

#[test]
fn it_honours_the_radio_group_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/segmented-control", viewport).await.unwrap();

            RadioSet {
                radios: RADIOS,
                checked: 1,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!(
                    "the segmented control contract at {}",
                    viewport.name()
                ))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
