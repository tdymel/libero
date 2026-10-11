//! `Carousel`'s slide content: a key pressed in a slide belongs to the slide (375), and each
//! guard arm of the track planted out in turn (311).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, selftest, wait};

use super::carousel::{
    BUTTON, INDEX, INDICATORS, RADIO_A, RADIO_B, RANGE, SLIDER, TEXT, TRACK, index, reach,
};

/// Every control in the slide keeps the keys that are its own, and none of
/// them moves the strip - and a plain button, which no arm covers, does.
#[test]
fn slide_content_keeps_its_own_keys() {
    block_on(async {
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();

        the_caret_moves_in_a_slides_text_field(&fixture.page)
            .await
            .unwrap();
        the_slider_in_a_slide_takes_the_arrows(&fixture.page)
            .await
            .unwrap();
        a_raw_range_in_a_slide_steps(&fixture.page).await.unwrap();
        a_raw_radio_pair_in_a_slide_moves(&fixture).await.unwrap();

        // Last: it is the one press that does move the strip, and everything
        // above wants the first slide still resting and out of `inert`.
        arrows_on_a_plain_button_move_the_strip(&fixture)
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("keys pressed inside a carousel's slides")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `typing_target`: the browser moves the caret, and nothing marks the press.
async fn the_caret_moves_in_a_slides_text_field(page: &Page) -> Result<()> {
    reach(page, TEXT).await?;
    // Tab selects the input's text, so Home gives the caret a position; Home is a
    // carousel key too, so this press is already under test.
    keyboard::press(page, keyboard::HOME).await?;

    let before = index(page).await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait::for_js_true(
        page,
        &format!("document.querySelector({TEXT:?}).selectionStart === 1"),
        "the caret in the slide's text field to move one character right",
    )
    .await
    .map_err(|e| anyhow::anyhow!("the caret in the slide's text field did not move: {e}"))?;

    assert_strip_held(page, before, "a caret move in a slide's text field").await
}

/// `key_taken`: `Slider` acts on the arrow and prevents its default, which is
/// how every libero control marks a key as taken.
async fn the_slider_in_a_slide_takes_the_arrows(page: &Page) -> Result<()> {
    reach(page, SLIDER).await?;

    let value = format!("document.querySelector({SLIDER:?}).getAttribute('aria-valuenow')");
    let before = index(page).await?;
    wait::for_js_change(page, &value, "the slider in the slide to step", || {
        keyboard::press(page, keyboard::ARROW_RIGHT)
    })
    .await
    .map_err(|e| anyhow::anyhow!("the slider in the slide did not step: {e}"))?;

    assert_strip_held(page, before, "an arrow on a slider in a slide").await
}

/// `arrow_target`, first shape: raw HTML the browser steps although nothing
/// about it is text entry, so no other arm can see it.
async fn a_raw_range_in_a_slide_steps(page: &Page) -> Result<()> {
    reach(page, RANGE).await?;

    let value = format!("document.querySelector({RANGE:?}).value");
    let before = index(page).await?;
    wait::for_js_change(page, &value, "the raw range to step", || {
        keyboard::press(page, keyboard::ARROW_RIGHT)
    })
    .await
    .map_err(|e| anyhow::anyhow!("the raw range in the slide did not step: {e}"))?;

    assert_strip_held(page, before, "an arrow on a raw range in a slide").await
}

/// `arrow_target`, second shape: a native radio group, where the arrow moves
/// the selection rather than a value.
async fn a_raw_radio_pair_in_a_slide_moves(fixture: &Fixture) -> Result<()> {
    let page = &fixture.page;
    reach(page, RADIO_A).await?;

    let before = index(page).await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait::for_js_true(
        page,
        &format!("document.querySelector({RADIO_B:?}).checked"),
        "the raw radio pair to move its selection",
    )
    .await
    .map_err(|e| anyhow::anyhow!("the raw radio pair in the slide did not move: {e}"))?;

    assert_strip_held(page, before, "an arrow on a raw radio in a slide").await
}

/// Ctrl/Alt/Meta chords are the browser's, on the track and on the dots.
#[test]
fn modifier_chords_go_to_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let keys = [
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_LEFT,
            keyboard::HOME,
            keyboard::END,
        ];
        let probe = format!("[{INDEX}, document.activeElement.id]");

        reach(page, BUTTON).await.unwrap();
        keyboard::assert_chords_ignored(page, &keys, &probe)
            .await
            .unwrap();
        reach(page, INDICATORS).await.unwrap();
        keyboard::assert_chords_ignored(page, &keys, &probe)
            .await
            .unwrap();

        fixture.console.assert_clean("carousel chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The positive control: no arm covers a `Button`, so a guard that returned for
/// everything, passing the four checks above, fails here.
async fn arrows_on_a_plain_button_move_the_strip(fixture: &Fixture) -> Result<()> {
    let page = &fixture.page;
    reach(page, BUTTON).await?;

    let before = index(page).await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait::until(
        "the carousel to advance from an arrow on a plain button in a slide",
        || async move { Ok(index(page).await? > before) },
    )
    .await
    .map_err(|e| anyhow::anyhow!("the carousel did not advance on an arrow it owns: {e}"))?;
    Ok(())
}

/// The strip is where it was. Call it only after waiting for the press's own effect:
/// straight after the key, "not yet moved" looks like "never moves".
async fn assert_strip_held(page: &Page, before: i64, what: &str) -> Result<()> {
    let after = index(page).await?;
    if after != before {
        bail!("the carousel advanced on {what}: its index went from {before} to {after}");
    }
    Ok(())
}

/// The three arms, as `plant_arm_out` names them.
const KEY_TAKEN: &str = "key_taken";
const TYPING_TARGET: &str = "typing_target";
const ARROW_TARGET: &str = "arrow_target";

/// Plants the track's key handler without guard arm `arm`: a `window` bubble listener
/// (after dioxus's root listener, so `defaultPrevented` is set) that prevents and advances.
fn plant_arm_out(arm: &str) -> String {
    format!(
        r#"(() => {{
            const arm = {arm};
            const track = {track};
            // `takes_typing`'s list, in `libero/src/platform/keyboard.rs`.
            const clicked = ['checkbox', 'radio', 'button', 'submit', 'reset', 'range',
                             'color', 'file', 'image'];
            addEventListener('keydown', e => {{
                const el = e.target;
                if (!el.closest || !el.closest(track)) return;
                const tag = el.tagName;
                const type = (el.getAttribute('type') || '').toLowerCase();
                const typing = tag === 'TEXTAREA' || tag === 'SELECT' || el.isContentEditable ||
                    (tag === 'INPUT' && !clicked.includes(type));
                const stepping = tag === 'INPUT' && (type === 'range' || type === 'radio');
                if (arm !== 'key_taken' && e.defaultPrevented) return;
                if (arm !== 'typing_target' && typing) return;
                if (arm !== 'arrow_target' && stepping) return;
                if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(e.key)) return;
                // The unguarded handler: prevent the default, then re-dispatch the press
                // at the track, so what moves afterwards is `Carousel` itself.
                e.preventDefault();
                el.closest(track).dispatchEvent(new KeyboardEvent('keydown', {{
                    key: e.key, code: e.code, bubbles: true, cancelable: true,
                }}));
            }});
        }})()"#,
        arm = serde_json::to_string(arm).unwrap(),
        track = serde_json::to_string(TRACK).unwrap(),
    )
}

/// `Carousel`'s track guard with one arm out per plant (`crate::plant_arm_out`),
/// each aimed at the control only that arm saves.
#[test]
fn a_carousel_that_ignores_key_taken_advances_under_its_own_slider() {
    block_on(async {
        selftest::must_fail(
            "/carousel",
            Some(&plant_arm_out(KEY_TAKEN)),
            "the carousel advanced on an arrow on a slider in a slide",
            the_slider_in_a_slide_takes_the_arrows,
        )
        .await;
    });
}

/// Without `typing_target` the track eats the caret move, so the check fails on the caret,
/// the symptom a user meets first.
#[test]
fn a_carousel_that_ignores_typing_target_eats_a_caret_move() {
    block_on(async {
        selftest::must_fail(
            "/carousel",
            Some(&plant_arm_out(TYPING_TARGET)),
            "the caret in the slide's text field did not move",
            the_caret_moves_in_a_slides_text_field,
        )
        .await;
    });
}

/// Without `arrow_target` a caller's `<input type="range">` stops stepping; no other arm
/// covers it.
#[test]
fn a_carousel_that_ignores_arrow_target_stops_a_raw_range() {
    block_on(async {
        selftest::must_fail(
            "/carousel",
            Some(&plant_arm_out(ARROW_TARGET)),
            "the raw range in the slide did not step",
            a_raw_range_in_a_slide_steps,
        )
        .await;
    });
}
