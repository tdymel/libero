//! Planted defects (311): each pass must fail on a real component's fixture, for the reason
//! named. Most plants are injected after mount (a stylesheet, a capturing listener).

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

/// `RadioGroup` ignores ArrowDown, as a group handling only its layout's axis would: a
/// capturing listener swallows the key.
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

/// `Menubar` with every trigger its own tab stop: still keyboard-usable, but three stops
/// where the pattern allows one.
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

/// `Tabs` with a close button inside a tab (review 7, E3): the `tabindex` attributes stay
/// right, so only a count taken by tabbing sees the second stop.
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

/// `Tree` with every row at `tabindex="-1"`: the count must say `0`, not default to one.
/// Evidence for the day someone changes the counter and turns zero green (366).
#[test]
fn tree_with_no_tab_stop_at_all_counts_zero() {
    block_on(async {
        let fixture = Fixture::open("/tree", Viewport::Desktop).await.unwrap();
        fixture
            .page
            .evaluate(
                // Guarded: `setAttribute` records a mutation even when unchanged, so an
                // unguarded observer loops forever and every CDP call times out (661 s).
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

/// `Select` whose rows lose their ids (todo 101): `aria-activedescendant` names nothing.
/// An observer strips every option's id as it renders.
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

/// `Select`'s open list with `#ddd` text, under axe over `[data-fixture-ready]`. It found
/// the portaled list outside that marker; the marker now wraps the provider.
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

/// `Spotlight`'s result list with `#ddd` text: todo 327's positive control. The scroll lock
/// hid every row from axe; it is lifted for the run, and this fails if the lift goes.
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

/// `contrast_covers` proved able to fail: the scroll lock pinned back inline (outranking the
/// harness's lift stylesheet) re-creates todo 327, and the message pins `8 of 8`.
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

/// The guard's disabled rule (386), proved narrow: `4 of 4`, only the `aria-disabled`
/// "Paste" dropped; `5 of 5` would mean the rule stopped working.
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

/// A closed `<details>` body keeps a rect in Chromium, but axe and the guard skip it (386):
/// clipped, only the summary is the hole, `1 of 2`, not `2 of 3`.
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

/// `Drawer` whose trigger is disabled while open, leaving focus nowhere to return. Planted
/// by an observer on the dialog, not a keydown timer, which raced (2026-09-19).
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

/// `Spotlight` whose focus trap never sees Tab: a capturing listener swallows it, and the
/// browser's Tab walks out of the dialog.
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

/// A phantom panel: an `opacity: 0` copy stays in the tree after close. The check runs at
/// the close signal, else it failed as a timeout or not at all (review 7, E6).
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

/// `Collapse` with a transition declared outside the `prefers-reduced-motion` arm.
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

/// `#444` text is 9.74:1 on white, ~2:1 on dark: red only in the dark run, proving that
/// run is really dark (314).
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

/// `Carousel`'s track guard with one arm out per plant (`crate::carousel::plant_arm_out`),
/// each aimed at the control only that arm saves.
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

/// Without `typing_target` the track eats the caret move, so the check fails on the caret,
/// the symptom a user meets first.
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

/// Without `arrow_target` a caller's `<input type="range">` stops stepping; no other arm
/// covers it.
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

/// `RadioGroup`'s rows crammed under the 24px pitch (302): todo 377's proof that
/// `targets_spaced` sees the unit's own rows as neighbours.
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

/// `FloatingWindow` whose title bar is deaf to the arrows (a capturing listener): nothing a
/// snapshot or axe sees changes.
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

/// `FloatingWindow` reporting the pre-move rect, the `owed` effect's defect. It skips the
/// value it last wrote: a flag is already cleared by the next microtask and hung the page.
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

/// `FloatingWindow` whose corner handle is deaf to Home and End only, so the arrows still
/// resize but the caller's bounds never answer.
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

/// `FloatingWindow`'s 20x20 close button with its 24x24 press box stripped (505, 566) and
/// crammed against the move handle, inside the 24px circle (377).
#[test]
fn a_floating_windows_crammed_title_bar_fails_the_spacing_exception() {
    block_on(async {
        must_fail(
            "/floating-window",
            Some(&stylesheet(
                "[data-window-title-bar] { gap: 0 !important; } \
                 [data-window-title-bar] button::before { content: none !important; } \
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

/// `FloatingWindow` with `#ddd` body text: proves the baseline's axe run reaches the
/// portaled window. Non-modal, so no scroll lock and no `UNLOCK` lift.
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
