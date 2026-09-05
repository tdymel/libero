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

/// Open a broken fixture, run `check`, and require it to report a failure.
///
/// Takes the pass as a closure rather than duplicating the fixture plumbing in
/// each test, and prints the error it got, so a reviewer can confirm the pass
/// failed **for the right reason** and not incidentally.
async fn must_fail<F, Fut>(route: &str, what: &str, check: F)
where
    F: FnOnce(Fixture) -> Fut,
    Fut: std::future::Future<Output = (Fixture, anyhow::Result<()>)>,
{
    let fixture = Fixture::open(route, Viewport::Desktop)
        .await
        .unwrap_or_else(|e| panic!("opening {route}: {e}"));

    let (fixture, outcome) = check(fixture).await;

    match outcome {
        Ok(()) => panic!(
            "{what} passed against {route}, which is deliberately broken. \
             The pass is not detecting what it claims to detect, and every \
             component relying on it is unguarded."
        ),
        Err(error) => println!("{what} correctly failed on {route}: {error}"),
    }

    let _ = fixture.close().await;
}

/// WCAG 2.4.7 Focus Visible.
#[test]
fn the_focus_ring_pass_catches_a_missing_ring() {
    block_on(async {
        must_fail(
            "/broken/focus-ring",
            "assert_focus_ring",
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

/// WCAG 2.5.8 Target Size (Minimum).
#[test]
fn the_target_size_pass_catches_a_small_target() {
    block_on(async {
        must_fail(
            "/broken/target-size",
            "assert_minimum",
            |fixture| async move {
                let result = target_size::assert_minimum(&fixture.page, "#tiny").await;
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

/// APG Tabs / roving tabindex: a composite widget is one tab stop.
#[test]
fn the_roving_pass_catches_many_tab_stops() {
    block_on(async {
        must_fail("/broken/roving", "RovingTabindex", |fixture| async move {
            let result = RovingTabindex {
                items: "[role=tab]",
                orientation: Orientation::Horizontal,
                wraps: true,
            }
            .assert_contract(&fixture.page)
            .await;
            (fixture, result)
        })
        .await;
    });
}

/// APG Dialog: focus returns to the element that opened it.
#[test]
fn the_overlay_pass_catches_focus_not_returning() {
    block_on(async {
        must_fail("/broken/focus-return", "Overlay", |fixture| async move {
            let result = Overlay {
                trigger: "#open-broken",
                panel: "#broken-dialog",
                traps_focus: false,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await;
            (fixture, result)
        })
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
