//! The suite's self-test: each pass must fail on a fixture broken the way it claims to catch.
//! A failure here means every component relying on that pass is silently unguarded.

use e2e::archetypes::{Combobox, Orientation, Overlay, RovingTabindex};
use e2e::browser::block_on;
use e2e::passes::{contrast, dismissal, focus, keyboard, reflow, target_size};
use e2e::{Fixture, Viewport, selftest, wait};

/// `must_fail` itself: a green check and a failure for another reason are both refused.
#[test]
fn must_fail_refuses_a_green_check_and_a_wrong_reason() {
    block_on(async {
        let green = selftest::caught("/button", None, "planted", async |_| Ok(())).await;
        assert!(
            green.is_err_and(|e| e.to_string().contains("stayed green")),
            "a green check was accepted"
        );
        let wrong = selftest::caught("/button", None, "planted", async |_| {
            anyhow::bail!("another reason")
        })
        .await;
        assert!(
            wrong.is_err_and(|e| e.to_string().contains("not for the broken reason")),
            "a failure for another reason was accepted"
        );
        let right = selftest::caught(
            "/button",
            Some("window.planted = 1"),
            "planted",
            async |page| {
                let planted: bool = page.evaluate("window.planted === 1").await?.into_value()?;
                anyhow::ensure!(!planted, "planted");
                Ok(())
            },
        )
        .await;
        assert_eq!(right.unwrap(), "planted");
    });
}

/// WCAG 2.4.7 Focus Visible.
#[test]
fn the_focus_ring_pass_catches_a_missing_ring() {
    block_on(async {
        selftest::must_fail(
            "/broken/focus-ring",
            None,
            "produced no visible ring anywhere on it",
            async |page| {
                focus::assert_focus_ring(page, "#no-ring", 5)
                    .await
                    .map(|_| ())
            },
        )
        .await;
    });
}

/// Only a child marked `data-ring` carries the ring (Tree's row line): any child would
/// credit a nested control's own ring.
#[test]
fn the_focus_ring_pass_ignores_a_ring_on_an_unmarked_child() {
    block_on(async {
        selftest::must_fail(
            "/broken/focus-ring",
            None,
            "produced no visible ring anywhere on it",
            async |page| {
                focus::assert_focus_ring(page, "#unmarked-ring", 5)
                    .await
                    .map(|_| ())
            },
        )
        .await;
    });
}

/// `outline: solid 0px` changes the shorthand and draws nothing (todo 1797).
#[test]
fn the_focus_ring_pass_catches_a_zero_width_outline() {
    block_on(ring_must_fail(
        "/broken/focus-ring",
        "#zero-ring",
        "produced no visible ring anywhere on it",
    ));
}

/// A translucent ring is measured as it shows over the surface, not as solid (todo 1796).
#[test]
fn the_focus_ring_pass_catches_a_translucent_ring() {
    block_on(ring_must_fail(
        "/broken/translucent-ring",
        "#translucent-ring",
        "WCAG 1.4.11",
    ));
}

/// WCAG 2.4.7: a ring cut by an `overflow: hidden` parent (todo 1793).
#[test]
fn the_focus_ring_pass_catches_a_clipped_ring() {
    block_on(ring_must_fail(
        "/broken/clipped-ring",
        "#clipped-ring",
        "by a clipping ancestor",
    ));
}

/// WCAG 2.4.11: a sticky bar over the focused control (todo 1793).
#[test]
fn the_focus_ring_pass_catches_covered_focus() {
    block_on(ring_must_fail(
        "/broken/covered-focus",
        "#covered-focus",
        "is covered at its centre by div",
    ));
}

/// `must_fail` for the ring and its contrast together, as `Suite` runs them.
async fn ring_must_fail(route: &str, selector: &'static str, expected: &'static str) {
    selftest::must_fail(route, None, expected, async |page| {
        let ring = focus::assert_focus_ring(page, selector, 5).await?;
        focus::assert_ring_contrast(&ring)
    })
    .await;
}

/// WCAG 1.4.11 on a field: the ring is the `[data-ring]` overlay, so a faint one fails even
/// though the border changes strongly.
#[test]
fn the_focus_ring_pass_catches_a_faint_field_ring() {
    block_on(async {
        selftest::must_fail(
            "/broken/faint-field-ring",
            None,
            "WCAG 1.4.11",
            async |page| {
                let ring = focus::assert_focus_ring(page, "#faint-input", 5).await?;
                anyhow::ensure!(
                    ring.overlay,
                    "measured {} instead of the ring overlay",
                    ring.selector
                );
                focus::assert_ring_contrast(&ring)
            },
        )
        .await;
    });
}

/// WCAG 2.5.8 Target Size (Minimum).
#[test]
fn the_target_size_pass_catches_a_small_target() {
    block_on(async {
        selftest::must_fail(
            "/broken/target-size",
            None,
            "target size below WCAG 2.5.8 minimum",
            async |page| target_size::assert_minimum(page, "#tiny").await,
        )
        .await;
    });
}

/// WCAG 2.5.8's spacing exception must be computed, not granted: two 20x20 buttons edge to
/// edge have overlapping 24px circles and must fail.
#[test]
fn the_target_size_pass_catches_targets_too_close_together() {
    block_on(async {
        selftest::must_fail(
            "/broken/target-spacing",
            None,
            "WCAG 2.5.8's spacing exception fails",
            async |page| target_size::assert_minimum_or_spacing(page, "#cramped-a").await,
        )
        .await;
    });
}

/// An undersized neighbour is judged by its box too: a long thin bar's centre is far away
/// although the bar lies inside the small target's circle (todo 1792).
#[test]
fn the_target_size_pass_catches_a_small_target_beside_a_thin_bar() {
    block_on(async {
        selftest::must_fail(
            "/broken/target-spacing",
            None,
            "needs 12px of clearance",
            async |page| target_size::assert_minimum_or_spacing(page, "#beside-bar").await,
        )
        .await;
    });
}

/// WCAG 1.4.10 Reflow: a 360px box at 320px wide, named past the exempted 2D grid (todo 1794).
#[test]
fn the_reflow_pass_catches_sideways_scroll_at_320() {
    block_on(async {
        selftest::must_fail(
            "/broken/reflow",
            None,
            "past div#too-wide reaching",
            async |page| reflow::assert_reflows(page, &["#wide-grid"], "the fixture").await,
        )
        .await;
    });
}

/// WCAG 1.4.3 Contrast (Minimum).
#[test]
fn the_contrast_pass_catches_faint_text() {
    block_on(async {
        selftest::must_fail("/broken/contrast", None, "color-contrast", async |page| {
            contrast::assert_clean(page, "[data-fixture-ready]").await
        })
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

/// What the page logged before `Recorder::settle` is in the recorder once it returns,
/// with no wait: a step's `assert_clean` sees its own messages (todo 1720).
#[test]
fn a_settled_recorder_holds_what_the_page_just_logged() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        for n in 0..20 {
            fixture
                .page
                .evaluate(format!("console.warn('settled {n}')"))
                .await
                .unwrap();
            fixture.console.settle().await.unwrap();
            let messages = fixture.console.drain();
            assert!(
                messages
                    .iter()
                    .any(|message| message.ends_with(&format!("\"settled {n}\""))),
                "warning {n} was not recorded at settle: {messages:?}"
            );
        }
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
        selftest::must_fail(
            "/broken/roving",
            None,
            "has 3 tab stops across 3 items",
            async |page| {
                RovingTabindex {
                    items: "[role=tab]",
                    orientation: Orientation::Horizontal,
                    wraps: true,
                }
                .assert_contract(page)
                .await
            },
        )
        .await;
    });
}

/// The roving fixture must break the count and nothing else: until 2026-09-20 its arrows
/// were broken too, hidden by check order. This walks the required keys.
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
        selftest::must_fail(
            "/broken/focus-return",
            None,
            "expected focus on #open-broken",
            async |page| {
                Overlay {
                    trigger: "#open-broken",
                    panel: "#broken-dialog",
                    traps_focus: false,
                    tab_budget: 5,
                }
                .assert_contract(page)
                .await
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
        selftest::must_fail(
            "/broken/dismissal",
            None,
            "is still in the accessibility tree",
            async |page| {
                keyboard::tab_to(page, "#dismiss", 5).await.unwrap();
                keyboard::press(page, keyboard::ENTER).await.unwrap();
                dismissal::assert_gone_from_at(page, "#phantom").await
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
        selftest::must_fail(
            "/broken/activedescendant",
            None,
            "but no such element is in the DOM",
            async |page| {
                Combobox {
                    trigger: "#dangling",
                    option_count: 1,
                    tab_budget: 5,
                }
                .assert_contract(page)
                .await
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

/// `must_fail` for the combobox contract, requiring the fixture's own failure: both are
/// sound up to their highlight.
async fn combobox_must_fail(route: &str, trigger: &'static str, expected: &'static str) {
    selftest::must_fail(route, None, expected, async |page| {
        Combobox {
            trigger,
            option_count: 3,
            tab_budget: 5,
        }
        .assert_contract(page)
        .await
    })
    .await;
}

/// Review 7, E1: a root narrowed to the trigger misses the overlay a state opens, so every
/// "open" state axe-checked the trigger alone.
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

/// Review 7, E8: a state whose `settled` selector is visible before its steps run returns
/// at once, and the snapshot races the re-render.
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

/// Review 7, E7: an `aria-controls` naming a missing element, which the snapshot used to
/// drop silently.
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
    selftest::must_fail(route, None, family, async |page| {
        let checked = e2e::sweep::check(page, Viewport::Desktop, route, "light desktop")
            .await
            .unwrap_or_else(|e| panic!("sweeping {route}: {e:#}"));
        let found = checked
            .hits
            .iter()
            .find(|hit| {
                hit.family == family && hit.element.contains(element) && hit.value.contains(value)
            })
            .map(|hit| anyhow::anyhow!("{}: {} ({})", hit.family, hit.element, hit.value));
        found.map_or(Ok(()), Err)
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

/// `wait::until` gives up inside `expecting_failure`'s share of the budget, naming what it
/// waited for; an error from the check ends it at once (1717).
#[test]
fn a_wait_gives_up_at_its_share_and_stops_on_an_error() {
    block_on(async {
        let started = std::time::Instant::now();
        let error = wait::expecting_failure_in(wait::QUICK_FAILURE_SHARE, async {
            wait::until("a condition that never holds", || async { Ok(false) }).await
        })
        .await
        .expect_err("a false condition passed");
        let took = started.elapsed();
        assert!(
            format!("{error}").contains("waiting for a condition that never holds"),
            "{error}"
        );
        // The full budget is the defect the share exists against.
        assert!(took * 2 < wait_budget(), "the share took {took:?}");

        let started = std::time::Instant::now();
        let error = wait::until("a check that errs", || async {
            anyhow::bail!("the page went away")
        })
        .await
        .expect_err("an erring check passed");
        assert!(format!("{error}").contains("the page went away"), "{error}");
        assert!(
            started.elapsed() * 2 < wait_budget(),
            "an error waited out the budget"
        );
    });
}

/// `E2E_TIMEOUT_MS`, as `wait` reads it.
fn wait_budget() -> std::time::Duration {
    std::env::var("E2E_TIMEOUT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .map(std::time::Duration::from_millis)
        .unwrap_or(std::time::Duration::from_secs(15))
}

/// `for_js_change` refuses an expression that reads nothing before the action (todo 383),
/// and gives up on one that never changes (1717).
#[test]
fn for_js_change_refuses_a_null_before_and_waits_for_a_real_change() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let error = wait::for_js_change(page, "null", "a null to change", || async { Ok(()) })
            .await
            .expect_err("a null before-value passed");
        assert!(
            format!("{error}").contains("reads nothing before it"),
            "{error}"
        );

        let error = wait::expecting_failure(wait::for_js_change(
            page,
            "'still'",
            "a constant to change",
            || async { Ok(()) },
        ))
        .await
        .expect_err("an unchanged value passed");
        assert!(
            format!("{error}").contains("a constant to change"),
            "{error}"
        );

        wait::for_js_change(
            page,
            "String(window.__e2eStep ?? 0)",
            "a real change",
            || async {
                page.evaluate("window.__e2eStep = 1").await?;
                Ok(())
            },
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The held clock holds only the delays named, and `fire` refuses anything but exactly one
/// pending timer of the delay (1717).
#[test]
fn the_held_clock_holds_its_delays_and_fires_one_at_a_time() {
    use e2e::clock;
    const HELD: u32 = 4321;
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        clock::hold(page, &[HELD]).await.unwrap();
        let error = clock::fire(page, HELD)
            .await
            .expect_err("fired with none pending");
        assert!(
            format!("{error}").contains("0 held timers of 4321 ms"),
            "{error}"
        );

        let _: bool = e2e::js(
            page,
            format!(
                "(window.__held = 0, window.__real = 0, \
                 setTimeout(() => window.__held++, {HELD}), setTimeout(() => window.__held++, {HELD}), \
                 setTimeout(() => window.__real++, 0), true)"
            ),
        )
        .await;
        wait::for_js_true(page, "window.__real === 1", "the unheld delay to run")
            .await
            .unwrap();
        assert_eq!(clock::armed(page, HELD).await.unwrap(), 2);
        let error = clock::fire(page, HELD).await.expect_err("fired one of two");
        assert!(
            format!("{error}").contains("2 held timers of 4321 ms"),
            "{error}"
        );
        let held: u32 = e2e::js(page, "window.__held").await;
        assert_eq!(held, 0, "a held timer ran on its own");

        assert_eq!(clock::fire_all(page, HELD).await.unwrap(), 2);
        let held: u32 = e2e::js(page, "window.__held").await;
        assert_eq!(held, 2);
        fixture.close().await.unwrap();
    });
}

/// `live_region` tells a missing region from a wrong politeness, and reads `role=status` as
/// polite (1717).
#[test]
fn the_live_region_pass_catches_a_missing_or_assertive_region() {
    use e2e::passes::live_region;
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let _: bool = e2e::js(
            page,
            "(document.body.insertAdjacentHTML('beforeend', \
             '<div id=e2e-loud aria-live=assertive>Saved</div><div id=e2e-status role=status></div>'), true)",
        )
        .await;

        let error = live_region::assert_politeness(page, "#e2e-loud", "polite")
            .await
            .expect_err("an assertive region passed as polite");
        assert!(
            format!("{error}").contains("is \"assertive\", expected \"polite\""),
            "{error}"
        );
        live_region::assert_politeness(page, "#e2e-status", "polite")
            .await
            .unwrap();
        let error = live_region::assert_politeness(page, "#e2e-none", "polite")
            .await
            .expect_err("a missing region passed");
        assert!(
            format!("{error}").contains("no live region at #e2e-none"),
            "{error}"
        );

        assert_eq!(
            live_region::text_of(page, "#e2e-loud").await.unwrap(),
            "Saved"
        );
        assert_eq!(live_region::text_of(page, "#e2e-status").await.unwrap(), "");
        let error = live_region::text_of(page, "#e2e-none")
            .await
            .expect_err("a missing region read as text");
        assert!(
            format!("{error}").contains("no live region at #e2e-none"),
            "{error}"
        );
        fixture.close().await.unwrap();
    });
}

/// The pointer pass refuses to aim at nothing, rather than click the page's corner (1717).
#[test]
fn the_pointer_pass_refuses_a_missing_target() {
    use e2e::passes::pointer;
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let error = pointer::click(&fixture.page, "#e2e-nothing")
            .await
            .expect_err("a click on nothing passed");
        assert!(
            format!("{error}").contains("no element at #e2e-nothing to point at"),
            "{error}"
        );
        fixture.close().await.unwrap();
    });
}
