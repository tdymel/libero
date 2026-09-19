//! `Collapse`: the motion fixture.
//!
//! The one reduced-motion test lives here because `Collapse` animates both
//! ways and ties its unmount to the exit (`use_presence`). The test it
//! replaces ran on `Autocomplete`, where nothing animates, so it could not
//! fail for what it named (review 7, E9).

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard::{self, TAB};
use e2e::passes::{motion, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, ax, wait};

pub const TOGGLE: &str = "#toggle-details";
pub const ROOT: &str = "#details";
const CONTENT: &str = "#details-text";

#[test]
fn it_meets_the_baseline() {
    Suite::new("collapse", "/collapse")
        .focusable(TOGGLE)
        .targets(TOGGLE)
        .state("open", &[Step::Click(TOGGLE)], CONTENT)
        .run();
}

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

/// Open, it grows to its content; closed, `keep_mounted: false` unmounts the
/// content once the exit ends.
async fn opens_and_unmounts<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(!d.exists(CONTENT).await?, "closed, it rendered its content");
    d.click(TOGGLE).await?;
    eventually(
        d,
        "the open collapse to reach its content's height",
        async |d| {
            if !d.exists(CONTENT).await? {
                return Ok(false);
            }
            let content = d.rect(CONTENT).await?.height;
            Ok(content > 0.0 && (d.rect(ROOT).await?.height - content).abs() <= 1.0)
        },
    )
    .await?;
    d.click(TOGGLE).await?;
    eventually(d, "the closed collapse to unmount its content", async |d| {
        Ok(!d.exists(CONTENT).await?)
    })
    .await
}

e2e::scenario!(
    it_opens_to_its_content_and_unmounts_it_once_closed,
    "/collapse",
    opens_and_unmounts
);

/// The focused element's id.
async fn focused(page: &chromiumoxide::Page) -> String {
    page.evaluate("document.activeElement?.id ?? ''")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// A kept-mounted panel's content is out of the tab order and the tree while
/// closed, and back in both once open.
#[test]
fn a_closed_kept_panel_is_neither_focusable_nor_announced() {
    block_on(async {
        let fixture = Fixture::open("/collapse-kept", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#toggle-kept").await.unwrap();
        let focus_trigger = "document.querySelector('#toggle-kept').focus()";

        page.evaluate(focus_trigger).await.unwrap();
        keyboard::press(page, TAB).await.unwrap();
        assert_eq!(focused(page).await, "after", "Tab skips the closed panel");
        let closed = ax::snapshot(page, "#kept-frame").await.unwrap();
        assert!(
            !closed.contains("Inside"),
            "closed panel announced: {closed}"
        );

        pointer::click(page, "#toggle-kept").await.unwrap();
        wait::for_visible(page, "#inside").await.unwrap();
        page.evaluate(focus_trigger).await.unwrap();
        keyboard::press(page, TAB).await.unwrap();
        assert_eq!(focused(page).await, "inside", "Tab enters the open panel");

        fixture.console.assert_clean("the kept collapse").unwrap();
        fixture.close().await.unwrap();
    });
}
