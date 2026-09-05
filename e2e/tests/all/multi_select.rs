//! `MultiSelect`: the combobox archetype.
//!
//! **The baselines hold a known defect, todo 326.** The chip's remove button
//! sits inside the `role="combobox"` element, so the combobox's value reads
//! `Cherry Remove Cherry`. Fixing it changes every `multi_select_*.snap`, and
//! that change is the fix showing up, not a regression.

use e2e::archetypes::Combobox;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const TRIGGER: &str = "[role=combobox]";
const LISTBOX: &str = "[role=listbox]";
const OPTION_COUNT: usize = 5;

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
