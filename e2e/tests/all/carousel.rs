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
use e2e::passes::motion;
use e2e::passes::pointer;
use e2e::passes::target_size::MINIMUM;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

/// Todo 618: the slide and the track clip flush at a slide's edge, which cut
/// away the outset ring of a slide's focusable content. Every visible slide's
/// button, the first one flush with the track's left edge.
#[test]
fn a_focused_slide_keeps_its_ring() {
    block_on(async {
        let fixture = Fixture::open("/carousel/loop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "[aria-roledescription=slide] button", 10)
            .await
            .unwrap();
        for stop in 0..3 {
            if stop > 0 {
                keyboard::press(page, keyboard::TAB).await.unwrap();
            }
            let clipped: f64 = page
                .evaluate(crate::image_list::ring_clipped(
                    "[aria-roledescription=carousel]",
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                clipped <= 0.0,
                "slide button {stop}: the ring runs {clipped}px past the clip"
            );
        }
        fixture.close().await.unwrap();
    });
}

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
/// The two chevrons. They are the only buttons in the component that point at
/// the track with `aria-controls`.
const CONTROLS: &str = "[aria-roledescription=carousel] button[aria-controls]";
/// The dots. Their ids are `<track id>-indicator-<n>`, and the track id is
/// generated, so the stable half of it is what this matches.
const INDICATORS: &str = "[aria-roledescription=carousel] button[id*='-indicator-']";
const NEXT: &str = "[aria-roledescription=carousel] button[aria-label='Next slide']";
/// The second slide once it is the current one. Nothing carries `data-current`
/// but the resting slide, so at rest this matches nothing - which is what
/// `Suite` requires of a state's settle selector.
const SECOND_CURRENT: &str = "[aria-roledescription=slide]:nth-of-type(2)[data-current]";

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
pub async fn a_raw_range_in_a_slide_steps(fixture: &Fixture) -> Result<()> {
    let page = &fixture.page;
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

/// The generic battery, added by todo 381.
///
/// `81d44d8e` gave `/carousel` a fixture and a behaviour test and no
/// `Suite::new`, so axe, the AX snapshot, the focus-ring pass and the
/// target-size pass had never seen a real `Carousel` - on a page that draws
/// two chevrons and, since this todo, a strip of dots.
///
/// **The advanced state is the point of the second declaration.** A carousel's
/// resting render is the easy half: at index 0 the previous control is
/// `aria-disabled`, one slide is live and two are `inert`, and the first dot is
/// the strip's only tab stop. Every one of those flips on the first advance,
/// and an accessibility tree that is right at rest and wrong one slide along is
/// the normal shape of these bugs.
///
/// `focusable` is the track alone, because that is what `Suite` can reach: it
/// measures the resting state, and the controls and dots are there too but the
/// track is the component's own tab stop and the ring that matters. The dots'
/// roving focus is exercised by the component's own keyboard tests in `libero`.
///
/// Contrast coverage holds because the guard drops `inert` text, as axe does
/// (todo 386). Only the live slide is claimed; in the advanced state the drawn
/// slide may still be `inert` (todo 375), and its text is not evaluated.
///
/// ## A pass not declared, and why
///
/// * **`targets(INDICATORS)`**, the dots. They are drawn **40x5, 24x5 and
///   24x5** at 1280x800, and since todo 389 each takes the pointer over a
///   `::before` at least 24px on both axes, as `Slider`'s thumb does. The pass
///   measures bounding boxes, which never include a pseudo-element, so it would
///   report 5px for dots that meet 2.5.8.
///   [`the_dots_take_the_pointer_over_24px`] hit-tests the area instead.
#[test]
fn it_meets_the_baseline() {
    Suite::new("carousel", "/carousel")
        .focusable(TRACK)
        .targets(CONTROLS)
        // The strip is scroll-driven, and `current` follows the scroll frame
        // by frame. Without this the advanced state snapshotted the previous
        // control both enabled and `[disabled]` on two runs of the same code
        // at 390px: a smooth scroll that never lands in a `background: true`
        // page ends in a scroll event at the old offset, which puts the index
        // back (todo 375).
        .reduced_motion()
        .state("advanced", &[Step::Click(NEXT)], SECOND_CURRENT)
        .run();
}

/// WCAG 2.5.8 on the dots' hit area, not their box: a dot is drawn 5px thick
/// and a `::before` at least 24px on both axes takes the pointer (todo 389).
///
/// Hit-tests each dot's corners 11.5px out from its centre on both axes, and
/// 13px out as the control that the check can fail. Then clicks the third dot
/// 11px above its centre, outside its drawn box, so only the hit area can take
/// that click.
#[test]
fn the_dots_take_the_pointer_over_24px() {
    block_on(async {
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        let inside = dot_corners(page, MINIMUM / 2.0 - 0.5).await.unwrap();
        assert!(
            !inside.is_empty() && inside.iter().all(|dot| dot.iter().all(|&hit| hit)),
            "a corner of a dot's 24px hit area missed the dot: {inside:?}"
        );
        let outside = dot_corners(page, MINIMUM / 2.0 + 1.0).await.unwrap();
        assert!(
            outside.iter().all(|dot| dot.iter().all(|&hit| !hit)),
            "a dot took the pointer outside its 24px hit area: {outside:?}"
        );

        let third = dot_centre(page, 2).await.unwrap();
        let at = pointer::Point {
            x: third.x,
            y: third.y - 11.0,
        };
        pointer::drag(page, at, at, 1).await.unwrap();
        wait::until(
            "a click on the third dot's hit area to go there",
            || async move { Ok(index(page).await? == 2) },
        )
        .await
        .unwrap_or_else(|e| panic!("a click 11px above the third dot did not select it: {e}"));

        fixture.console.assert_clean("clicks on the dots").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Per dot, whether each corner of a square `offset` out from its centre
/// hit-tests to that dot.
async fn dot_corners(page: &Page, offset: f64) -> Result<Vec<Vec<bool>>> {
    Ok(page
        .evaluate(format!(
            "[...document.querySelectorAll({INDICATORS:?})].map(dot => {{ \
             const r = dot.getBoundingClientRect(); \
             const x = r.x + r.width / 2, y = r.y + r.height / 2; \
             return [[-1,-1],[1,-1],[-1,1],[1,1]].map(([dx, dy]) => \
             document.elementFromPoint(x + dx * {offset}, y + dy * {offset}) === dot); }})"
        ))
        .await?
        .into_value()?)
}

/// The centre of the `n`th dot, in viewport coordinates.
async fn dot_centre(page: &Page, n: usize) -> Result<pointer::Point> {
    let (x, y): (f64, f64) = page
        .evaluate(format!(
            "(() => {{ const r = document.querySelectorAll({INDICATORS:?})[{n}]\
             .getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; }})()"
        ))
        .await?
        .into_value()?;
    Ok(pointer::Point { x, y })
}

/// A clone showing in the viewport is the only live copy of its slide, so it
/// cannot be `aria-hidden`: its button took Tab while hidden from a screen
/// reader (axe `aria-hidden-focus`), and its slide was missing from the tree.
#[test]
#[ignore = "todo 544"]
fn a_visible_clone_is_not_hidden_from_assistive_tech() {
    block_on(async {
        let fixture = Fixture::open("/carousel/loop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelectorAll('[aria-roledescription=carousel] [inert]').length > 0",
            "the offscreen slides to go inert",
        )
        .await
        .unwrap();
        let hidden_focusable: Vec<String> = page
            .evaluate(
                "[...document.querySelectorAll('[aria-roledescription=carousel] [aria-hidden=true]')]\
                 .flatMap(h => [...h.querySelectorAll('button, a[href], input, [tabindex]')])\
                 .filter(el => !el.closest('[inert]'))\
                 .map(el => el.textContent)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            hidden_focusable.is_empty(),
            "focusable content under aria-hidden: {hidden_focusable:?}"
        );
        let names: Vec<String> = page
            .evaluate(
                "[...document.querySelectorAll('[aria-roledescription=slide]')]\
                 .filter(el => !el.closest('[inert]'))\
                 .map(el => el.getAttribute('aria-label'))",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            names,
            ["5 of 5", "1 of 5", "2 of 5"],
            "the live slide groups"
        );
        fixture
            .console
            .assert_clean("a looping carousel at rest")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 2.2.2: autoplay advances, stops while focus is inside, and the pause
/// control stops it for good. The status is silent while it rotates.
#[test]
fn autoplay_pauses_on_focus_and_on_its_control() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        let live = "document.querySelector('[aria-roledescription=carousel] [role=status]')\
                    .getAttribute('aria-live')";
        let start = index(page).await.unwrap();
        wait::until("autoplay to advance", || async move {
            Ok(index(page).await? != start)
        })
        .await
        .unwrap();
        let polite: String = page.evaluate(live).await.unwrap().into_value().unwrap();
        assert_eq!(polite, "off", "the status while rotating");

        reach(page, TRACK).await.unwrap();
        // The status turns polite in the render that stops the timer.
        wait::for_js_true(
            page,
            &format!("{live} === 'polite'"),
            "the status once rotation stops",
        )
        .await
        .unwrap();
        let held = index(page).await.unwrap();
        sleep(1200).await;
        assert_eq!(
            index(page).await.unwrap(),
            held,
            "rotated with focus inside"
        );

        let pause = "[aria-roledescription=carousel] button[aria-pressed]";
        keyboard::tab_to(page, pause, TAB_BUDGET).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({pause:?}).getAttribute('aria-pressed') === 'true'"),
            "the pause control to press",
        )
        .await
        .unwrap();
        reach(page, "#before").await.unwrap();
        let held = index(page).await.unwrap();
        sleep(1200).await;
        assert_eq!(
            index(page).await.unwrap(),
            held,
            "rotated after the pause control"
        );

        fixture.console.assert_clean("autoplay").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 551: under reduced motion autoplay starts paused, and the control
/// starts it.
#[test]
fn autoplay_waits_for_its_control_under_reduced_motion() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, true).await.unwrap();
        // The pause state is read once, at mount.
        page.reload().await.unwrap();
        motion::assert_reduced_motion_matches(page).await.unwrap();
        let pause = "[aria-roledescription=carousel] button[aria-pressed]";
        wait::for_visible(page, pause).await.unwrap();
        let start = index(page).await.unwrap();
        sleep(1200).await;
        assert_eq!(
            index(page).await.unwrap(),
            start,
            "rotated under reduced motion"
        );
        let pressed: String = page
            .evaluate(format!(
                "document.querySelector({pause:?}).getAttribute('aria-pressed')"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(pressed, "true", "the pause control under reduced motion");

        page.evaluate(format!("document.querySelector({pause:?}).click()"))
            .await
            .unwrap();
        wait::until("autoplay to advance once started", || async move {
            Ok(index(page).await? != start)
        })
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("autoplay under reduced motion")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 567: reduced motion switched on while it rotates pauses it, not only
/// at mount.
#[test]
fn autoplay_pauses_when_reduced_motion_turns_on() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        let pause = "[aria-roledescription=carousel] button[aria-pressed]";
        let start = index(page).await.unwrap();
        wait::until("autoplay to advance", || async move {
            Ok(index(page).await? != start)
        })
        .await
        .unwrap();

        motion::set_reduced_motion(page, true).await.unwrap();
        motion::assert_reduced_motion_matches(page).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({pause:?}).getAttribute('aria-pressed') === 'true'"),
            "the pause control to press once reduced motion is on",
        )
        .await
        .unwrap();
        let held = index(page).await.unwrap();
        sleep(1200).await;
        assert_eq!(
            index(page).await.unwrap(),
            held,
            "rotated after reduced motion turned on"
        );

        // Play pressed under reduced motion is the reader's choice, and holds.
        page.evaluate(format!("document.querySelector({pause:?}).click()"))
            .await
            .unwrap();
        wait::until("autoplay to advance once started again", || async move {
            Ok(index(page).await? != held)
        })
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("autoplay after reduced motion turned on")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// On a short strip the pause control sat on top of Next: Next's focus ring
/// was hidden (2.4.11) and a click on Next paused instead.
#[test]
fn the_pause_control_leaves_next_uncovered() {
    block_on(async {
        for viewport in [Viewport::Desktop, Viewport::Mobile] {
            let fixture = Fixture::open("/carousel/autoplay", viewport).await.unwrap();
            let hit: bool = fixture
                .page
                .evaluate(format!(
                    "(() => {{ const next = document.querySelector({NEXT:?}); \
                     const r = next.getBoundingClientRect(); \
                     const at = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2); \
                     return next.contains(at); }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(hit, "something covers Next's centre at {viewport:?}");
            fixture.close().await.unwrap();
        }
    });
}

/// A looping strip opens on slide 1, drawn where the component thinks it is.
/// Smooth, the opening scroll swept past the clones, and in a background page
/// never landed: the drawn slides were `inert` and the live ones offscreen.
#[test]
fn a_looping_strip_opens_on_its_first_slide() {
    block_on(async {
        let fixture = Fixture::open("/carousel/loop", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_on_new_document(format!(
            "window.__carouselSamples = []; \
             requestAnimationFrame(function sample() {{ \
             const t = document.querySelector({TRACK:?}); \
             if (t) window.__carouselSamples.push(t.scrollLeft); \
             if (window.__carouselSamples.length < 120) requestAnimationFrame(sample); }});"
        ))
        .await
        .unwrap();
        page.reload().await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const t = document.querySelector({TRACK:?})?.getBoundingClientRect(); \
                 const s = document.querySelector('[aria-label=\"1 of 5\"]')?.getBoundingClientRect(); \
                 if (!t || !s) return false; const mid = s.x + s.width / 2; return mid > t.left && mid < t.right; }})()"
            ),
            "slide 1 to be drawn inside the track",
        )
        .await
        .unwrap();
        // Sampled from load: an instant placement has no frame between the two.
        let between: Vec<f64> = page
            .evaluate(format!(
                "window.__carouselSamples.filter(v => v > 0 && \
                 v < document.querySelector({TRACK:?}).scrollLeft - 1)"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            between.is_empty(),
            "the opening scroll swept through {between:?}"
        );
        fixture
            .console
            .assert_clean("a looping carousel's mount")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A smooth scroll crawls in a background page, and its late settle reads
/// the index back from where it stopped: a "rotation" the autoplay never
/// made. The autoplay units are about the timer, so the strip jumps.
async fn instant_scroll(page: &Page) {
    page.evaluate(format!(
        "(() => {{ const s = document.createElement('style'); \
         s.textContent = '{TRACK} {{ scroll-behavior: auto !important; }}'; \
         document.head.append(s); }})()"
    ))
    .await
    .unwrap();
}

async fn sleep(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}

/// Forced colours paint every dot `Canvas`: the strip vanished, and with it
/// which slide is current (todo 524).
#[test]
fn the_dots_show_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        // The dots fade their fill; read it once the fade has landed.
        wait::for_js_true(
            page,
            "document.getAnimations().length === 0",
            "the dots' fades to end",
        )
        .await
        .unwrap();
        // The page itself is `Canvas`; a fill the same colour is no fill.
        let [current, other, canvas]: [String; 3] = page
            .evaluate(format!(
                "(() => {{ const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 const dots = [...document.querySelectorAll({INDICATORS:?})]; \
                 const current = dots.find((d) => d.getAttribute('aria-current') === 'true'); \
                 const other = dots.find((d) => d !== current); \
                 return [getComputedStyle(current).backgroundColor, \
                 getComputedStyle(other).backgroundColor, canvas]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(other, canvas, "a dot's fill is the page's own");
        assert_ne!(current, canvas, "the current dot's fill is the page's own");
        assert_ne!(current, other, "the current dot looks like the others");
        fixture.close().await.unwrap();
    });
}
