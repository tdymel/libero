//! `Carousel` autoplay: the rotation timer, the Pause toggle and what stops it (focus, a
//! pointer, hover, reduced motion), plus the toggle's layout over Next and the dots.

use anyhow::{Result, ensure};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;
use e2e::passes::motion;
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, clock, wait};

use super::carousel::{NEXT, TRACK, index, instant_scroll, reach};

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
        let polite: String = page
            .evaluate_bounded(live)
            .await
            .unwrap()
            .into_value()
            .unwrap();
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
            .evaluate_bounded(format!("document.activeElement.matches({PAUSE:?})"))
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

/// Todo 1928, APG: without `loop`, autoplay rests on the last slide and presses Pause;
/// Play pressed there starts over from the first.
#[test]
fn autoplay_without_loop_rests_on_the_last_slide() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        hold_rotation(page).await;
        instant_scroll(page).await;
        let mut at = index(page).await.unwrap();
        while at < 3 {
            rotates(page, at, "autoplay to advance to the last slide").await;
            at = index(page).await.unwrap();
        }
        clock::fire_all(page, AUTOPLAY_MS).await.unwrap();
        assert_pressed(page, "true", "the last slide's tick").await;
        assert_stopped(page, 3, "rotated past the last slide").await;

        page.evaluate_bounded(format!("document.querySelector({PAUSE:?}).click()"))
            .await
            .unwrap();
        rotates(page, 3, "Play on the last slide to start over").await;
        assert_eq!(index(page).await.unwrap(), 0, "Play on the last slide");
        fixture
            .console
            .assert_clean("autoplay resting on the last slide")
            .unwrap();
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
            .evaluate_bounded(format!(
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
        hold_rotation(page).await;
        instant_scroll(page).await;
        let live = "document.querySelector('[aria-roledescription=carousel] [role=status]')\
                    .getAttribute('aria-live')";
        pointer::hover(page, TRACK).await.unwrap();
        wait::for_js_true(page, &format!("{live} === 'polite'"), "hover to pause")
            .await
            .unwrap();
        let held = index(page).await.unwrap();
        assert_stopped(page, held, "rotated while hovered").await;
        pointer::hover(page, "#before").await.unwrap();
        rotates(page, held, "autoplay to resume once the pointer left").await;
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
            .evaluate_bounded(format!(
                "document.querySelector({pause:?}).getAttribute('aria-pressed')"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(pressed, "true", "the pause control under reduced motion");

        page.evaluate_bounded(format!("document.querySelector({pause:?}).click()"))
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
        page.evaluate_bounded(format!("document.querySelector({pause:?}).click()"))
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
                .evaluate_bounded(format!(
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

/// Todo 3035: the pause control sat at the root's bottom corner, over the dot row's end.
#[test]
fn the_pause_control_stays_off_the_dots() {
    block_on(async {
        for viewport in [Viewport::Desktop, Viewport::Mobile] {
            let fixture = Fixture::open("/carousel/autoplay", viewport).await.unwrap();
            let clear: Vec<f64> = fixture
                .page
                .evaluate_bounded(format!(
                    "(() => {{ const p = document.querySelector({PAUSE:?}).getBoundingClientRect(); \
                     const dot = document.querySelector('[aria-roledescription=carousel] [role=group] > button') \
                       .getBoundingClientRect(); \
                     return [p.bottom, dot.top]; }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                clear[0] <= clear[1],
                "the pause control reaches the dot row at {viewport:?}: {clear:?}"
            );
            fixture.close().await.unwrap();
        }
    });
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

/// Todo 2359: a press on Pause that brings no focus (Safari, iOS) leaves no stale
/// flag, so the next focus entry still stops the rotation.
#[test]
fn a_press_without_focus_leaves_the_next_entry_an_entry() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        hold_rotation(page).await;
        instant_scroll(page).await;
        // Synthetic events move no focus, as a click in Safari does not.
        let press = format!(
            "(() => {{ const b = document.querySelector({PAUSE:?}); \
             const o = {{ bubbles: true, pointerType: 'mouse', isPrimary: true }}; \
             b.dispatchEvent(new PointerEvent('pointerdown', o)); \
             b.dispatchEvent(new PointerEvent('pointerup', o)); b.click(); }})()"
        );
        page.evaluate_bounded(press.clone()).await.unwrap();
        assert_pressed(page, "true", "a focusless press on Pause").await;
        page.evaluate_bounded(press).await.unwrap();
        assert_pressed(page, "false", "a focusless press on Play").await;
        let start = index(page).await.unwrap();
        rotates(page, start, "autoplay to advance once Play is pressed").await;

        // The entry itself, on the toggle: the stale flag is read there.
        reach(page, PAUSE).await.unwrap();
        assert_pressed(page, "true", "focus entering after a focusless press").await;
        fixture
            .console
            .assert_clean("a focusless press on Pause")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3118: a focusless press on Pause released off it clicks nothing; the next focus
/// entry still stops the rotation.
#[test]
fn a_press_released_off_pause_leaves_the_next_entry_an_entry() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        hold_rotation(page).await;
        instant_scroll(page).await;
        page.evaluate_bounded(format!(
            "(() => {{ const b = document.querySelector({PAUSE:?}); \
             const t = document.querySelector({TRACK:?}); \
             const o = {{ bubbles: true, pointerType: 'mouse', isPrimary: true }}; \
             b.dispatchEvent(new PointerEvent('pointerdown', o)); \
             b.dispatchEvent(new PointerEvent('pointerout', o)); \
             b.dispatchEvent(new PointerEvent('pointerleave', {{ ...o, bubbles: false }})); \
             t.dispatchEvent(new PointerEvent('pointerup', o)); }})()"
        ))
        .await
        .unwrap();
        assert_pressed(page, "false", "a press released off Pause").await;

        reach(page, PAUSE).await.unwrap();
        assert_pressed(
            page,
            "true",
            "focus entering after a press released off Pause",
        )
        .await;
        fixture
            .console
            .assert_clean("a press released off Pause")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2393: Tab from Pause to the track moves within the carousel: no new focus entry
/// that stops the rotation Play just started.
#[test]
fn tab_from_pause_to_the_track_is_no_new_entry() {
    block_on(async {
        let fixture = Fixture::open("/carousel/autoplay", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        hold_rotation(page).await;
        instant_scroll(page).await;
        reach(page, PAUSE).await.unwrap();
        assert_pressed(page, "true", "focus entering").await;
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        assert_pressed(page, "false", "Play").await;

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement.matches({TRACK:?})"),
            "Tab to move from the toggle to the track",
        )
        .await
        .unwrap();
        clock::settle(page).await.unwrap();
        assert_pressed(page, "false", "a Tab within the carousel").await;
        let start = index(page).await.unwrap();
        rotates(page, start, "autoplay to advance after a Tab within").await;
        fixture.console.assert_clean("a Tab within").unwrap();
        fixture.close().await.unwrap();
    });
}

const CURRENT_SLIDE: &str = "[aria-roledescription=slide][data-current]";

/// Todo 2652: the desktop WebView's arm of the Tab above, whose focusout/focusin pair is one
/// move once `next_task` waits out the page's task (2664); Chromium and Blitz have their own tests.
async fn tab_from_pause_keeps_it_playing<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        d.hold_timers(&[AUTOPLAY_MS]).await?,
        "the rotation timer cannot be held"
    );
    eventually(d, "the toggle to render", async |d| d.exists(PAUSE).await).await?;
    d.focus(PAUSE).await?;
    eventually(d, "focus entering to pause", async |d| {
        Ok(d.attr(PAUSE, "aria-pressed").await?.as_deref() == Some("true"))
    })
    .await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Enter to play", async |d| {
        Ok(d.attr(PAUSE, "aria-pressed").await?.as_deref() == Some("false"))
    })
    .await?;

    d.press(keyboard::TAB).await?;
    eventually_focused(d, TRACK, "Tab from the toggle").await?;
    d.settle().await?;
    ensure!(
        d.attr(PAUSE, "aria-pressed").await?.as_deref() == Some("false"),
        "a Tab within the carousel paused it"
    );
    let before = d.text(CURRENT_SLIDE).await?;
    eventually(d, "the rotation to stay armed", async |d| {
        Ok(d.armed(AUTOPLAY_MS).await? >= 1)
    })
    .await?;
    eventually(d, "a fired rotation to advance", async |d| {
        d.fire_timers(AUTOPLAY_MS).await?;
        Ok(d.text(CURRENT_SLIDE).await? != before)
    })
    .await
}

e2e::scenario!(
    tab_from_pause_within_the_carousel,
    "/carousel/autoplay",
    tab_from_pause_keeps_it_playing,
    web: skip("tab_from_pause_to_the_track_is_no_new_entry is the web's arm"),
    native: skip("native::carousel has the Blitz arm"),
    android: skip("2652 is the desktop WebView's arm")
);

/// Todo 2361: one slide, or every slide in view, has nothing to rotate: no Pause.
#[test]
fn nothing_to_rotate_shows_no_pause() {
    block_on(async {
        let fixture = Fixture::open("/carousel/fits", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelectorAll('[aria-roledescription=carousel]').length === 2",
            "both carousels to render",
        )
        .await
        .unwrap();
        let pauses: i64 = page
            .evaluate_bounded(format!("document.querySelectorAll({PAUSE:?}).length"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(pauses, 0, "Pause controls on carousels that cannot rotate");
        fixture.console.assert_clean("nothing to rotate").unwrap();
        fixture.close().await.unwrap();
    });
}
