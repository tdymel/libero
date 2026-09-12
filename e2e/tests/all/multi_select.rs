//! `MultiSelect`: the combobox archetype.
//!
//! The chips sit beside the `role="combobox"` element, not in it, so its value
//! reads `Cherry` and not `Cherry Remove Cherry` (todo 70 (b), which rebaselined
//! every `multi_select_*.snap`).

use e2e::archetypes::Combobox;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const TRIGGER: &str = "[role=combobox]";
const LISTBOX: &str = "[role=listbox]";
const OPTION_COUNT: usize = 5;

/// Todo 483: the label focuses the trigger it names by id.
#[test]
fn a_click_on_the_label_focuses_the_trigger() {
    crate::select::label_click_focuses("/multi-select", TRIGGER);
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("multi_select", "/multi-select")
        .focusable(TRIGGER)
        .targets("[role=option]")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ARROW_DOWN)],
            LISTBOX,
        )
        .run();
}

#[test]
fn it_honours_the_combobox_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/multi-select", viewport).await.unwrap();

            Combobox {
                trigger: TRIGGER,
                option_count: OPTION_COUNT,
                tab_budget: 10,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the combobox contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
