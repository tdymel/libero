//! `Carousel`: a key pressed in a slide belongs to the slide (375); each check asserts the
//! control acted and the strip stayed. Desktop only: smooth scroll is unreliable in background.

use anyhow::{Result, bail, ensure};
use chromiumoxide::Page;
use e2e::archetypes::reset_tab_position;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, linger};
use e2e::passes::keyboard;
use e2e::passes::motion;
use e2e::passes::pointer;
use e2e::passes::target_size::MINIMUM;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, clock, wait};

/// The slide pads by exactly the ring's reach, so a fractional slide width can
/// round the clip a layout unit (1/64px) inside it.
const SUBPIXEL: f64 = 0.5;

/// Todos 618, 619: the slide and the track clipped the outset ring of a slide's content.
/// Checks every visible slide's button, the first flush with the track's left edge.
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
                clipped < SUBPIXEL,
                "slide button {stop}: the ring runs {clipped}px past the clip"
            );
        }
        fixture.close().await.unwrap();

        // Todo 619: nested, not a slide's direct child, and flush with its edge.
        let fixture = Fixture::open("/carousel", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        reach(page, BUTTON).await.unwrap();
        let clipped: f64 = page
            .evaluate(crate::image_list::ring_clipped(
                "[aria-roledescription=carousel]",
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            clipped < SUBPIXEL,
            "the nested slide button's ring runs {clipped}px past the clip"
        );
        fixture.close().await.unwrap();
    });
}

/// The scroll container, tab stop and key handler. Not `[role=group]` alone: every slide
/// is one too.
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
/// The second slide once current. At rest this matches nothing, as `Suite` requires of a
/// state's settle selector.
const SECOND_CURRENT: &str = "[aria-roledescription=slide]:nth-of-type(2)[data-current]";

/// The track, the text field, the slider, the range, the radio and the button,
/// with room to spare.
const TAB_BUDGET: usize = 10;

/// The index of the slide carrying `data-current`. Not `scrollLeft`: a smooth scroll never
/// lands in a `background: true` page, so it stays 0 (`codebase/e2e-harness`).
const INDEX: &str = "[...document.querySelectorAll('[aria-roledescription=slide]')]\
                     .findIndex(el => el.hasAttribute('data-current'))";

const FIRST_CURRENT: &str = "[aria-roledescription=slide]:nth-of-type(1)[data-current]";
const FIRST_SLIDE: &str = "[aria-roledescription=slide]:nth-of-type(1)";

async fn an_arrow_on_the_track_moves<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRACK).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "ArrowRight on the track to reach slide 2", async |d| {
        d.exists(SECOND_CURRENT).await
    })
    .await
}

/// An arrow in a field inside a slide stays the field's (`typing_target`,
/// `arrow_target`): the first slide is still the current one.
async fn an_arrow_in_a_field_stays<D: Driver>(d: &mut D, field: &str) -> Result<()> {
    d.focus(field).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    d.settle().await?;
    ensure!(
        d.exists(FIRST_CURRENT).await?,
        "{:?}: the carousel took the arrow pressed in {field}",
        d.platform()
    );
    Ok(())
}

/// Todo 1178: with the caret at the field's end, WebKitGTK scrolled the track on an
/// ArrowRight the caret could not follow.
async fn an_arrow_at_the_fields_end_stays<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TEXT).await?;
    d.press(keyboard::END).await?;
    let before = d.rect(FIRST_SLIDE).await?.x;
    d.press(keyboard::ARROW_RIGHT).await?;
    // A native scroll moves on a later frame and sends nothing to await before it.
    if !d.frame().await? {
        linger(d, 10).await;
    }
    let after = d.rect(FIRST_SLIDE).await?.x;
    ensure!(
        (after - before).abs() < 1.0 && d.is_focused(TEXT).await?,
        "{:?}: an ArrowRight at the field's end scrolled the track from {before} to {after}",
        d.platform()
    );
    Ok(())
}

async fn an_arrow_in_the_text_field_stays<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    an_arrow_in_a_field_stays(d, TEXT).await
}

async fn an_arrow_on_the_range_stays<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    an_arrow_in_a_field_stays(d, RANGE).await
}

e2e::scenario!(
    an_arrow_on_the_track_moves_to_the_next_slide,
    "/carousel",
    an_arrow_on_the_track_moves
);
e2e::scenario!(
    an_arrow_in_a_text_field_stays_in_the_field,
    "/carousel",
    an_arrow_in_the_text_field_stays
);
e2e::scenario!(
    an_arrow_at_a_text_fields_end_leaves_the_track,
    "/carousel",
    an_arrow_at_the_fields_end_stays
);
e2e::scenario!(
    an_arrow_on_a_range_stays_on_the_range,
    "/carousel",
    an_arrow_on_the_range_stays
);

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

/// The positive control: no arm covers a `Button`, so a guard that returned for
/// everything, passing the four checks above, fails here.
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

/// Tab to `selector` from the top of the document (`blur()` does not reset the Tab
/// position, see `archetypes::reset_tab_position`); asserts reachability on the way.
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
pub const KEY_TAKEN: &str = "key_taken";
pub const TYPING_TARGET: &str = "typing_target";
pub const ARROW_TARGET: &str = "arrow_target";

/// Plants the track's key handler without guard arm `arm`: a `window` bubble listener
/// (after dioxus's root listener, so `defaultPrevented` is set) that prevents and advances.
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

/// The generic battery (381), at rest and advanced: every state flips on the first advance.
/// No `targets(INDICATORS)`: the pass ignores the dots' 24px `::before` (see the hit test).
#[test]
fn it_meets_the_baseline() {
    Suite::new("carousel", "/carousel")
        .focusable(TRACK)
        .targets(CONTROLS)
        // `current` follows the scroll; a smooth scroll that never lands in a background
        // page puts the index back, and the snapshot flaked at 390px (375).
        .reduced_motion()
        .state("advanced", &[Step::Click(NEXT)], SECOND_CURRENT)
        .run();
}

/// WCAG 2.5.8 on the dots' 24px `::before` hit area (389): hit-tests 11.5px out, 13px as
/// the failing control, then clicks the third dot 11px above its drawn box.
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

/// A clone in the viewport is its slide's only live copy, so it cannot be `aria-hidden`
/// (axe `aria-hidden-focus`: its button took Tab while hidden).
#[test]
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

/// The autoplay toggle.
const PAUSE: &str = "[aria-roledescription=carousel] button[aria-pressed]";

/// WCAG 2.2.2, APG (548): focus entering stops autoplay for good; only Play resumes it.
/// The toggle is the first tab stop; the status is silent while it rotates.
#[test]
fn autoplay_stops_once_focus_enters_until_play() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        hold_rotation(page).await;
        instant_scroll(page).await;
        let live = "document.querySelector('[aria-roledescription=carousel] [role=status]')\
                    .getAttribute('aria-live')";
        let start = index(page).await.unwrap();
        rotates(page, start, "autoplay to advance").await;
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
        assert_pressed(page, "true", "focus entering").await;

        reach(page, "#before").await.unwrap();
        let held = index(page).await.unwrap();
        assert_stopped(page, held, "rotation resumed once focus left").await;

        keyboard::press(page, keyboard::TAB).await.unwrap();
        let first: bool = page
            .evaluate(format!("document.activeElement.matches({PAUSE:?})"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(first, "the toggle is not the carousel's first tab stop");
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        assert_pressed(page, "false", "Play").await;
        rotates(page, held, "autoplay to advance once Play is pressed").await;

        fixture.console.assert_clean("autoplay").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A mouse press on Pause while it rotates pauses it: the focus the press
/// brings in is not an entry that stops it first, which the click would undo.
#[test]
fn a_pointer_press_on_pause_pauses() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        assert_pressed(page, "false", "load").await;
        let (x, y): (f64, f64) = page
            .evaluate(format!(
                "(() => {{ const r = document.querySelector({PAUSE:?}).getBoundingClientRect(); \
                 return [r.x + r.width * 0.15, r.y + r.height / 2]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let at = pointer::Point { x, y };
        // On the rim: a glyph swapped under a held press takes its click with it. The
        // moves in place let the render after `focusin` run before the release.
        pointer::drag(page, at, at, 5).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement.matches({PAUSE:?})"),
            "the press to focus the toggle",
        )
        .await
        .unwrap();
        // The release's click and its render are in; a focus entry undone by it would read false.
        clock::settle(page).await.unwrap();
        assert_pressed(page, "true", "a pointer press on Pause").await;
        fixture
            .console
            .assert_clean("a pointer press on Pause")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Hover only pauses (todo 548): the pointer leaving resumes rotation, and
/// the toggle never presses.
#[test]
fn hover_only_pauses() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        instant_scroll(page).await;
        let live = "document.querySelector('[aria-roledescription=carousel] [role=status]')\
                    .getAttribute('aria-live')";
        pointer::hover(page, TRACK).await.unwrap();
        wait::for_js_true(page, &format!("{live} === 'polite'"), "hover to pause")
            .await
            .unwrap();
        pointer::hover(page, "#before").await.unwrap();
        let held = index(page).await.unwrap();
        wait::until("autoplay to resume once the pointer left", || async move {
            Ok(index(page).await? != held)
        })
        .await
        .unwrap();
        assert_pressed(page, "false", "hover").await;
        fixture.console.assert_clean("hover").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn assert_pressed(page: &Page, expected: &str, after: &str) {
    wait::for_js_true(
        page,
        &format!("document.querySelector({PAUSE:?}).getAttribute('aria-pressed') === {expected:?}"),
        &format!("the toggle's aria-pressed to be {expected} after {after}"),
    )
    .await
    .unwrap();
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
        // The pause state is read once, at mount: the hold reloads.
        hold_rotation(page).await;
        motion::assert_reduced_motion_matches(page).await.unwrap();
        let pause = "[aria-roledescription=carousel] button[aria-pressed]";
        let start = index(page).await.unwrap();
        assert_stopped(page, start, "rotated under reduced motion").await;
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
        rotates(page, start, "autoplay to advance once started").await;
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
        hold_rotation(page).await;
        instant_scroll(page).await;
        let pause = "[aria-roledescription=carousel] button[aria-pressed]";
        let start = index(page).await.unwrap();
        rotates(page, start, "autoplay to advance").await;

        motion::set_reduced_motion(page, true).await.unwrap();
        motion::assert_reduced_motion_matches(page).await.unwrap();
        // The next tick reads the preference and pauses instead of moving.
        let held = index(page).await.unwrap();
        clock::fire_all(page, AUTOPLAY_MS).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({pause:?}).getAttribute('aria-pressed') === 'true'"),
            "the pause control to press once reduced motion is on",
        )
        .await
        .unwrap();
        assert_stopped(page, held, "rotated after reduced motion turned on").await;

        // Play pressed under reduced motion is the reader's choice, and holds.
        page.evaluate(format!("document.querySelector({pause:?}).click()"))
            .await
            .unwrap();
        rotates(page, held, "autoplay to advance once started again").await;
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

/// A looping strip opens on slide 1, drawn where the component thinks it is. A smooth
/// opening scroll never landed in a background page: the drawn slides were `inert`.
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

/// A smooth scroll's late settle in a background page reads back a "rotation" autoplay
/// never made. The autoplay units test the timer, so the strip jumps.
async fn instant_scroll(page: &Page) {
    page.evaluate(format!(
        "(() => {{ const s = document.createElement('style'); \
         s.textContent = '{TRACK} {{ scroll-behavior: auto !important; }}'; \
         document.head.append(s); }})()"
    ))
    .await
    .unwrap();
}

/// The fixture's `autoplay_delay`.
const AUTOPLAY_MS: u32 = 400;

/// Holds the rotation timer from mount on, so the strip moves only when the test fires it.
async fn hold_rotation(page: &Page) {
    clock::hold_from_load(page, &[AUTOPLAY_MS]).await.unwrap();
    wait::for_visible(page, PAUSE).await.unwrap();
}

/// Fires the held rotation until the strip leaves `from`. Held, not the real clock's.
async fn rotates(page: &Page, from: i64, what: &str) {
    clock::until_armed(page, AUTOPLAY_MS, 1, what)
        .await
        .unwrap();
    wait::until(what, || async move {
        clock::fire_all(page, AUTOPLAY_MS).await?;
        Ok(index(page).await? != from)
    })
    .await
    .unwrap();
}

/// Stopped for good: no rotation timer is armed, and firing every held one moves nothing.
async fn assert_stopped(page: &Page, at: i64, why: &str) {
    clock::settle(page).await.unwrap();
    let armed = clock::armed(page, AUTOPLAY_MS).await.unwrap();
    clock::fire_all(page, AUTOPLAY_MS).await.unwrap();
    clock::settle(page).await.unwrap();
    assert_eq!(
        (armed, index(page).await.unwrap()),
        (0, at),
        "{why}: (rotation timers armed, index)"
    );
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

/// [`open_in`](crate::rtl_keys::open_in), with smooth scrolling off.
async fn open_in(route: &str, dir: &str) -> Fixture {
    let fixture = crate::rtl_keys::open_in(route, dir).await;
    instant_scroll(&fixture.page).await;
    fixture
}

/// Todo 708: a drag towards the start reveals the next slide, leftwards in
/// LTR and rightwards in RTL, and Next scrolls towards the end.
#[test]
fn a_drag_and_a_step_head_for_the_end_in_either_direction() {
    block_on(async {
        for (dir, sign) in [("ltr", -1.0), ("rtl", 1.0)] {
            let fixture = open_in("/carousel/text", dir).await;
            let page = &fixture.page;
            let from = pointer::centre_of(page, TRACK).await.unwrap();
            let to = pointer::Point {
                x: from.x + sign * 300.0,
                y: from.y,
            };
            pointer::drag(page, from, to, 12).await.unwrap();
            wait::until(
                &format!("{dir}: the drag to reach slide 2"),
                || async move { Ok(index(page).await? == 1) },
            )
            .await
            .unwrap();

            pointer::click(page, NEXT).await.unwrap();
            let moved = match dir {
                "rtl" => format!("document.querySelector({TRACK:?}).scrollLeft < -500"),
                _ => format!("document.querySelector({TRACK:?}).scrollLeft > 500"),
            };
            wait::for_js_true(page, &moved, &format!("{dir}: Next to scroll to slide 3"))
                .await
                .unwrap();
            wait::until(&format!("{dir}: Next to reach slide 3"), || async move {
                Ok(index(page).await? == 2)
            })
            .await
            .unwrap();
            fixture.console.assert_clean(dir).unwrap();
            fixture.close().await.unwrap();
        }
    });
}
