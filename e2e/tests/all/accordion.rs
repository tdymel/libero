//! `Accordion`: the arrows move focus and never toggle, Enter and Space toggle,
//! and every panel is a labelled region that a closed root hides.

use anyhow::{Result, ensure};
use e2e::archetypes::reset_tab_position;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::{keyboard, motion, pointer};
use e2e::suite::Step;
use e2e::wait::PageBounded;
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
            .evaluate_bounded(
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

/// Todo 2430: an unbreakable word wraps and a 600px child scrolls inside the body at
/// 390px, instead of being clipped by `Collapse` or widening the page (1.4.10).
#[test]
fn wide_panel_content_wraps_or_scrolls_inside_the_panel() {
    block_on(async {
        let fixture = Fixture::open("/accordion-wide", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#wide-child").await.unwrap();
        let widths: Vec<f64> = page
            .evaluate_bounded(
                "(() => {
                    const body = document.querySelector('#wide [data-accordion-body]');
                    const word = document.querySelector('#word');
                    body.scrollLeft = 1e4;
                    return [document.documentElement.scrollWidth, innerWidth,
                            word.scrollWidth, word.clientWidth, body.scrollLeft];
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [page_width, viewport, word, word_box, scrolled] = widths[..] else {
            panic!("{widths:?}");
        };
        assert!(
            page_width <= viewport,
            "the page scrolls sideways: {widths:?}"
        );
        assert!(word <= word_box, "the word overflows: {widths:?}");
        assert!(
            scrolled > 0.0,
            "the wide child cannot be scrolled to: {widths:?}"
        );
        fixture.console.assert_clean("a wide panel").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2431: a panel's first focusable keeps its ring's top edge inside the clip.
#[test]
fn a_first_child_in_a_panel_keeps_its_ring() {
    block_on(async {
        let fixture = Fixture::open("/accordion-wide", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#first", 10).await.unwrap();
        let clipped: f64 = js(page, &crate::image_list::ring_clipped("#wide")).await;
        assert!(
            clipped <= 0.0,
            "the ring runs {clipped}px past the panel's clip"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 2432: with one enabled trigger the arrows and Home/End have nowhere to move
/// focus, so they stay the page's.
#[test]
fn a_lone_trigger_leaves_the_arrows_to_the_page() {
    block_on(async {
        let fixture = Fixture::open("/accordion-lone", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#lone-trigger-0", 10).await.unwrap();
        assert_keys_left_to_page(
            page,
            &[
                keyboard::ARROW_DOWN,
                keyboard::END,
                keyboard::ARROW_UP,
                keyboard::HOME,
            ],
        )
        .await;
        fixture.console.assert_clean("a lone trigger").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2889: Home on the first trigger and End on the last stay the page's, while the
/// same keys elsewhere still move focus.
#[test]
fn home_and_end_at_the_ends_leave_the_page_to_scroll() {
    block_on(async {
        let fixture = Fixture::open("/accordion-ends", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#ends-trigger-0", 10).await.unwrap();
        assert_keys_left_to_page(page, &[keyboard::HOME]).await;
        keyboard::press(page, keyboard::END).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.id === 'ends-trigger-2'",
            "focus on the last trigger",
        )
        .await
        .unwrap();
        assert_keys_left_to_page(page, &[keyboard::END]).await;
        fixture.console.assert_clean("the ends").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4140: with `collapsible: false` the open trigger does nothing and is `aria-disabled`;
/// opening another moves the pin.
#[test]
fn a_non_collapsible_accordion_keeps_one_section_open() {
    block_on(async {
        let fixture = Fixture::open("/accordion-pinned", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let state = |index: usize| {
            format!(
                "(() => {{ const b = document.querySelector('#pinned-trigger-{index}'); \
                 return [b.getAttribute('aria-expanded'), b.getAttribute('aria-disabled')]; }})()"
            )
        };
        pointer::click(page, "#pinned-trigger-0").await.unwrap();
        e2e::clock::frame(page).await.unwrap();
        let unmoved: Vec<String> = js(page, &state(0)).await;
        assert_eq!(unmoved, ["true", "true"], "the pinned section closed");
        pointer::click(page, "#pinned-trigger-1").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#pinned-trigger-1').getAttribute('aria-expanded') === 'true'",
            "the second section to open",
        )
        .await
        .unwrap();
        let first: Vec<String> = js(page, &state(0)).await;
        let second: Vec<String> = js(page, &state(1)).await;
        assert_eq!(first, ["false", "false"], "the first closed and unpinned");
        assert_eq!(second, ["true", "true"], "the second open and pinned");
        // Focusable though inert: the tab stop stays.
        keyboard::tab_to(page, "#pinned-trigger-1", 10)
            .await
            .unwrap();
        fixture.console.assert_clean("a pinned accordion").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 4141, 4142: the chevron leads the label and stays down on an open section.
#[test]
fn the_chevron_can_lead_and_stay_still() {
    block_on(async {
        let fixture = Fixture::open("/accordion-chevron", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#chevron-trigger-0").await.unwrap();
        let [chevron_right, label_left]: [f64; 2] = js(
            page,
            "(() => { const b = document.querySelector('#chevron-trigger-0'); \
             const c = b.querySelector('[data-accordion-chevron]'); \
             const l = b.querySelector('span:not([data-accordion-chevron])'); \
             return [c.getBoundingClientRect().right, l.getBoundingClientRect().left]; })()",
        )
        .await;
        assert!(
            chevron_right <= label_left,
            "the chevron is not before the label: {chevron_right} vs {label_left}"
        );
        let open: String = js(
            page,
            "getComputedStyle(document.querySelector('#chevron-trigger-0 [data-accordion-chevron]')).transform",
        )
        .await;
        assert_eq!(open, "none", "the open chevron turned");
        fixture.console.assert_clean("a leading chevron").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Presses `keys` on the focused element and asserts none was cancelled and focus
/// stayed: a lone roving item leaves them to the page (todos 2432, 2465).
pub(crate) async fn assert_keys_left_to_page(page: &Page, keys: &[keyboard::Key]) {
    page.evaluate_bounded(
        "window.__prevented = []; window.__focused = document.activeElement; \
         if (!window.__listening) { window.__listening = true; \
         window.addEventListener('keydown', e => window.__prevented.push(e.defaultPrevented)); }",
    )
    .await
    .unwrap();
    for key in keys {
        keyboard::press(page, *key).await.unwrap();
    }
    let count = keys.len();
    wait::for_js_true(
        page,
        &format!("window.__prevented.length === {count}"),
        "the keys",
    )
    .await
    .unwrap();
    let prevented: Vec<bool> = js(page, "window.__prevented").await;
    assert_eq!(
        prevented,
        vec![false; count],
        "a key was cancelled: {keys:?}"
    );
    let stayed: bool = js(page, "document.activeElement === window.__focused").await;
    assert!(stayed, "focus moved");
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
    page.evaluate_bounded(format!(
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
    // Continue opens Review as Shipping closes: the opened trigger takes the focus (todo 4671).
    eventually_focused(d, &trigger(2), "Review, opened as Shipping closes").await
}

e2e::scenario!(
    the_arrows_move_focus_skip_the_disabled_and_never_toggle,
    "/accordion",
    arrows_move_focus,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView"),
    ios: skip("958: element identity on the WebView")
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
    desktop: skip("958: element identity on the WebView"),
    ios: skip("958: element identity on the WebView")
);

/// Todo 4400: a section inserted ahead of the open one keeps its panel, so the focus and
/// the text typed in it stay.
async fn growing_keeps_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#field-Beta").await?;
    d.type_text("typed").await?;
    d.press(keyboard::F2).await?;
    eventually(d, "Alpha's trigger to appear", async |d| {
        d.exists("#growing-trigger-0").await
    })
    .await?;
    eventually_focused(d, "#field-Beta", "Alpha inserted ahead of Beta").await?;
    let text = d.value("#field-Beta").await?;
    ensure!(text == "typed", "Beta's input lost its text: {text:?}");
    Ok(())
}

e2e::scenario!(
    a_section_inserted_ahead_keeps_the_focus_and_its_text,
    "/accordion-growing",
    growing_keeps_focus,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView"),
    ios: skip("958: element identity on the WebView")
);

/// The trigger whose text is `label`, so an insert ahead does not change which one it is.
fn trigger_named(label: &str) -> String {
    format!(
        "[...document.querySelectorAll('{TRIGGER}')].find(b => b.textContent.trim() === '{label}')"
    )
}

/// Beta's trigger has focus and is expanded.
fn beta_focused_and_open() -> String {
    let beta = trigger_named("Beta");
    format!(
        "(() => {{ const t = {beta}; return !!t && document.activeElement === t && t.getAttribute('aria-expanded') === 'true'; }})()"
    )
}

/// Focuses Beta's trigger once it is open, then fires the `input` on `id`, which moves no focus.
async fn focus_beta_then_fire(page: &Page, id: &str) {
    page.evaluate_bounded(format!("({}).focus()", trigger_named("Beta")))
        .await
        .unwrap();
    wait::for_js_true(page, &beta_focused_and_open(), "Beta open and focused")
        .await
        .unwrap();
    page.evaluate_bounded(format!(
        "document.getElementById('{id}').dispatchEvent(new Event('input', {{ bubbles: true }}))"
    ))
    .await
    .unwrap();
}

/// Todo 4419: a section inserted ahead of the open Beta keeps it open and its trigger focused.
#[test]
fn a_section_inserted_ahead_keeps_its_open_state_and_focus() {
    block_on(async {
        let fixture = Fixture::open("/accordion-insert", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_beta_then_fire(page, "insert").await;
        wait::for_js_true(
            page,
            &format!("!!{}", trigger_named("Zero")),
            "Zero's trigger",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &beta_focused_and_open(),
            "Beta still open and focused",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("an inserted section").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4419: a section removed ahead of the open Beta keeps it open and its trigger focused.
#[test]
fn a_section_removed_ahead_keeps_its_open_state_and_focus() {
    block_on(async {
        let fixture = Fixture::open("/accordion-insert", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_beta_then_fire(page, "remove").await;
        wait::for_js_true(
            page,
            &format!("!{}", trigger_named("Alpha")),
            "Alpha's trigger removed",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &beta_focused_and_open(),
            "Beta still open and focused",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a removed section").unwrap();
        fixture.close().await.unwrap();
    });
}

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
