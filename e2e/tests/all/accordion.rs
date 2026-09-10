//! `Accordion`: the arrows move focus and never toggle, Enter and Space toggle,
//! and every panel is a labelled region that a closed root hides.

use e2e::archetypes::reset_tab_position;
use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, motion};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

use chromiumoxide::Page;

const TRIGGER: &str = "[data-accordion-heading] > button";
const FIRST: &str = "#checkout-trigger-0";
const CONTINUE: &str = "#continue";

#[test]
fn it_meets_the_baseline() {
    Suite::new("accordion", "/accordion")
        .focusable(FIRST)
        .targets(TRIGGER)
        .state(
            "open",
            &[Step::TabTo(FIRST), Step::Press(keyboard::ENTER)],
            CONTINUE,
        )
        .run();
}

/// `true` once trigger `index` reports `aria-expanded` as `expanded`.
fn expanded(index: usize, expanded: bool) -> String {
    format!(
        "document.querySelector('#checkout-trigger-{index}').getAttribute('aria-expanded') === '{expanded}'"
    )
}

/// `true` once region `index`'s root is `visibility: hidden`, which takes the
/// landmark out of the accessibility tree.
fn hidden(index: usize) -> String {
    format!(
        "getComputedStyle(document.querySelector('#checkout-region-{index}')).visibility === 'hidden'"
    )
}

/// The region's role and label, as `[role, aria-labelledby]`.
async fn region(page: &Page, index: usize) -> (Option<String>, Option<String>) {
    page.evaluate(format!(
        "(() => {{ const r = document.querySelector('#checkout-region-{index}'); \
         return [r.getAttribute('role'), r.getAttribute('aria-labelledby')]; }})()"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

async fn is(page: &Page, expression: &str) -> bool {
    page.evaluate(expression)
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn expect_focus(page: &Page, id: &str, what: &str) {
    let arrived = wait::for_js_true(
        page,
        &format!("document.activeElement && document.activeElement.id === '{id}'"),
        what,
    )
    .await;
    if let Err(e) = arrived {
        panic!("{e}; focus is on {:?}", focus::active_element(page).await);
    }
}

async fn press_and_expect_focus(page: &Page, key: keyboard::Key, id: &str) {
    keyboard::press(page, key).await.unwrap();
    expect_focus(page, id, &format!("{} to focus #{id}", key.key)).await;
}

/// Both motion settings, because headless Chromium defaults to reduced and a
/// zero-duration close is where the focus return reads a DOM already gone.
#[test]
fn the_keys_toggle_sections_and_only_an_open_panel_is_a_landmark() {
    block_on(async {
        for reduced in [false, true] {
            let fixture = Fixture::open("/accordion", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            motion::set_reduced_motion(page, reduced).await.unwrap();
            if reduced {
                motion::assert_reduced_motion_matches(page).await.unwrap();
            }

            // Closed: a labelled region, hidden by `Collapse`'s root.
            reset_tab_position(page).await.unwrap();
            press_and_expect_focus(page, keyboard::TAB, "checkout-trigger-0").await;
            for index in 0..3 {
                assert!(
                    is(page, &expanded(index, false)).await,
                    "panel {index} starts open"
                );
                assert_eq!(
                    region(page, index).await,
                    (
                        Some("region".into()),
                        Some(format!("checkout-trigger-{index}"))
                    ),
                    "closed panel {index}"
                );
                assert!(is(page, &hidden(index)).await, "closed panel {index} shows");
            }

            // Arrows skip the disabled Payment, wrap, and never toggle.
            press_and_expect_focus(page, keyboard::ARROW_DOWN, "checkout-trigger-2").await;
            press_and_expect_focus(page, keyboard::ARROW_DOWN, "checkout-trigger-0").await;
            press_and_expect_focus(page, keyboard::ARROW_UP, "checkout-trigger-2").await;
            press_and_expect_focus(page, keyboard::HOME, "checkout-trigger-0").await;
            press_and_expect_focus(page, keyboard::END, "checkout-trigger-2").await;
            assert!(
                is(page, &expanded(2, false)).await,
                "an arrow toggled Review"
            );

            // The disabled trigger is a tab stop that Enter does not open.
            // Shift+Tab back and open the first: once it reads open, the
            // disabled trigger's Enter has been handled too.
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            expect_focus(
                page,
                "checkout-trigger-1",
                "Shift+Tab to the disabled Payment",
            )
            .await;
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            expect_focus(page, "checkout-trigger-0", "Shift+Tab to Shipping").await;
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_js_true(page, &expanded(0, true), "Enter to open Shipping")
                .await
                .unwrap();
            assert!(
                is(page, &expanded(1, false)).await,
                "Enter opened the disabled Payment"
            );

            // Open: a region named by its trigger, and `aria-controls` finds it.
            wait::for_visible(page, CONTINUE).await.unwrap();
            assert!(!is(page, &hidden(0)).await, "the open panel is hidden");
            assert_eq!(
                region(page, 0).await,
                (Some("region".into()), Some("checkout-trigger-0".into()))
            );
            let controls: String = page
                .evaluate(format!(
                    "document.querySelector('{FIRST}').getAttribute('aria-controls')"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(controls, "checkout-region-0");

            // Space closes it again, and the root hides once the close ends.
            keyboard::press(page, keyboard::SPACE).await.unwrap();
            wait::for_js_true(page, &expanded(0, false), "Space to close Shipping")
                .await
                .unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('#continue')",
                "the closed panel to unmount",
            )
            .await
            .unwrap();
            wait::for_js_true(page, &hidden(0), "the closed root to hide")
                .await
                .unwrap();

            // Focus return: Continue opens Review and closes the panel it sits in.
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_visible(page, CONTINUE).await.unwrap();
            press_and_expect_focus(page, keyboard::TAB, "continue").await;
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_js_true(page, &expanded(2, true), "Continue to open Review")
                .await
                .unwrap();
            expect_focus(
                page,
                "checkout-trigger-0",
                "focus to return to Shipping's trigger",
            )
            .await;
            wait::for_js_true(page, &hidden(0), "Continue's closed root to hide")
                .await
                .unwrap();

            // The chevron turns, except under reduced motion.
            let still = motion::assert_still(page, "#checkout").await;
            match reduced {
                true => still.expect("the accordion under reduced motion"),
                false => {
                    still.expect_err("the chevron should transition without reduced motion");
                }
            }

            fixture
                .console
                .assert_clean(&format!("the accordion keys, reduced={reduced}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
