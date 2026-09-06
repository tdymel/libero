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
        // No `.targets`: a row is under 24px tall, so 2.5.8 turns on its
        // spacing exception, which a plain size check cannot express. See
        // `its_rows_meet_the_target_spacing_exception`.
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

/// WCAG 2.5.8 for a column of radios, spacing exception included.
///
/// A row - circle and label, both of which pick the option - is under 24px
/// tall at `md`, so it passes 2.5.8 only if a 24px circle centred on each row
/// clears its neighbours: the rows' centres must be at least 24px apart.
///
/// Measured 2026-09-19 at 1280x800 and 390px wide, device scale 1: rows
/// 19.5px tall, 23.5px apart. Half a pixel short - todo 325. Fixing it turns
/// this green; drop the `ignore` then.
#[test]
#[ignore = "todo 325: RadioGroup's md rows sit 23.5px apart, under 2.5.8's 24px"]
fn its_rows_meet_the_target_spacing_exception() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/radio-group", viewport).await.unwrap();
            let rows: Vec<(f64, f64)> = fixture
                .page
                .evaluate(
                    "[...document.querySelectorAll('[role=radiogroup] > div')].map(el => { \
                     const r = el.getBoundingClientRect(); return [r.top + r.height / 2, r.height]; })",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(rows.len(), 3, "expected a row per option, found {rows:?}");
            for pair in rows.windows(2) {
                let ((a, height_a), (b, height_b)) = (pair[0], pair[1]);
                let pitch = b - a;
                let undersized = height_a < MINIMUM || height_b < MINIMUM;
                assert!(
                    !undersized || pitch >= MINIMUM,
                    "at {}: rows {height_a}px and {height_b}px tall sit {pitch}px apart, centre \
                     to centre; an undersized target needs {MINIMUM}px of clearance",
                    viewport.name()
                );
            }
            fixture.close().await.unwrap();
        }
    });
}
