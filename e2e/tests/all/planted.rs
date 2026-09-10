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
//! (`fixtures/src/negative.rs`).
//!
//! Most plants are injected from here after the page mounts - a stylesheet or a
//! capturing listener - so the library is untouched and the fixture is the one
//! its own unit runs against. One needs a prop, and has a route of its own.

use e2e::archetypes::{Combobox, Orientation, Overlay, RadioSet, RovingTabindex, count_tab_stops};
use e2e::browser::block_on;
use e2e::passes::{contrast, focus, keyboard, motion, pointer, target_size};
use e2e::{Fixture, Scheme, Viewport, wait};

use crate::carousel;

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

/// `Spotlight`'s open result list with `#ddd` text, checked where `Suite`
/// checks it.
///
/// **This is the positive control for todo 327.** This exact plant was made on
/// 2026-09-19 and stayed green: `Modal`'s `body { overflow: hidden }` scroll
/// lock made axe's `overflowHidden` judge every row below the search box
/// clipped away by a content-sized body, so `color-contrast` was inapplicable
/// to the entire list. The lock is lifted for the axe run now, and this test is
/// what says so - it fails if the lift is ever removed, because the rows go
/// back to being unreachable and no violation is reported.
#[test]
fn spotlight_with_faint_rows_fails_the_contrast_pass_in_the_open_state() {
    block_on(async {
        must_fail(
            "/spotlight",
            Some(&stylesheet(
                "[role=dialog] [role=option] * { color: #dddddd !important; }",
            )),
            "color-contrast",
            |fixture| async move {
                let page = &fixture.page;
                let result = async {
                    keyboard::tab_to(page, crate::spotlight::TRIGGER, 10).await?;
                    keyboard::press(page, keyboard::ENTER).await?;
                    wait::for_visible(page, "[role=dialog] [role=option]").await?;
                    contrast::assert_clean(page, "[data-fixture-ready]").await
                }
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// The coverage guard itself, proved able to fail.
///
/// `contrast_covers` asserts axe *evaluated* text rather than merely finding
/// nothing wrong with it, so its own defect is text axe cannot reach. Pinning
/// the scroll lock back on reproduces todo 327 exactly, and the guard has to
/// name the rows it could not account for.
///
/// An **inline** `!important` is the plant, not a stylesheet: the harness lifts
/// the lock with a stylesheet of its own appended at run time, which would win
/// the cascade against any rule planted earlier. A style attribute outranks
/// both. The general rule is in `principles/browser-harness-traps`: if the
/// harness injects style at run time, a plant must outrank it inline or it is
/// not a plant at all - it is a test of the harness's own stylesheet.
///
/// The expected message pins the **count**, `8 of 8`, not just the wording
/// (the shortcut's "+" is punctuation, which axe and the guard both skip). A
/// plant that only asserts "something was missed" would still pass if the lift
/// half-worked and one row slipped through, which is the shape of failure this
/// whole todo was.
#[test]
fn spotlight_rows_axe_cannot_reach_fail_the_coverage_guard() {
    block_on(async {
        must_fail(
            "/spotlight",
            Some("document.body.style.setProperty('overflow', 'hidden', 'important')"),
            "8 of 8 on-screen text element(s) under [role=dialog] were never evaluated by \
             axe's `color-contrast` rule",
            |fixture| async move {
                let page = &fixture.page;
                let result = async {
                    keyboard::tab_to(page, crate::spotlight::TRIGGER, 10).await?;
                    keyboard::press(page, keyboard::ENTER).await?;
                    wait::for_visible(page, "[role=dialog] [role=option]").await?;
                    contrast::assert_covers(page, "[data-fixture-ready]", "[role=dialog]")
                        .await
                        .map(|_| ())
                }
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// A zero-height `overflow: hidden` menu: axe's `overflowHidden` calls every row
/// clipped, while each row keeps its own on-screen rect. A hole for the guard.
const MENU_CLIPPED: &str = "[role=menu] { overflow: hidden !important; height: 0 !important; \
     min-height: 0 !important; padding: 0 !important; border: 0 !important; }";

/// Open `/menu`, run `before` (a plant that must leave the guard green), then clip
/// the menu and run the guard again, which must fail.
async fn menu_coverage_with(fixture: &Fixture, before: &str) -> anyhow::Result<()> {
    let page = &fixture.page;
    keyboard::tab_to(page, crate::menu::TRIGGER, 10).await?;
    keyboard::press(page, keyboard::ARROW_DOWN).await?;
    wait::for_visible(page, "[role=menu]").await?;
    page.evaluate(before).await?;
    contrast::assert_covers(page, "[data-fixture-ready]", "[role=menu]")
        .await
        .map_err(|e| anyhow::anyhow!("green half: {e:#}"))?;
    page.evaluate(stylesheet(MENU_CLIPPED)).await?;
    contrast::assert_covers(page, "[data-fixture-ready]", "[role=menu]")
        .await
        .map(|_| ())
}

/// The guard's disabled rule (todo 386), proved narrow. The clipped menu has to
/// name all four enabled labels, `4 of 4`: the `aria-disabled` "Paste" is the
/// only one dropped, and `5 of 5` would mean the rule stopped working.
#[test]
fn a_clipped_menu_fails_the_coverage_guard_on_its_enabled_rows_only() {
    block_on(async {
        must_fail(
            "/menu",
            None,
            "4 of 4 on-screen text element(s) under [role=menu] were never evaluated",
            |fixture| async move {
                let result = menu_coverage_with(&fixture, "0").await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// An `inert` row is skipped by axe and by the guard (todo 386): green alone,
/// then `3 of 3` once the menu is clipped, not `4 of 4`.
#[test]
fn an_inert_menu_row_is_no_hole_but_a_clipped_menu_still_is() {
    block_on(async {
        must_fail(
            "/menu",
            None,
            "3 of 3 on-screen text element(s) under [role=menu] were never evaluated",
            |fixture| async move {
                let result = menu_coverage_with(
                    &fixture,
                    "Array.from(document.querySelectorAll('[role=menuitem]'))\
                     .find(el => el.textContent.includes('Save')).inert = true",
                )
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// A closed `<details>` body keeps a rect in Chromium, but axe skips it and the
/// guard has to as well (todo 386). Green alone; clipping its wrapper leaves only
/// the summary as the hole, `1 of 2`, where `2 of 3` would count the body.
#[test]
fn a_closed_details_body_is_no_hole_but_its_clipped_summary_is() {
    block_on(async {
        must_fail(
            "/menu",
            Some(
                "(() => { const box = document.createElement('div'); box.id = 'planted-box'; \
                 box.innerHTML = '<details><summary>More</summary><p>Folded words</p></details>'; \
                 document.querySelector('[data-fixture-ready]').append(box); })()",
            ),
            "1 of 2 on-screen text element(s) under [data-fixture-ready] were never evaluated",
            |fixture| async move {
                let page = &fixture.page;
                let result = async {
                    contrast::assert_covers(page, "[data-fixture-ready]", "[data-fixture-ready]")
                        .await
                        .map_err(|e| anyhow::anyhow!("green half: {e:#}"))?;
                    page.evaluate(stylesheet(
                        "#planted-box { overflow: hidden !important; height: 0 !important; }",
                    ))
                    .await?;
                    contrast::assert_covers(page, "[data-fixture-ready]", "[data-fixture-ready]")
                        .await
                        .map(|_| ())
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

/// A text colour fixed for the light page: `#444` is 9.74:1 on white and
/// about 2:1 on the dark surface. Green in the light run and red in the dark
/// one, so the dark run measures a page that is really dark (todo 314).
#[test]
fn a_colour_fixed_for_the_light_page_fails_only_in_the_dark_run() {
    block_on(async {
        let plant = stylesheet("#presses { color: #444 !important; }");
        let mut outcomes = Vec::new();
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open_in("/button", Viewport::Desktop, scheme)
                .await
                .unwrap_or_else(|e| panic!("opening /button ({}): {e}", scheme.name()));
            fixture.page.evaluate(plant.as_str()).await.unwrap();
            let outcome = contrast::assert_clean(&fixture.page, "[data-fixture-ready]").await;
            let _ = fixture.close().await;
            outcomes.push(outcome.map_err(|e| format!("{e:#}")));
        }
        assert!(
            outcomes[0].is_ok(),
            "the plant fails on the light page too, so it proves nothing about the dark run: {:?}",
            outcomes[0]
        );
        match &outcomes[1] {
            Err(error) => assert!(
                error.contains("color-contrast") && error.contains("#444444"),
                "the dark run failed, but not on the planted colour: {error}"
            ),
            Ok(()) => {
                panic!("the dark run stayed green on #444 text: it is not measuring a dark page")
            }
        }
    });
}

/// A page pinned to the light theme, the way a stored `lsx-color-scheme`
/// pins it, under dark emulation: the scheme check must refuse it.
#[test]
fn a_page_pinned_light_under_dark_emulation_fails_the_scheme_check() {
    block_on(async {
        let fixture = Fixture::open_in("/button", Viewport::Desktop, Scheme::Dark)
            .await
            .unwrap_or_else(|e| panic!("opening /button (dark): {e}"));
        fixture
            .page
            .evaluate("document.documentElement.setAttribute('data-lsx-theme', 'light')")
            .await
            .unwrap();
        let outcome = fixture.assert_scheme().await;
        let _ = fixture.close().await;
        let error = format!(
            "{:#}",
            outcome.expect_err("a light-pinned page passed as dark")
        );
        assert!(
            error.contains("not drawn in the dark scheme"),
            "failed, but not for the pin: {error}"
        );
    });
}

/// `Carousel`'s track guard with one arm taken out, three times over.
///
/// The arms are `key_taken || typing_target || arrow_target` in
/// `carousel.rs`, and each covers a different kind of control in a slide, so
/// removing all three at once would prove only that the checks see *a*
/// broken carousel. One plant per arm, each aimed at the control only that arm
/// can save, is what says the three checks are three checks.
/// `crate::carousel::plant_arm_out` is how an arm is taken out of a running
/// page.
#[test]
fn a_carousel_that_ignores_key_taken_advances_under_its_own_slider() {
    block_on(async {
        must_fail(
            "/carousel",
            Some(&carousel::plant_arm_out(carousel::KEY_TAKEN)),
            "the carousel advanced on an arrow on a slider in a slide",
            |fixture| async move {
                let result = carousel::the_slider_in_a_slide_takes_the_arrows(&fixture).await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// Without `typing_target` the track's `prevent_default()` eats the caret
/// move, so the check fails on the caret before it reaches the strip - which
/// is the symptom a user meets first.
#[test]
fn a_carousel_that_ignores_typing_target_eats_a_caret_move() {
    block_on(async {
        must_fail(
            "/carousel",
            Some(&carousel::plant_arm_out(carousel::TYPING_TARGET)),
            "the caret in the slide's text field did not move",
            |fixture| async move {
                let result = carousel::the_caret_moves_in_a_slides_text_field(&fixture).await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// Without `arrow_target` a caller's own `<input type="range">` stops
/// stepping: nothing in the library marks that press, and it is not typing, so
/// no other arm covers it.
#[test]
fn a_carousel_that_ignores_arrow_target_stops_a_raw_range() {
    block_on(async {
        must_fail(
            "/carousel",
            Some(&carousel::plant_arm_out(carousel::ARROW_TARGET)),
            "the raw range in the slide did not step",
            |fixture| async move {
                let result = carousel::a_raw_range_in_a_slide_steps(&fixture).await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `RadioGroup`'s rows crammed back under the 24px pitch todo 302 fixed.
///
/// This is the proof for todo 377, and the reason
/// `radio_group::it_meets_the_baseline` may declare its rows with
/// `targets_spaced` at all. Before the neighbour set included the unit's own
/// declared selector, the pass was **green on this page**: a row's only
/// neighbours were the `1x1` hidden inputs, and the adjacent rows - bare
/// `div`s - were in no selector the pass looked at.
#[test]
fn radio_group_rows_crammed_together_fail_the_spacing_exception() {
    block_on(async {
        must_fail(
            "/radio-group",
            Some(&stylesheet(
                "[role=radiogroup] { gap: 0 !important; row-gap: 0 !important; } \
                 [role=radiogroup] > div { height: 10px !important; min-height: 0 !important; }",
            )),
            "spacing exception fails",
            |fixture| async move {
                let result =
                    target_size::assert_minimum_or_spacing(&fixture.page, crate::radio_group::ROWS)
                        .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `FloatingWindow` whose title bar is deaf to the arrows: a capturing
/// listener swallows the key before the handle's own `onkeydown` sees it.
///
/// The window still opens, still takes focus, and still looks right - which is
/// the point. A keyboard move that quietly stopped working would change
/// nothing a snapshot or an axe run can see, so this is the plant for the half
/// of the unit that only a browser can test.
#[test]
fn a_floating_window_with_a_deaf_title_bar_never_moves() {
    block_on(async {
        must_fail(
            "/floating-window",
            Some(
                "document.addEventListener('keydown', e => { \
                 if (e.target.closest('[data-window-handle]')) e.stopImmediatePropagation(); }, true)",
            ),
            "the window to move by (10.0, 0.0)",
            |fixture| async move {
                let result = crate::floating_window::keyboard_move(&fixture.page).await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `FloatingWindow` reporting the rect it had *before* the move.
///
/// This is the defect the component's `owed` effect exists to prevent: reading
/// the rect in the same task as the write reports the previous one
/// (`codebase/components/floating-window`). The plant re-creates it without
/// touching the library - a `MutationObserver` rewrites the readout's `x` one
/// move back, so every report is exactly one step stale.
///
/// It remembers what it last wrote and skips that value, rather than guarding
/// with a flag: a `MutationObserver` callback is a microtask, so a flag set and
/// cleared synchronously is already false by the time the next one runs, and
/// the first version of this plant rewrote the node for ever and hung the page
/// (the run reported "Request timed out", not the planted reason).
///
/// Ten pixels is the whole difference, and nothing on the page shows it. The
/// unit catches it only because it compares what the component *said* against
/// the rect the browser *drew*, rather than only asserting that the report
/// changed.
#[test]
fn a_floating_window_reporting_a_stale_rect_fails_the_move_report() {
    block_on(async {
        must_fail(
            "/floating-window",
            Some(
                "(() => { let written = null; \
                 new MutationObserver(() => { const el = document.querySelector('#move-report'); \
                 if (!el) return; const now = el.textContent; if (now === written) return; \
                 const parts = now.trim().split(' '); if (parts.length !== 4) return; \
                 written = [Number(parts[0]) - 10, parts[1], parts[2], parts[3]].join(' '); \
                 el.textContent = written; }) \
                 .observe(document.body, { subtree: true, childList: true, characterData: true }); })()",
            ),
            "but it reported",
            |fixture| async move {
                let result = crate::floating_window::keyboard_move(&fixture.page).await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `FloatingWindow` whose corner handle is deaf to Home and End, so the two
/// presses that ask for `0x0` and `u16::MAX` never reach the geometry and the
/// caller's own bounds are never what answers.
///
/// The arrow step is left alone, so the separator still resizes: this plant
/// removes the clamp alone, which is the half a reviewer would not notice
/// missing.
#[test]
fn a_floating_window_that_ignores_home_on_its_separator_never_reaches_the_minimum() {
    block_on(async {
        must_fail(
            "/floating-window",
            Some(
                "document.addEventListener('keydown', e => { \
                 if ((e.key === 'Home' || e.key === 'End') && e.target.closest('[role=separator]')) \
                 e.stopImmediatePropagation(); }, true)",
            ),
            "Home to size the window to 240x120",
            |fixture| async move {
                let result = crate::floating_window::keyboard_resize(&fixture.page).await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `FloatingWindow`'s close button crammed against the move handle.
///
/// It is 20x20 - an `ActionIcon` at `size: "sm"` - so it is under WCAG 2.5.8's
/// 24x24 outright and `floating_window::it_meets_the_baseline` declares it with
/// `targets_spaced`. A spacing exception is silently green on a target with no
/// neighbours in reach, so the declaration is only worth having once something
/// has watched it fail on **this** component (todo 377). The plant takes the
/// title bar's gap away and shrinks the move handle from `flex: 1` to a strip,
/// which brings the two centres inside the 24px circle.
#[test]
fn a_floating_windows_crammed_title_bar_fails_the_spacing_exception() {
    block_on(async {
        must_fail(
            "/floating-window",
            Some(&stylesheet(
                "[data-window-title-bar] { gap: 0 !important; } \
                 [data-window-handle] { flex: 0 0 8px !important; min-width: 0 !important; }",
            )),
            "spacing exception fails",
            |fixture| async move {
                let page = &fixture.page;
                let result = async {
                    keyboard::tab_to(page, crate::floating_window::TRIGGER, 8).await?;
                    keyboard::press(page, keyboard::ENTER).await?;
                    wait::for_visible(page, crate::floating_window::DIALOG).await?;
                    target_size::assert_minimum_or_spacing(page, crate::floating_window::CLOSE)
                        .await
                }
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}

/// `FloatingWindow` with `#ddd` text in its body, checked where `Suite` checks
/// it: axe over `[data-fixture-ready]`.
///
/// A window is portaled beside the fixture, which is the arrangement that left
/// every overlay's axe pass measuring the trigger alone until `dc3766db`, and
/// the one that made todo 327's scroll lock invisible. Neither hole reports
/// anything - both read as a clean run - so the only proof that
/// `floating_window::it_meets_the_baseline` looks at the window at all is a
/// defect planted in the window that it has to catch.
///
/// The window is **not** modal, so it locks no scroll and needs no `UNLOCK`
/// lift; this is the plain half of the pair.
#[test]
fn a_floating_window_with_faint_text_fails_the_contrast_pass_in_the_open_state() {
    block_on(async {
        must_fail(
            "/floating-window",
            Some(&stylesheet(
                "[role=dialog] [data-window-body] * { color: #dddddd !important; }",
            )),
            "color-contrast",
            |fixture| async move {
                let page = &fixture.page;
                let result = async {
                    keyboard::tab_to(page, crate::floating_window::TRIGGER, 8).await?;
                    keyboard::press(page, keyboard::ENTER).await?;
                    wait::for_visible(page, crate::floating_window::DIALOG).await?;
                    contrast::assert_clean(page, "[data-fixture-ready]").await
                }
                .await;
                (fixture, result)
            },
        )
        .await;
    });
}
