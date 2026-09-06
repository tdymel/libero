//! `Collapse`: the motion fixture.
//!
//! The one reduced-motion test lives here because `Collapse` animates both
//! ways and ties its unmount to the exit (`use_presence`). The test it
//! replaces ran on `Autocomplete`, where nothing animates, so it could not
//! fail for what it named (review 7, E9).

use e2e::browser::block_on;
use e2e::passes::{motion, pointer};
use e2e::{Fixture, Viewport, wait};

pub const TOGGLE: &str = "#toggle-details";
pub const ROOT: &str = "#details";
const CONTENT: &str = "#details-text";

/// Under reduced motion nothing in the collapse transitions, open or closed,
/// and closing still unmounts the content.
///
/// The same measurement at `no-preference` must see motion first. That is
/// what makes a green reading here mean "the reduced arm switched it off"
/// rather than "the check measured an element that never animates".
#[test]
fn reduced_motion_switches_its_transitions_off() {
    block_on(async {
        let fixture = Fixture::open("/collapse", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::click(page, TOGGLE).await.unwrap();
        wait::for_visible(page, CONTENT).await.unwrap();
        motion::assert_still(page, ROOT)
            .await
            .expect_err("the collapse should animate without reduced motion");

        motion::set_reduced_motion(page, true).await.unwrap();
        motion::assert_reduced_motion_matches(page).await.unwrap();
        motion::assert_still(page, ROOT)
            .await
            .expect("open, under reduced motion");

        pointer::click(page, TOGGLE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!document.querySelector({CONTENT:?})"),
            "the closed collapse to unmount its content",
        )
        .await
        .unwrap();
        motion::assert_still(page, ROOT)
            .await
            .expect("closed, under reduced motion");

        fixture
            .console
            .assert_clean("the reduced-motion collapse")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
