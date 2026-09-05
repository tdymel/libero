//! `Notifications`: the component the framework was not built for.
//!
//! Added deliberately to find the edge. Every other unit is a static page that
//! `Suite` opens, measures and snapshots. A notification exists only after an
//! action and removes itself on a timer, so its accessibility tree is a
//! function of *time* - which is the one axis none of the machinery has.
//!
//! ## What this is checked against
//!
//! - **WCAG 4.1.3 Status Messages** - a status is presented to assistive
//!   technology without taking focus. That second half is the assertion that
//!   matters and the one a sighted check cannot make.
//! - **WAI-ARIA `aria-live`** - politeness, and a region that is mounted before
//!   it has anything to say.
//!   <https://www.w3.org/TR/wai-aria-1.2/#aria-live>
//!
//! ## Where the framework did not reach, and what was done about it
//!
//! 1. **`Suite` cannot express "do this, then measure".** Its `state` steps run
//!    before a snapshot, so a notification *can* be reached that way - but the
//!    interesting assertions here are about what happens over time, not about a
//!    state. The unit is hand-written instead, and that is the right answer
//!    rather than a gap to close: bending `Suite` into a timeline runner would
//!    make it worse at the twenty static components it serves well.
//! 2. **Auto-close is pinned off in the fixture.** A fixture that removes
//!    itself while being measured is a race, not a test. Auto-close is worth
//!    testing and needs the clock driven rather than waited on - filed, not
//!    built.
//! 3. **No AX snapshot.** The tree differs by whether a notification is
//!    present, so a resting baseline would pin the empty case and prove very
//!    little. Snapshotting the *shown* state would work and is left for
//!    whoever grows this unit.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const TRIGGER: &str = "#notify";
const MESSAGE: &str = "Saved to your library";

/// The assertion that only a browser can make: the status is announced **and
/// focus does not move**.
///
/// WCAG 4.1.3 exists precisely because the tempting implementation - focus the
/// new thing so a screen reader reads it - is a worse experience than a live
/// region, and is indistinguishable from it in a screenshot.
#[test]
fn a_notification_is_announced_without_stealing_focus() {
    block_on(async {
        let fixture = Fixture::open("/notifications", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        // The notification arrives in a live region somewhere on the page.
        wait::for_js_true(
            page,
            &format!(
                "[...document.querySelectorAll('[aria-live], [role=status], [role=alert]')]\
                 .some(el => el.textContent.includes({}))",
                serde_json::to_string(MESSAGE).unwrap()
            ),
            "the notification to reach a live region",
        )
        .await
        .expect("a notification should be announced through a live region");

        // And focus is still on the button that asked for it.
        e2e::passes::focus::assert_focused(page, TRIGGER, "showing a notification")
            .await
            .expect("showing a notification must not move focus (WCAG 4.1.3)");

        fixture
            .console
            .assert_clean("showing a notification")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The live regions are mounted before they have anything to say.
///
/// A region that mounts together with its text is skipped by some screen
/// readers, which is why the library keeps them always-mounted and empty
/// (`codebase/components/notifications`). An assertion that a region exists
/// *while it is speaking* would hold just as well for the broken shape.
#[test]
fn the_live_regions_are_mounted_and_silent_before_anything_happens() {
    block_on(async {
        let fixture = Fixture::open("/notifications", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let regions: usize = page
            .evaluate("document.querySelectorAll('[aria-live], [role=status]').length")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            regions > 0,
            "no live region is mounted before the first notification; a region created \
             together with its text is skipped by some screen readers"
        );

        let spoken: bool = page
            .evaluate(
                "[...document.querySelectorAll('[aria-live], [role=status]')]\
                 .some(el => el.textContent.trim().length > 0)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            !spoken,
            "a live region is speaking before anything happened"
        );

        // Queried by what the regions *are* rather than by a role the host may
        // not use: the first version of this assumed `[role=status]`, found
        // nothing, and reported it as a harness error rather than as a wrong
        // assumption.
        let polite: usize = page
            .evaluate(
                "[...document.querySelectorAll('[aria-live], [role=status]')]\
                 .filter(el => (el.getAttribute('aria-live') || \
                   (el.getAttribute('role') === 'status' ? 'polite' : '')) === 'polite').length",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            polite > 0,
            "a notification host should mount at least one polite live region, found none \
             among {regions}"
        );

        fixture.close().await.unwrap();
    });
}
