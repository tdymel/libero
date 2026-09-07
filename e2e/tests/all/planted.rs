//! Planted defects: proof that a pass can fail **on a real component**.
//!
//! `negative.rs` proves each pass can fail on a hand-written broken page. That
//! says the pass works, not that it works on the component a unit points it
//! at: a selector that matches a different element, a ring drawn somewhere the
//! pass does not look, or a contract that stops early would each leave that
//! component's unit green whatever the component did.
//!
//! So each component added in todo 311 gets one defect planted into its own
//! fixture, and the pass that should catch it has to fail - **for the reason
//! named**, not for any reason. A plant that made the pass fail somewhere
//! earlier would prove nothing about the check it was aimed at; that is how the
//! first `DanglingActiveDescendant` fixture passed while testing nothing
//! (`fixtures/src/broken.rs`).
//!
//! Most plants are injected from here after the page mounts - a stylesheet or a
//! capturing listener - so the library is untouched and the fixture is the one
//! its own unit runs against. One needs a prop, and has a route of its own.

use e2e::archetypes::{Combobox, Orientation, Overlay, RadioSet, RovingTabindex, count_tab_stops};
use e2e::browser::block_on;
use e2e::passes::{contrast, focus, keyboard, motion, pointer, target_size};
use e2e::{Fixture, Viewport, wait};

/// Open `route`, plant `defect` (JavaScript, run once the app has mounted),
/// run `check`, and require it to fail with an error containing `because`.
async fn must_fail<F, Fut>(route: &str, defect: Option<&str>, because: &str, check: F)
where
    F: FnOnce(Fixture) -> Fut,
    Fut: std::future::Future<Output = (Fixture, anyhow::Result<()>)>,
{
    let fixture = Fixture::open(route, Viewport::Desktop)
        .await
        .unwrap_or_else(|e| panic!("opening {route}: {e}"));
    if let Some(defect) = defect {
        fixture
            .page
            .evaluate(defect)
            .await
            .unwrap_or_else(|e| panic!("planting the defect on {route}: {e}"));
    }

    let (fixture, outcome) = check(fixture).await;
    let _ = fixture.close().await;

    match outcome {
        Ok(()) => panic!(
            "the pass stayed green on {route} with a defect planted. It cannot see the \
             defect it exists for on this component."
        ),
        Err(error) => {
            let error = format!("{error:#}");
            assert!(
                error.contains(because),
                "the pass failed on {route}, but not for the planted reason.\n  expected: \
                 {because:?}\n  got: {error}"
            );
            println!("planted defect on {route} correctly caught: {error}");
        }
    }
}

/// A stylesheet appended to the document, as a plant.
fn stylesheet(css: &str) -> String {
    format!(
        "(() => {{ const s = document.createElement('style'); s.textContent = {}; \
         document.head.append(s); }})()",
        serde_json::to_string(css).unwrap()
    )
}

/// `RadioGroup` ignores ArrowDown - the shape of a group that handles only
/// the axis of its own layout. A capturing listener swallows the key before
/// the component or the browser sees it.
#[test]
fn radio_group_ignoring_arrow_down_fails_the_radio_set_contract() {
    block_on(async {
        must_fail(
            "/radio-group",
            Some(
                "window.addEventListener('keydown', e => { if (e.key === 'ArrowDown') { \
                 e.stopImmediatePropagation(); e.preventDefault(); } }, true)",
            ),
            "forward press 1 (ArrowDown)",
            |fixture| async move {
                let result = RadioSet {
                    radios: crate::radio_group::RADIOS,
                    checked: 1,
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

/// `SegmentedControl` whose arrows do nothing. `readonly` refuses them, which
/// is the defect exactly as a keyboard user meets it.
#[test]
fn segmented_control_with_dead_arrows_fails_the_radio_set_contract() {
    block_on(async {
        must_fail(
            "/planted/segmented-control-readonly",
            None,
            "focus is on radio Some(1), expected 2",
            |fixture| async move {
                let result = RadioSet {
                    radios: crate::segmented_control::RADIOS,
                    checked: 1,
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

/// `Menubar` with every trigger its own tab stop. The bar still works from a
/// keyboard, which is why this is worth a pass: it is three tab stops where
/// the pattern allows one.
#[test]
fn menubar_with_a_tab_stop_per_trigger_fails_the_roving_contract() {
    block_on(async {
        must_fail(
            "/menubar",
            Some(
                "document.querySelectorAll('[role=menubar] [data-menubar-index]')\
                 .forEach(el => el.setAttribute('tabindex', '0'))",
            ),
            "has 3 tab stops across 3 items",
            |fixture| async move {
                let result = RovingTabindex {
                    items: crate::menubar::TRIGGERS,
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

/// `Tabs` with a close button inside a tab: the defect the old count could
/// not see (review 7, E3).
///
/// Every `[role=tab]` still carries the right `tabindex`, so an attribute
/// count over the item selector reports one stop and the strip reports clean.
/// A keyboard user meets two - the tab, then its close button - and a strip of
/// twelve would be thirteen stops before the next control. The count has to be
/// taken by tabbing for this to be visible at all, which is why a plant on the
/// real component is the proof and not a fixture written to fail.
#[test]
fn tabs_with_a_tab_stop_inside_a_tab_fail_the_roving_contract() {
    block_on(async {
        must_fail(
            "/tabs",
            Some(
                "(() => { const plant = () => { \
                 const tab = document.querySelector('[role=tab]'); \
                 if (!tab || tab.querySelector('[data-planted-close]')) return; \
                 const close = document.createElement('button'); \
                 close.type = 'button'; close.textContent = 'x'; \
                 close.setAttribute('aria-label', 'Close tab'); \
                 close.setAttribute('data-planted-close', ''); \
                 tab.append(close); }; \
                 new MutationObserver(plant).observe(document.body, \
                 { subtree: true, childList: true }); \
                 plant(); })()",
            ),
            "has 2 tab stops across 3 items",
            |fixture| async move {
                let result = RovingTabindex {
                    items: crate::tabs::TAB,
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

/// `Tree` with every row out of the tab order: the other direction the count
/// has to tell apart.
///
/// Too many stops and none at all are different defects, and a count that
/// cannot report zero would give the same green for a keyboard-unreachable
/// widget as for a correct one - which is how the notification unit's missing
/// target selector read as coverage (todo 366). An observer keeps every
/// `[role=treeitem]` at `tabindex="-1"`, so nothing inside the tree is
/// reachable, and the count has to say `0` rather than erroring or defaulting
/// to the one stop the unit expects.
///
/// **This is evidence, not the guard.** The mechanism already distinguishes
/// zero: `count_tab_stops` bails when the item selector matches nothing, and
/// both `assert_single_tab_stop` and `tree::it_is_a_single_tab_stop` fail on
/// any count other than 1. What this test is for is the day someone changes
/// the counter - a budget, an early return, a "default to one if the walk
/// never entered" - and turns zero back into a green.
#[test]
fn tree_with_no_tab_stop_at_all_counts_zero() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        fixture
            .page
            .evaluate(
                // The guard on the write is not a nicety. `setAttribute` records
                // a mutation even when the value is unchanged, so an observer
                // that writes unconditionally re-triggers itself forever, the
                // main thread never yields, and every later CDP call fails as
                // `Timeout` - which reads as a slow machine rather than as a
                // plant that froze the page. Measured 2026-09-20: 661s.
                "(() => { const strip = () => document.querySelectorAll('[role=treeitem]')\
                 .forEach(el => { if (el.getAttribute('tabindex') !== '-1') \
                 el.setAttribute('tabindex', '-1'); }); \
                 new MutationObserver(strip).observe(document.body, \
                 { subtree: true, childList: true, attributes: true, \
                 attributeFilter: ['tabindex'] }); \
                 strip(); })()",
            )
            .await
            .unwrap();

        let stops = count_tab_stops(&fixture.page, "[role=treeitem]")
            .await
            .unwrap();
        let _ = fixture.close().await;

        assert_eq!(
            stops, 0,
            "a tree nothing can tab into should count 0 stops, not {stops}; \
             `tree::it_is_a_single_tab_stop` is what this makes fail"
        );
    });
}

/// `Select` whose rows lose their ids: todo 101's bug on the real component.
/// `aria-activedescendant` is present and well-formed and names nothing. An
/// observer strips every option's id as soon as the list renders it.
#[test]
fn select_with_a_dangling_activedescendant_fails_the_combobox_contract() {
    block_on(async {
        must_fail(
            "/select",
            Some(
                "(() => { const strip = () => document.querySelectorAll('[role=option][id]')\
                 .forEach(el => el.removeAttribute('id')); \
                 new MutationObserver(strip).observe(document.body, \
                 { subtree: true, childList: true, attributes: true, attributeFilter: ['id'] }); \
                 strip(); })()",
            ),
            "but no such element is in the DOM",
            |fixture| async move {
                let result = Combobox {
                    trigger: crate::select::TRIGGER,
                    option_count: 5,
                    tab_budget: 10,
                }
                .assert_contract(&fixture.page)
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `MultiSelect` with every focus indicator stripped: outline, shadow, and
/// the border colour a field frame changes on focus.
#[test]
fn multi_select_without_a_focus_indicator_fails_the_focus_ring_pass() {
    block_on(async {
        must_fail(
            "/multi-select",
            Some(&stylesheet(
                "*, *::before, *::after { outline: none !important; \
                 box-shadow: none !important; border-color: #888 !important; \
                 transition: none !important; }",
            )),
            "produced no visible ring",
            |fixture| async move {
                let result =
                    focus::assert_focus_ring(&fixture.page, crate::multi_select::TRIGGER, 10)
                        .await
                        .map(|_| ());
                (fixture, result)
            },
        )
        .await;
    });
}

/// `TagsField` with its suggestion rows squeezed to 12px.
#[test]
fn tags_field_with_squeezed_rows_fails_the_target_size_pass() {
    block_on(async {
        must_fail(
            "/tags-field",
            Some(&stylesheet(
                "[role=option] { height: 12px !important; min-height: 0 !important; \
                 padding-block: 0 !important; line-height: 12px !important; \
                 font-size: 10px !important; overflow: hidden !important; }",
            )),
            "target size below WCAG 2.5.8 minimum",
            |fixture| async move {
                let page = &fixture.page;
                let result = async {
                    keyboard::tab_to(page, crate::tags_field::TRIGGER, 10).await?;
                    keyboard::press(page, keyboard::ARROW_DOWN).await?;
                    wait::for_visible(page, "[role=listbox]").await?;
                    target_size::assert_minimum(page, "[role=option]").await
                }
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `Select`'s open list with `#ddd` text on white, checked where `Suite`
/// checks it: axe over `[data-fixture-ready]`.
///
/// **This plant found a hole in the harness rather than in the pass.** The
/// listbox is portaled beside the fixture, and the marker used to sit inside
/// the provider - so the list was outside the subtree axe ran on, and this test
/// stayed green. Every open-state contrast check in the suite had been
/// measuring the trigger alone. The marker now wraps the provider.
#[test]
fn select_with_faint_options_fails_the_contrast_pass_in_the_open_state() {
    block_on(async {
        must_fail(
            "/select",
            Some(&stylesheet("[role=option] { color: #dddddd !important; }")),
            "color-contrast",
            |fixture| async move {
                let page = &fixture.page;
                let result = async {
                    keyboard::tab_to(page, crate::select::TRIGGER, 10).await?;
                    keyboard::press(page, keyboard::ARROW_DOWN).await?;
                    wait::for_visible(page, "[role=listbox]").await?;
                    contrast::assert_clean(page, "[data-fixture-ready]").await
                }
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `Drawer` whose trigger is disabled while the drawer is open - a common way
/// to stop a double open, and one that leaves focus nowhere to return to.
///
/// Planted by an observer on the dialog appearing, not by a timer on the
/// Enter keydown. The timer version was green in a parallel run and red alone
/// (2026-09-19); tying the plant to the state it breaks removes the race,
/// whatever it was.
#[test]
fn drawer_with_a_disabled_trigger_fails_the_focus_return() {
    block_on(async {
        must_fail(
            "/drawer",
            Some(
                "new MutationObserver(() => { \
                 const trigger = document.querySelector('#open-drawer'); \
                 if (trigger && document.querySelector('[role=dialog]')) trigger.disabled = true; }) \
                 .observe(document.body, { subtree: true, childList: true })",
            ),
            "Escape closing the overlay, expected focus on #open-drawer",
            |fixture| async move {
                let result = Overlay {
                    trigger: crate::drawer::TRIGGER,
                    panel: "[role=dialog]",
                    traps_focus: true,
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

/// `Menu` whose items are `visibility: hidden` - `focus()` on one does nothing
/// and reports success, which is `codebase/use-popover`'s trap.
#[test]
fn menu_with_unfocusable_items_fails_the_overlay_contract() {
    block_on(async {
        must_fail(
            "/menu",
            Some(&stylesheet(
                "[role=menu] [role=menuitem] { visibility: hidden !important; }",
            )),
            "did not move focus into [role=menu]",
            |fixture| async move {
                let result = Overlay {
                    trigger: crate::menu::TRIGGER,
                    panel: "[role=menu]",
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

/// `Spotlight` whose focus trap never sees Tab: a capturing listener swallows
/// the event before the palette's handler, and the browser's own Tab walks
/// out of the dialog.
#[test]
fn spotlight_with_a_leaking_trap_fails_the_overlay_contract() {
    block_on(async {
        must_fail(
            "/spotlight",
            Some(
                "document.addEventListener('keydown', e => { \
                 if (e.key === 'Tab' && e.target.closest('[role=dialog]')) \
                 e.stopImmediatePropagation(); }, true)",
            ),
            "focus escaped [role=dialog]",
            |fixture| async move {
                let result = Overlay {
                    trigger: crate::spotlight::TRIGGER,
                    panel: "[role=dialog]",
                    traps_focus: true,
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

/// A phantom panel: when the overlay closes, an `opacity: 0` copy of it stays
/// behind, still in the accessibility tree. The check has to run at the close
/// signal and fail with the dismissal message. Behind a wait for the panel to
/// be hidden it failed as a timeout instead, or not at all (review 7, E6).
fn phantom(panel: &str) -> String {
    format!(
        "(() => {{ const sel = {}; let done = false; \
         new MutationObserver(records => {{ if (done) return; \
         for (const r of records) for (const n of r.removedNodes) {{ \
         if (n.nodeType !== 1) continue; \
         const hit = n.matches(sel) ? n : n.querySelector(sel); if (!hit) continue; \
         done = true; const ghost = hit.cloneNode(true); \
         ghost.style.opacity = '0'; ghost.style.pointerEvents = 'none'; \
         document.body.append(ghost); return; }} }}) \
         .observe(document.body, {{ subtree: true, childList: true }}); }})()",
        serde_json::to_string(panel).unwrap()
    )
}

/// The phantom on `Menu`, whose trigger reports the close in `aria-expanded`.
#[test]
fn menu_leaving_a_phantom_panel_fails_the_dismissal_check() {
    block_on(async {
        must_fail(
            "/menu",
            Some(&phantom("[role=menu]")),
            "is still in the accessibility tree",
            |fixture| async move {
                let result = Overlay {
                    trigger: crate::menu::TRIGGER,
                    panel: "[role=menu]",
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

/// The phantom on `Drawer`, whose trigger carries no state: focus returning
/// to it is the close signal.
#[test]
fn drawer_leaving_a_phantom_panel_fails_the_dismissal_check() {
    block_on(async {
        must_fail(
            "/drawer",
            Some(&phantom("[role=dialog]")),
            "is still in the accessibility tree",
            |fixture| async move {
                let result = Overlay {
                    trigger: crate::drawer::TRIGGER,
                    panel: "[role=dialog]",
                    traps_focus: true,
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

/// `Collapse` with its reduced-motion guard lost: a transition declared
/// outside the `prefers-reduced-motion` arm, which is the move
/// `collapse.rs` warns about.
#[test]
fn collapse_animating_under_reduced_motion_fails_the_motion_check() {
    block_on(async {
        must_fail(
            "/collapse",
            Some(&stylesheet(
                "@media (prefers-reduced-motion: reduce) { \
                 #details { transition: grid-template-rows 400ms ease !important; } }",
            )),
            "still animates under reduced motion",
            |fixture| async move {
                let result = async {
                    let page = &fixture.page;
                    motion::set_reduced_motion(page, true).await?;
                    motion::assert_reduced_motion_matches(page).await?;
                    pointer::click(page, crate::collapse::TOGGLE).await?;
                    wait::for_visible(page, "#details-text").await?;
                    motion::assert_still(page, crate::collapse::ROOT).await
                }
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}
