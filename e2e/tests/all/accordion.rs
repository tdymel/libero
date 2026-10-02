//! `Accordion`: the arrows move focus and never toggle, Enter and Space toggle,
//! and every panel is a labelled region that a closed root hides.

use anyhow::{Result, ensure};
use e2e::archetypes::reset_tab_position;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::{keyboard, motion};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, js, wait};

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

/// Labels that wrap (todo 1773).
#[test]
fn the_long_labels_meet_the_baseline() {
    Suite::new("accordion-long", "/accordion-long")
        .focusable(TRIGGER)
        .targets(TRIGGER)
        .run();
}

/// Todo 517: the disabled trigger faded only, which forced colours undo.
#[test]
fn a_disabled_trigger_is_gray_text_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/accordion", Viewport::Desktop)
            .await
            .unwrap();
        e2e::browser::force_colours(&fixture.page).await.unwrap();
        crate::button::assert_gray_in_forced_colours(&fixture.page, "#checkout-trigger-1").await;
        fixture.close().await.unwrap();
    });
}

/// A label with no break opportunity wraps inside its trigger at 390px: nothing scrolls
/// sideways and the chevron stays on screen (1.4.10).
#[test]
fn a_long_label_wraps_instead_of_widening_the_page() {
    block_on(async {
        let fixture = Fixture::open("/accordion-long", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#long-trigger-0").await.unwrap();
        let widths: Vec<f64> = page
            .evaluate(
                "(() => {
                    const b = document.querySelector('#long-trigger-0');
                    const chevron = b.querySelector('[data-accordion-chevron]');
                    return [document.documentElement.scrollWidth, innerWidth,
                            b.scrollWidth, b.clientWidth,
                            chevron.getBoundingClientRect().right];
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [page_width, viewport, scroll, client, chevron] = widths[..] else {
            panic!("{widths:?}");
        };
        assert!(
            page_width <= viewport,
            "the page scrolls sideways: {widths:?}"
        );
        assert!(scroll <= client, "the trigger overflows: {widths:?}");
        assert!(chevron <= viewport, "the chevron is off screen: {widths:?}");
        fixture.console.assert_clean("a long label").unwrap();
        fixture.close().await.unwrap();
    });
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

/// The scenarios under both motion settings, because headless Chromium defaults to reduced
/// and a zero-duration close is where the focus return reads a DOM already gone; plus what
/// they do not read: the regions' names, the disabled tab stop, the chevron (todo 1826).
#[test]
fn the_keys_toggle_sections_and_only_an_open_panel_is_a_landmark() {
    block_on(async {
        e2e::browser::at_once([false, true], async |reduced| {
            let fixture = Fixture::open("/accordion", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            motion::set_reduced_motion(page, reduced).await.unwrap();
            if reduced {
                motion::assert_reduced_motion_matches(page).await.unwrap();
            }

            // Each panel is a region named by its trigger, which controls it.
            for index in 0..3 {
                assert_eq!(
                    region(page, index).await,
                    (
                        Some("region".into()),
                        Some(format!("checkout-trigger-{index}"))
                    ),
                    "panel {index}"
                );
                let controls: Option<String> = js(
                    page,
                    &format!(
                        "document.querySelector('{}').getAttribute('aria-controls')",
                        trigger(index)
                    ),
                )
                .await;
                assert_eq!(controls, Some(format!("checkout-region-{index}")));
            }
            // The disabled Payment is still a tab stop.
            reset_tab_position(page).await.unwrap();
            keyboard::tab_to(page, &trigger(1), 2).await.unwrap();

            let mut d = e2e::driver::Web { fixture };
            arrows_move_focus(&mut d, "/accordion").await.unwrap();
            enter_and_space_toggle(&mut d, "/accordion").await.unwrap();
            focus_returns(&mut d, "/accordion").await.unwrap();
            let page = &d.fixture.page;
            wait::for_js_true(page, &hidden(0), "Continue's closed root to hide")
                .await
                .unwrap();

            // The chevron turns, except under reduced motion: the chevron's own (todo 1830).
            match reduced {
                true => motion::assert_still(page, "#checkout")
                    .await
                    .expect("the accordion under reduced motion"),
                false => {
                    let moving = motion::assert_still(page, "#checkout [data-accordion-chevron]")
                        .await
                        .expect_err("the chevron should transition without reduced motion");
                    assert!(moving.to_string().contains("still animates"), "{moving}");
                }
            }

            d.finish(&format!("the accordion keys, reduced={reduced}"))
                .await
                .unwrap();
        })
        .await;
    });
}

fn trigger(index: usize) -> String {
    format!("#checkout-trigger-{index}")
}

async fn is_expanded<D: Driver>(d: &mut D, index: usize) -> Result<bool> {
    Ok(d.attr(&trigger(index), "aria-expanded").await?.as_deref() == Some("true"))
}

async fn is_hidden<D: Driver>(d: &mut D, index: usize) -> Result<bool> {
    let region = format!("#checkout-region-{index}");
    Ok(d.style(&region, "visibility").await? == "hidden")
}

async fn until_expanded<D: Driver>(d: &mut D, index: usize, open: bool, after: &str) -> Result<()> {
    eventually(
        d,
        &format!("{after}: panel {index} open={open}"),
        async |d| Ok(is_expanded(d, index).await? == open),
    )
    .await
}

async fn arrows_move_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(FIRST).await?;
    for (key, to) in [
        (keyboard::ARROW_DOWN, 2),
        (keyboard::ARROW_DOWN, 0),
        (keyboard::ARROW_UP, 2),
        (keyboard::HOME, 0),
        (keyboard::END, 2),
    ] {
        d.press(key).await?;
        eventually_focused(d, &trigger(to), key.key).await?;
    }
    ensure!(!is_expanded(d, 2).await?, "an arrow toggled Review");
    Ok(())
}

async fn enter_and_space_toggle<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for index in 0..3 {
        ensure!(is_hidden(d, index).await?, "closed panel {index} shows");
    }
    d.focus(&trigger(1)).await?;
    d.press(keyboard::ENTER).await?;
    d.focus(FIRST).await?;
    d.press(keyboard::ENTER).await?;
    until_expanded(d, 0, true, "Enter on Shipping").await?;
    ensure!(
        !is_expanded(d, 1).await?,
        "Enter opened the disabled Payment"
    );
    eventually(d, "the open panel to show Continue", async |d| {
        Ok(!is_hidden(d, 0).await? && d.exists(CONTINUE).await?)
    })
    .await?;

    d.press(keyboard::SPACE).await?;
    until_expanded(d, 0, false, "Space on Shipping").await?;
    eventually(d, "the closed panel to unmount and hide", async |d| {
        Ok(!d.exists(CONTINUE).await? && is_hidden(d, 0).await?)
    })
    .await
}

async fn a_click_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(&trigger(2)).await?;
    until_expanded(d, 2, true, "a click on Review").await?;
    d.click(&trigger(2)).await?;
    until_expanded(d, 2, false, "a second click on Review").await
}

/// Continue opens Review, which closes Shipping around the focused button.
async fn focus_returns<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(FIRST).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Continue to show", async |d| d.exists(CONTINUE).await).await?;
    d.focus(CONTINUE).await?;
    d.press(keyboard::ENTER).await?;
    until_expanded(d, 2, true, "Continue").await?;
    eventually_focused(d, FIRST, "Shipping closing around Continue").await
}

e2e::scenario!(
    the_arrows_move_focus_skip_the_disabled_and_never_toggle,
    "/accordion",
    arrows_move_focus,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    enter_opens_space_closes_and_a_closed_panel_hides,
    "/accordion",
    enter_and_space_toggle
);
e2e::scenario!(a_click_toggles_a_section, "/accordion", a_click_toggles);
e2e::scenario!(
    focus_returns_to_the_trigger_of_a_panel_that_closes_around_it,
    "/accordion",
    focus_returns,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);

/// Ctrl/Alt/Meta chords are the browser's: no focus move, no toggle.
#[test]
fn modifier_chords_go_to_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/accordion", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, FIRST, 10).await.unwrap();
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_DOWN,
                keyboard::ARROW_UP,
                keyboard::HOME,
                keyboard::END,
            ],
            &format!(
                "[document.activeElement.id, ...[...document.querySelectorAll('{TRIGGER}')]\
                 .map(t => t.getAttribute('aria-expanded'))]"
            ),
        )
        .await
        .unwrap();

        fixture.console.assert_clean("accordion chords").unwrap();
        fixture.close().await.unwrap();
    });
}
