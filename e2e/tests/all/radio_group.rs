//! `RadioGroup`: the `RadioSet` archetype.
//!
//! Listed under `RovingTabindex` until this unit was written, and checked
//! against it first: it shares the single tab stop and nothing else that
//! archetype asserts. APG's radio group has no Home or End, and its arrows
//! select as they move. See `archetypes/radio_set.rs`.

use e2e::archetypes::RadioSet;
use e2e::browser::block_on;
use e2e::passes::target_size::MINIMUM;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const RADIOS: &str = "[role=radiogroup] input[type=radio]";
/// The press target, and what WCAG 2.5.8 measures here: a click anywhere on
/// the row - circle or label - picks the option. Structural, because a row has
/// no role and no attribute of its own; if a measurement over it ever
/// disagrees with one over the inputs, suspect this first.
pub const ROWS: &str = "[role=radiogroup] > div";
/// The tab stop. Not `RADIOS`: `querySelector` would name the first radio,
/// which is not where Tab enters a group whose second option is checked.
const CHECKED: &str = "[role=radiogroup] input[type=radio]:checked";
/// What the "third" state waits on. Not `CHECKED`: the second radio is
/// checked at rest, so that wait returned before the arrow press landed.
const THIRD_CHECKED: &str = "[role=radiogroup] input[type=radio][data-radio-index=\"2\"]:checked";

#[test]
fn it_meets_the_baseline() {
    Suite::new("radio_group", "/radio-group")
        .focusable(CHECKED)
        // Not `.targets`: a row is under 24px tall, so 2.5.8 is met through
        // its spacing exception. `planted.rs`'s
        // `radio_group_rows_crammed_together_fail_the_spacing_exception` is
        // the proof this can refuse - before todo 377 the pass could not see
        // one row from another here, and this line would have been a check
        // that cannot fail.
        .targets_spaced(ROWS)
        // The third option checked, so a snapshot that still says the second
        // is checked after an arrow press is caught.
        .state(
            "third",
            &[Step::TabTo(CHECKED), Step::Press(keyboard::ARROW_DOWN)],
            THIRD_CHECKED,
        )
        .run();
}

#[test]
fn it_honours_the_radio_group_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/radio-group", viewport).await.unwrap();

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
                .assert_clean(&format!("the radio group contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// The rows really are undersized, so the spacing exception is load-bearing.
///
/// **Not a second implementation of WCAG 2.5.8.** Since todo 377
/// `it_meets_the_baseline` declares these rows with `targets_spaced` and the
/// pass computes the criterion over them, with
/// `planted::radio_group_rows_crammed_together_fail_the_spacing_exception` as
/// the proof it can refuse. Todo 376 converged the two, and the hand-written
/// arithmetic went with it.
///
/// What is left is the claim the pass cannot make. `targets_spaced` is the
/// weaker of 2.5.8's two ways of passing, and it is silently green on a target
/// that meets 24x24 outright - so if a theme change ever made these rows full
/// size, the weaker declaration would stay green and nothing would say the
/// unit had stopped needing it. This fails there, and the next reader gets
/// told to write `targets` instead.
///
/// Measured 2026-09-19 at 1280x800 and 390px wide, device scale 1: rows
/// 19.5px tall, 23.5px apart - half a pixel short, which was todo 302. Since
/// 302 each row is at least `24px - gap` tall, so the rows sit 24px apart.
#[test]
fn its_rows_are_undersized_so_the_spacing_exception_is_load_bearing() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/radio-group", viewport).await.unwrap();
            let heights: Vec<f64> = fixture
                .page
                .evaluate(format!(
                    "[...document.querySelectorAll({ROWS:?})]\
                     .map(el => el.getBoundingClientRect().height)"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(
                heights.len(),
                3,
                "expected a row per option, found {heights:?}"
            );
            for height in &heights {
                assert!(
                    *height < MINIMUM,
                    "at {}: a row is {height}px tall, which meets WCAG 2.5.8's {MINIMUM}px \
                     outright. The unit no longer needs the spacing exception, so \
                     `it_meets_the_baseline` should declare `targets(ROWS)` rather than \
                     `targets_spaced(ROWS)`",
                    viewport.name()
                );
            }
            fixture.close().await.unwrap();
        }
    });
}
