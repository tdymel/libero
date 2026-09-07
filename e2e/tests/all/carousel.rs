//! `Carousel`: a key pressed inside a slide belongs to the slide.
//!
//! The track handles the arrows, Home and End, and `prevent_default()`s them.
//! Its slides hold a caller's own content, which this component cannot ask to
//! stop propagating, so every press made inside a slide arrives at the track's
//! handler as well. Three guard arms separate the two cases
//! (`libero/src/components/navigation/carousel.rs`):
//!
//! | arm | what it covers | the control in the fixture |
//! |---|---|---|
//! | `key_taken` | a libero control that took the key by preventing its default | `Slider` |
//! | `typing_target` | a caret move the browser performs, which nothing marks | `TextField` |
//! | `arrow_target` | raw HTML the browser steps with the arrows | `<input type="range">`, a radio pair |
//!
//! Nothing in the tree covered any of it before todo 375: the docs page's
//! slides are plain text, so the guard could be deleted whole with every test
//! still green.
//!
//! **Each check asserts two things at once, and both halves matter.** That a
//! control did what the key means to it is the visible symptom of the guard
//! holding; that the strip did not move is the other half, since a carousel
//! that advanced *as well* is the double action the guard exists to stop.
//! [`arrows_on_a_plain_button_move_the_strip`] is what keeps the second half
//! from being a sentence about a carousel that never moves: a `Button` is
//! covered by no arm, so arrows pressed on one have to advance the strip.
//!
//! Desktop only. The guard is about where a press landed, not about layout,
//! and `Carousel` moves the strip with a smooth scroll, which is unreliable in
//! a background page at 390px (`codebase/e2e-harness`).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use e2e::archetypes::reset_tab_position;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

/// The scroll container, which is also the tab stop and where the key handler
/// sits. Not `[role=group]` alone: every slide is one too. The track is the
/// group that describes itself with the live status region.
pub const TRACK: &str = "[aria-roledescription=carousel] [role=group][aria-describedby]";
const TEXT: &str = "[aria-roledescription=slide] input[type=text]";
const SLIDER: &str = "[aria-roledescription=slide] [role=slider]";
const RANGE: &str = "#raw-range";
const RADIO_A: &str = "#raw-radio-a";
const RADIO_B: &str = "#raw-radio-b";
const BUTTON: &str = "#slide-button";

/// The track, the text field, the slider, the range, the radio and the button,
/// with room to spare.
const TAB_BUDGET: usize = 10;

/// Where the carousel says it is: the position of the slide carrying
/// `data-current`, which is the index `onindexchange` reports and the status
/// region reads out.
///
/// **Not the track's `scrollLeft`.** `go_to` sets the index and asks the track
/// to scroll to it, and that scroll is smooth - which in a page created
/// `background: true` does not land at all. Measured on this fixture at
/// 1280x900: after an arrow the index went 0 -> 1 -> 2 and the slide the
/// component treats as offscreen went `inert`, while `scrollLeft` stayed 0
/// against a 1284px strip in a 420px track. `codebase/e2e-harness` records the
/// same trap at 390px; this is it at desktop width. So the offset is not a
/// measure of anything here, and the index is: it is what a caller is told and
/// what a screen reader is read.
const INDEX: &str = "[...document.querySelectorAll('[aria-roledescription=slide]')]\
                     .findIndex(el => el.hasAttribute('data-current'))";

/// Every control in the slide keeps the keys that are its own, and none of
/// them moves the strip - and a plain button, which no arm covers, does.
#[test]
fn slide_content_keeps_its_own_keys() {
    block_on(async {
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();

        the_caret_moves_in_a_slides_text_field(&fixture)
            .await
            .unwrap();
        the_slider_in_a_slide_takes_the_arrows(&fixture)
            .await
            .unwrap();
        a_raw_range_in_a_slide_steps(&fixture).await.unwrap();
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
pub async fn the_caret_moves_in_a_slides_text_field(fixture: &Fixture) -> Result<()> {
    let page = &fixture.page;
    reach(page, TEXT).await?;
    // Tab into a text input selects its contents, so the caret has no
    // position to move from until Home gives it one. Home is a carousel key
    // too, so this press is already inside what is being tested.
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
pub async fn the_slider_in_a_slide_takes_the_arrows(fixture: &Fixture) -> Result<()> {
    let page = &fixture.page;
    reach(page, SLIDER).await?;

    let value = format!("document.querySelector({SLIDER:?}).getAttribute('aria-valuenow')");
    let before_value: Option<String> = page.evaluate(value.clone()).await?.into_value()?;
    let Some(before_value) = before_value else {
        bail!("the slider in the slide reports no aria-valuenow");
    };

    let before = index(page).await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait::for_js_change(
        page,
        &value,
        &before_value,
        "the slider in the slide to step",
    )
    .await
    .map_err(|e| anyhow::anyhow!("the slider in the slide did not step: {e}"))?;

    assert_strip_held(page, before, "an arrow on a slider in a slide").await
}

/// `arrow_target`, first shape: raw HTML the browser steps although nothing
/// about it is text entry, so no other arm can see it.
pub async fn a_raw_range_in_a_slide_steps(fixture: &Fixture) -> Result<()> {
    let page = &fixture.page;
    reach(page, RANGE).await?;

    let value = format!("document.querySelector({RANGE:?}).value");
    let before_value: Option<String> = page.evaluate(value.clone()).await?.into_value()?;
    let before_value = before_value.unwrap_or_default();

    let before = index(page).await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait::for_js_change(page, &value, &before_value, "the raw range to step")
        .await
        .map_err(|e| anyhow::anyhow!("the raw range in the slide did not step: {e}"))?;

    assert_strip_held(page, before, "an arrow on a raw range in a slide").await
}

/// `arrow_target`, second shape: a native radio group, where the arrow moves
/// the selection rather than a value.
pub async fn a_raw_radio_pair_in_a_slide_moves(fixture: &Fixture) -> Result<()> {
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

/// The positive control, and the reason the four checks above say anything.
///
/// A `Button` is covered by no arm: it takes no arrows, the browser steps
/// nothing on it, and nothing prevents their default. So the press belongs to
/// the carousel, and a guard that returned for everything - the obvious way to
/// make the four checks above pass - fails here.
pub async fn arrows_on_a_plain_button_move_the_strip(fixture: &Fixture) -> Result<()> {
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

/// Tab to `selector` from the top of the document.
///
/// Every check leaves focus inside the widget, and `blur()` does not put the
/// next Tab back at the start (`archetypes::reset_tab_position`). Tabbing
/// rather than calling `focus()` also gets each control's reachability
/// asserted on the way past.
async fn reach(page: &Page, selector: &str) -> Result<()> {
    reset_tab_position(page).await?;
    keyboard::tab_to(page, selector, TAB_BUDGET).await?;
    Ok(())
}

async fn index(page: &Page) -> Result<i64> {
    let index: i64 = page.evaluate(INDEX).await?.into_value()?;
    if index < 0 {
        bail!("no slide carries data-current, so the carousel's index cannot be read");
    }
    Ok(index)
}

/// The strip is where it was.
///
/// Read **after** something the press was supposed to do has been waited for,
/// never straight after the key: a carousel that has not moved yet and one
/// that never will look the same on the next line.
async fn assert_strip_held(page: &Page, before: i64, what: &str) -> Result<()> {
    let after = index(page).await?;
    if after != before {
        bail!("the carousel advanced on {what}: its index went from {before} to {after}");
    }
    Ok(())
}

/// The three arms, as `plant_arm_out` names them.
pub const KEY_TAKEN: &str = "key_taken";
pub const TYPING_TARGET: &str = "typing_target";
pub const ARROW_TARGET: &str = "arrow_target";

/// The track's key handler with one guard arm taken out, as a plant.
///
/// The arm cannot be removed from the library for a test, so it is removed
/// from a copy: a listener that re-implements the guard without `arm`, and
/// applies the unguarded handler's two effects - `prevent_default()` and an
/// advance - to every press it lets through. What a slide's control then sees
/// is exactly what it would see with that arm deleted from `carousel.rs`.
///
/// The re-dispatched press is the real handler's, so this plant needs the
/// component to be working to show anything: it takes an arm off the guard and
/// leaves everything else alone.
///
/// It listens on `window` in the bubble phase, which is **after** dioxus's own
/// delegated listener at the document root. That ordering is what makes the
/// `key_taken` arm mean anything here: a listener on the track itself runs
/// before every dioxus handler, so `defaultPrevented` would still be false for
/// a key the `Slider` had already taken, and the arm would read as removed in
/// all three plants.
pub fn plant_arm_out(arm: &str) -> String {
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
                // What the unguarded handler does: eat the browser's default
                // action, and navigate. The navigation is the component's own
                // - the same press re-dispatched at the track, where no arm
                // applies - rather than a scroll written by hand, so what is
                // measured afterwards is `Carousel` moving.
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
