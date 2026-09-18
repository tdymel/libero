//! The suite's self-test: proof that each pass can fail.
//!
//! Every other unit here asserts that a component is correct. This one asserts
//! that the **harness** is, by pointing each pass at a fixture broken in
//! exactly the way that pass claims to catch and requiring an error.
//!
//! Why it earns its place: a pass that has never been observed to fail is
//! indistinguishable from a pass that cannot fail, and the second kind reports
//! as coverage forever. Five of the harness bugs found while building this
//! suite were assertions that looked right and were measuring the wrong thing;
//! three of those would have gone unnoticed if they had happened to be measuring
//! something that was *always true* rather than always false.
//!
//! If a test in this file starts failing, the pass it names has stopped
//! working, and **every component relying on that pass is silently unguarded**.
//! Treat it as more urgent than a component failure, not less.

use e2e::archetypes::{Combobox, Orientation, Overlay, RovingTabindex};
use e2e::browser::block_on;
use e2e::passes::{contrast, dismissal, focus, keyboard, target_size};
use e2e::{Fixture, Viewport, wait};

/// Open a broken fixture, run `check`, and require it to fail with an error
/// containing `because`.
///
/// Takes the pass as a closure rather than duplicating the fixture plumbing in
/// each test. `because` is what makes this a test of the pass rather than of
/// the route: an error is easy to produce, and a check that gave up somewhere
/// earlier - a selector that matched nothing, a step that never landed - would
/// satisfy a bare `is_err()` while proving nothing about the thing the fixture
/// breaks. `planted.rs` has required the reason from the start; this file did
/// not, which is how the roving count could fail for its *own* reason
/// (three `tabindex="0"` attributes) rather than for the one it claims to
/// measure.
async fn must_fail<F, Fut>(route: &str, what: &str, because: &str, check: F)
where
    F: FnOnce(Fixture) -> Fut,
    Fut: std::future::Future<Output = (Fixture, anyhow::Result<()>)>,
{
    let fixture = Fixture::open(route, Viewport::Desktop)
        .await
        .unwrap_or_else(|e| panic!("opening {route}: {e}"));

    let (fixture, outcome) = check(fixture).await;

    let _ = fixture.close().await;

    match outcome {
        Ok(()) => panic!(
            "{what} passed against {route}, which is deliberately broken. \
             The pass is not detecting what it claims to detect, and every \
             component relying on it is unguarded."
        ),
        Err(error) => {
            let error = format!("{error:#}");
            assert!(
                error.contains(because),
                "{what} failed on {route}, but not for the reason the fixture \
                 breaks.\n  expected: {because:?}\n  got: {error}"
            );
            println!("{what} correctly failed on {route}: {error}");
        }
    }
}

/// WCAG 2.4.7 Focus Visible.
#[test]
fn the_focus_ring_pass_catches_a_missing_ring() {
    block_on(async {
        must_fail(
            "/broken/focus-ring",
            "assert_focus_ring",
            "produced no visible ring anywhere on it",
            |fixture| async move {
                let result = focus::assert_focus_ring(&fixture.page, "#no-ring", 5)
                    .await
                    .map(|_| ());
                (fixture, result)
            },
        )
        .await;
    });
}

/// WCAG 1.4.11 on a field: the ring is on the `[data-ring]` overlay, not on
/// the frame's border, and a faint one must fail even though the border
/// changes strongly.
#[test]
fn the_focus_ring_pass_catches_a_faint_field_ring() {
    block_on(async {
        must_fail(
            "/broken/faint-field-ring",
            "assert_focus_ring + assert_ring_contrast",
            "WCAG 1.4.11",
            |fixture| async move {
                let result = async {
                    let ring = focus::assert_focus_ring(&fixture.page, "#faint-input", 5).await?;
                    anyhow::ensure!(
                        ring.overlay,
                        "measured {} instead of the ring overlay",
                        ring.selector
                    );
                    focus::assert_ring_contrast(&ring)
                }
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// WCAG 2.5.8 Target Size (Minimum).
#[test]
fn the_target_size_pass_catches_a_small_target() {
    block_on(async {
        must_fail(
            "/broken/target-size",
            "assert_minimum",
            "target size below WCAG 2.5.8 minimum",
            |fixture| async move {
                let result = target_size::assert_minimum(&fixture.page, "#tiny").await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// WCAG 2.5.8's spacing exception, which is what lets an undersized target
/// conform.
///
/// The exception is the half of 2.5.8 that can quietly pass everything: a
/// check that granted it without computing it would turn every undersized
/// control green. Two 20x20 buttons edge to edge are the case it must refuse -
/// each is under 24px, and the 24px circles on their centres overlap.
#[test]
fn the_target_size_pass_catches_targets_too_close_together() {
    block_on(async {
        must_fail(
            "/broken/target-spacing",
            "assert_minimum_or_spacing",
            "WCAG 2.5.8's spacing exception fails",
            |fixture| async move {
                let result =
                    target_size::assert_minimum_or_spacing(&fixture.page, "#cramped-a").await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// WCAG 1.4.3 Contrast (Minimum).
#[test]
fn the_contrast_pass_catches_faint_text() {
    block_on(async {
        must_fail(
            "/broken/contrast",
            "axe color-contrast",
            "color-contrast",
            |fixture| async move {
                let result = contrast::assert_clean(&fixture.page, "[data-fixture-ready]").await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// The recorder must see an exception thrown during the first mount, which is
/// the window an after-navigation listener misses.
#[test]
fn the_console_recorder_catches_a_mount_time_error() {
    block_on(async {
        let fixture = Fixture::open("/broken/console", Viewport::Desktop)
            .await
            .unwrap();

        // The throw happens during mount; give the report a moment to arrive
        // rather than racing it.
        let _ = wait::until("a console error to arrive", || async {
            Ok(!fixture.console.peek().is_empty())
        })
        .await;

        let messages = fixture.console.drain();
        assert!(
            !messages.is_empty(),
            "the console recorder saw nothing on a fixture that throws while mounting. \
             Every clean-console assertion in the suite is meaningless if this is true."
        );
        println!("console recorder correctly caught: {messages:?}");

        fixture.close().await.unwrap();
    });
}

/// A `console.warn` must fail the console pass. Dioxus reports a misused
/// scope at warn level (todo 283), and the pass used to keep only errors.
#[test]
fn the_console_pass_catches_a_warning() {
    block_on(async {
        let fixture = Fixture::open("/broken/console-warning", Viewport::Desktop)
            .await
            .unwrap();
        let _ = wait::until("a console warning to arrive", || async {
            Ok(!fixture.console.peek().is_empty())
        })
        .await;

        let outcome = fixture.console.assert_clean("mounting a warning fixture");
        fixture.close().await.unwrap();
        match outcome {
            Ok(()) => panic!(
                "the console pass stayed clean on a fixture that warns while mounting. \
                 Every warning the library or dioxus logs is unguarded."
            ),
            Err(error) => {
                assert!(
                    format!("{error:#}").contains("deliberate mount-time warning"),
                    "{error:#}"
                );
                println!("console pass correctly caught: {error:#}");
            }
        }
    });
}

/// APG Tabs / roving tabindex: a composite widget is one tab stop.
#[test]
fn the_roving_pass_catches_many_tab_stops() {
    block_on(async {
        must_fail(
            "/broken/roving",
            "RovingTabindex",
            "has 3 tab stops across 3 items",
            |fixture| async move {
                let result = RovingTabindex {
                    items: "[role=tab]",
                    orientation: Orientation::Horizontal,
                    wraps: true,
                }
                .assert_contract(&fixture.page)
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// The roving fixture must break the count and **nothing else**.
///
/// Every fixture here violates exactly one thing, and until 2026-09-20 this
/// one violated two: its arrows did nothing either. The count check happened
/// to run first, so the test above passed while resting on the wrong defect -
/// move the count check and it would have failed for a reason it never
/// claimed. This walks the keys the archetype requires and leaves that
/// ordering unable to hide anything.
#[test]
fn the_broken_roving_fixture_breaks_only_its_count() {
    block_on(async {
        let fixture = Fixture::open("/broken/roving", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let focused =
            "[...document.querySelectorAll('[role=tab]')].indexOf(document.activeElement)";

        keyboard::tab_to(page, "[role=tab]", 5).await.unwrap();
        for (key, expected) in [
            (keyboard::ARROW_RIGHT, 1),
            (keyboard::ARROW_RIGHT, 2),
            // Past the last item, which is where wrapping is decided.
            (keyboard::ARROW_RIGHT, 0),
            (keyboard::ARROW_LEFT, 2),
            (keyboard::HOME, 0),
            (keyboard::END, 2),
        ] {
            keyboard::press(page, key).await.unwrap();
            wait::for_js_true(
                page,
                &format!("{focused} === {expected}"),
                &format!("{} to move focus to tab {expected}", key.key),
            )
            .await
            .unwrap_or_else(|e| panic!("the fixture's keyboard is broken too: {e:#}"));
        }

        fixture.close().await.unwrap();
    });
}

/// APG Dialog: focus returns to the element that opened it.
#[test]
fn the_overlay_pass_catches_focus_not_returning() {
    block_on(async {
        must_fail(
            "/broken/focus-return",
            "Overlay",
            "expected focus on #open-broken",
            |fixture| async move {
                let result = Overlay {
                    trigger: "#open-broken",
                    panel: "#broken-dialog",
                    traps_focus: false,
                    tab_budget: 5,
                }
                .assert_contract(&fixture.page)
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// A dismissed panel must leave the accessibility tree, not merely become
/// invisible. `opacity: 0` hides nothing from assistive technology.
#[test]
fn the_dismissal_pass_catches_a_phantom_panel() {
    block_on(async {
        must_fail(
            "/broken/dismissal",
            "assert_gone_from_at",
            "is still in the accessibility tree",
            |fixture| async move {
                keyboard::tab_to(&fixture.page, "#dismiss", 5)
                    .await
                    .unwrap();
                keyboard::press(&fixture.page, keyboard::ENTER)
                    .await
                    .unwrap();
                let result = dismissal::assert_gone_from_at(&fixture.page, "#phantom").await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// Todo 101's bug: `aria-activedescendant` naming an element not in the DOM.
/// The attribute is present and well-formed; only its referent is missing.
#[test]
fn the_combobox_pass_catches_a_dangling_activedescendant() {
    block_on(async {
        must_fail(
            "/broken/activedescendant",
            "Combobox",
            "but no such element is in the DOM",
            |fixture| async move {
                let result = Combobox {
                    trigger: "#dangling",
                    option_count: 1,
                    tab_budget: 5,
                }
                .assert_contract(&fixture.page)
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// Review 7's hole: a combobox that never sets `aria-activedescendant` used to
/// pass, because an absent attribute counted as valid.
#[test]
fn the_combobox_pass_catches_a_missing_activedescendant() {
    block_on(combobox_must_fail(
        "/broken/activedescendant-missing",
        "#missing",
        "aria-activedescendant is absent",
    ));
}

/// A highlight that names a real option and never moves. Every reference
/// resolves, so only the check that arrows move the highlight can catch it.
#[test]
fn the_combobox_pass_catches_a_static_highlight() {
    block_on(combobox_must_fail(
        "/broken/static-highlight",
        "#static",
        "stayed on \"static-option-0\"",
    ));
}

/// `must_fail` for the combobox contract, also requiring the failure to be the
/// one the fixture was built for - both fixtures here are sound up to their
/// highlight, so failing anywhere else would mean an earlier step broke.
async fn combobox_must_fail(route: &str, trigger: &'static str, expected: &'static str) {
    must_fail(route, "Combobox", expected, |fixture| async move {
        let result = Combobox {
            trigger,
            option_count: 3,
            tab_budget: 5,
        }
        .assert_contract(&fixture.page)
        .await;
        (fixture, result)
    })
    .await;
}

/// Review 7, E1: a root that does not contain the overlay a state opens.
/// Narrowed to the trigger here, which is the shape the hole had: every
/// "open" state axe-checked the trigger alone and reported clean.
#[test]
#[should_panic(expected = "lie outside the suite root")]
fn the_suite_rejects_a_state_settling_outside_its_root() {
    e2e::Suite::new("broken_root", "/modal")
        .root("#open-modal")
        .no_snapshot()
        .state(
            "open",
            &[
                e2e::suite::Step::TabTo("#open-modal"),
                e2e::suite::Step::Press(keyboard::ENTER),
            ],
            "[role=dialog]",
        )
        .run();
}

/// Review 7, E8: a state whose `settled` selector is visible before its steps
/// run. Tabs' "second" state waited on `[role=tab]` that way, so reaching it
/// returned at once and the snapshot raced the re-render.
#[test]
#[should_panic(expected = "already visible before its steps")]
fn the_suite_rejects_a_state_that_waits_on_nothing() {
    e2e::Suite::new("broken_settled", "/tabs")
        .no_snapshot()
        .state(
            "second",
            &[
                e2e::suite::Step::TabTo("[role=tab]"),
                e2e::suite::Step::Press(keyboard::ARROW_RIGHT),
            ],
            "[role=tab]",
        )
        .run();
}

/// Review 7, E7: an `aria-controls` naming an element that does not exist.
/// The snapshot used to drop it without a line, so a baseline taken while it
/// dangled recorded nothing to notice.
#[test]
fn the_snapshot_records_a_dangling_aria_controls() {
    block_on(async {
        let fixture = Fixture::open("/tabs", Viewport::Desktop).await.unwrap();
        fixture
            .page
            .evaluate(
                "document.querySelector('[role=tab]:nth-child(2)')\
                 .setAttribute('aria-controls', 'planted-nowhere')",
            )
            .await
            .unwrap();

        let tree = e2e::ax::snapshot(&fixture.page, "[data-fixture-ready]")
            .await
            .unwrap();
        let _ = fixture.close().await;

        assert!(
            tree.contains("--> aria-controls [id=\"planted-nowhere\"] (missing)"),
            "the snapshot does not name the dangling reference:\n{tree}"
        );
    });
}

/// Todo 760: each family of the docs sweep reports its broken fixture. The
/// sweep never fails on a hit, so a hit is turned into the error here.
async fn sweep_must_report(route: &str, family: &'static str, element: &'static str, value: &str) {
    must_fail(route, "the docs sweep", family, |fixture| async move {
        let checked = e2e::sweep::check(&fixture.page, Viewport::Desktop, route, "light desktop")
            .await
            .unwrap_or_else(|e| panic!("sweeping {route}: {e:#}"));
        let found = checked
            .hits
            .iter()
            .find(|hit| {
                hit.family == family && hit.element.contains(element) && hit.value.contains(value)
            })
            .map(|hit| anyhow::anyhow!("{}: {} ({})", hit.family, hit.element, hit.value));
        (fixture, found.map_or(Ok(()), Err))
    })
    .await;
}

#[test]
fn the_sweep_reports_each_family() {
    block_on(async {
        sweep_must_report("/broken/contrast", "text-contrast", "#faint", "").await;
        sweep_must_report(
            "/broken/layered-text",
            "text-contrast",
            "#layered",
            "measured by the sweep",
        )
        .await;
        sweep_must_report(
            "/broken/faint-boundary",
            "boundary-contrast",
            "#faint-boundary",
            "",
        )
        .await;
        sweep_must_report("/broken/focus-ring", "focus-ring-missing", "#no-ring", "").await;
        sweep_must_report(
            "/broken/faint-field-ring",
            "focus-ring-contrast",
            "#faint-input",
            "",
        )
        .await;
        sweep_must_report("/broken/target-spacing", "target-size", "#cramped-a", "").await;
    });
}
