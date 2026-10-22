//! `Video`'s controls fade while playing untouched, in the page and in fullscreen, and
//! come back on a tap, a key or focus.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, clock, wait};

use super::{FULLSCREEN, PLAY, reads};

/// The controls' idle timer (libero's CONTROLS_IDLE_MS).
const IDLE_MS: u32 = 3000;

/// Holds the idle timer, so a test fires it instead of waiting 3 s.
async fn hold_idle<D: Driver>(d: &mut D) -> Result<()> {
    anyhow::ensure!(
        d.hold_timers(&[IDLE_MS]).await?,
        "{:?} cannot hold the idle timer",
        d.platform()
    );
    Ok(())
}

/// Fires the idle timer once it is armed and waits for the fade.
async fn until_faded<D: Driver>(d: &mut D, what: &str) -> Result<()> {
    eventually(d, "the idle timer to arm", async |d| {
        Ok(d.armed(IDLE_MS).await? > 0)
    })
    .await?;
    d.fire_timers(IDLE_MS).await?;
    if let Err(error) = eventually(d, what, faded).await {
        let seen = d
            .evaluate(
                "(() => { const a = document.activeElement; const g = document.querySelector('#player [role=group]'); \
                 return JSON.stringify({ active: a && (a.tagName + '.' + a.className + ' ' + (a.getAttribute('aria-label') || '')), \
                 focusVisible: a && a.matches(':focus-visible'), controls: g.dataset.controls, \
                 fullscreen: !!document.fullscreenElement || !!document.webkitFullscreenElement, hasFocus: document.hasFocus(), \
                 hover: g.matches(':hover') }); })()",
            )
            .await;
        anyhow::bail!("{error:#}; page: {seen:?}");
    }
    Ok(())
}

/// The controls stay shown past every pending idle timer and the frame after it.
async fn stay_shown<D: Driver>(d: &mut D, why: &str) -> Result<()> {
    d.fire_timers(IDLE_MS).await?;
    d.settle().await?;
    anyhow::ensure!(controls_shown(d).await?, "the controls faded {why}");
    Ok(())
}

/// The player and its bar, read by `faded` and `faded_js` alike.
const GROUP: &str = "#player [role=group]";
const BAR: &str = "#player [data-slot=controls]";

async fn faded<D: Driver>(d: &mut D) -> Result<bool> {
    Ok(
        d.attr(GROUP, "data-controls").await?.as_deref() == Some("hidden")
            && d.style(BAR, "opacity").await? == "0",
    )
}

/// The bar's opacity with its transition finished: a tab behind runs it at ~1 frame/s.
fn bar_opacity_js() -> String {
    format!(
        "(() => {{ const bar = document.querySelector('{BAR}'); \
         bar.getAnimations().forEach((a) => a.finish()); return getComputedStyle(bar).opacity; }})()"
    )
}

/// `faded`, for a page read.
fn faded_js() -> String {
    format!(
        "document.querySelector('{GROUP}').dataset.controls === 'hidden' && {} === '0'",
        bar_opacity_js()
    )
}

async fn controls_shown<D: Driver>(d: &mut D) -> Result<bool> {
    Ok(d.attr("#player [role=group]", "data-controls")
        .await?
        .is_none()
        && d.style("#player [data-slot=controls]", "opacity").await? == "1")
}

/// In fullscreen the controls overlay the picture and fade while playing
/// untouched; keyboard focus and a tap on the picture bring them back.
async fn fullscreen_controls_fade<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let _fullscreen = e2e::frames::keep_fullscreen().await;
    if d.platform() == Platform::Native {
        return Ok(());
    }
    hold_idle(d).await?;
    eventually(d, "the duration", async |d| {
        Ok(d.text("#player [data-slot=time]").await? == "0:00 / 0:15")
    })
    .await?;
    d.click(FULLSCREEN).await?;
    eventually(d, "the player to fill the screen", async |d| {
        Ok(d.attr("#player [role=group]", "data-fullscreen")
            .await?
            .is_some())
    })
    .await?;
    let position = d.style("#player [data-slot=controls]", "position").await?;
    anyhow::ensure!(
        position == "absolute",
        "the controls are {position}, not overlaid"
    );
    d.click(PLAY).await?;
    until_faded(d, "the controls to fade while playing").await?;

    d.click("#player video").await?;
    eventually(d, "a press on the picture to show them", controls_shown).await?;
    // A click pauses too; a tap on faded controls only shows them (todo 1362).
    if d.platform() != Platform::Android {
        reads(d, "the click to pause", PLAY, "aria-label", "Play").await?;
        d.click(PLAY).await?;
    }
    reads(d, "playing", PLAY, "aria-label", "Pause").await?;
    until_faded(d, "the controls to fade again").await?;

    d.press(keyboard::TAB).await?;
    eventually(d, "keyboard focus to show them", controls_shown).await?;
    stay_shown(d, "under keyboard focus").await?;
    d.press(keyboard::ESCAPE).await
}

/// Todo 1342: in the page too the bar fades while playing untouched, comes back
/// on a tap, and stays while paused.
async fn inline_controls_fade<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() == Platform::Native {
        return Ok(());
    }
    hold_idle(d).await?;
    eventually(d, "the duration", async |d| {
        Ok(d.text("#player [data-slot=time]").await? == "0:00 / 0:15")
    })
    .await?;
    d.click(PLAY).await?;
    until_faded(d, "the controls to fade while playing").await?;
    d.click("#player video").await?;
    eventually(d, "a press on the picture to show them", controls_shown).await?;
    // A click pauses as well; a tap on faded controls only shows them (todo 1362).
    if d.platform() == Platform::Android {
        d.click(PLAY).await?;
    }
    reads(d, "playing to pause", PLAY, "aria-label", "Play").await?;
    stay_shown(d, "while paused").await
}

e2e::scenario!(
    the_controls_fade_while_playing,
    "/video/long",
    inline_controls_fade
);

e2e::scenario!(
    in_fullscreen_the_controls_fade_and_come_back,
    "/video/long",
    fullscreen_controls_fade
);

/// Fires the held idle timer once armed and waits for the fade.
async fn fade(page: &chromiumoxide::Page) {
    clock::until_armed(page, IDLE_MS, 1, "the idle timer")
        .await
        .unwrap();
    clock::fire(page, IDLE_MS).await.unwrap();
    wait::for_js_true(page, &faded_js(), "the controls to fade")
        .await
        .unwrap();
}

// The play button's name too: `paused` flips at `play()`, before the player's own state,
// and a click that lands in between toggles from the stale one.
const PLAYING: &str = "!document.querySelector('#player video').paused
    && document.querySelector('#player [data-slot=controls] button').getAttribute('aria-label') === 'Pause'";
const PAUSED: &str = "document.querySelector('#player video').paused
    && document.querySelector('#player [data-slot=controls] button').getAttribute('aria-label') === 'Play'";
const LONG_DURATION: &str =
    "document.querySelector('#player [data-slot=time]').textContent === '0:00 / 0:15'";

/// Todo 1362: a click on the picture plays and pauses; a tap on faded controls only
/// shows them, and once shown a tap pauses.
#[test]
fn a_click_on_the_picture_plays_or_pauses() {
    block_on(async {
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        clock::hold(page, &[IDLE_MS]).await.unwrap();
        wait::for_js_true(page, LONG_DURATION, "the duration")
            .await
            .unwrap();
        pointer::click(page, "#player video").await.unwrap();
        wait::for_js_true(page, PLAYING, "a click to play")
            .await
            .unwrap();
        pointer::click(page, "#player video").await.unwrap();
        wait::for_js_true(page, PAUSED, "a click to pause")
            .await
            .unwrap();

        pointer::click(page, "#player video").await.unwrap();
        // The idle timer armed while paused is replaced once playing: fire the new one.
        wait::for_js_true(page, PLAYING, "a click to play again")
            .await
            .unwrap();
        clock::settle(page).await.unwrap();
        fade(page).await;
        let at = pointer::centre_of(page, "#player video").await.unwrap();
        pointer::touch_drag(page, at, at, 0).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!({}) && {PLAYING}", faded_js()),
            "a tap to show the controls and keep playing",
        )
        .await
        .unwrap();
        pointer::touch_drag(page, at, at, 0).await.unwrap();
        wait::for_js_true(page, PAUSED, "a tap on shown controls to pause")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2681: an error mid-playback leaves the element unpaused and short of data; its alert
/// brings the faded controls back, and the status reads no buffering beside it.
#[test]
fn a_mid_playback_error_shows_the_controls_and_no_buffering() {
    block_on(async {
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        clock::hold(page, &[IDLE_MS]).await.unwrap();
        wait::for_js_true(page, LONG_DURATION, "the duration")
            .await
            .unwrap();
        pointer::click(page, PLAY).await.unwrap();
        wait::for_js_true(page, PLAYING, "playing").await.unwrap();
        fade(page).await;
        // A stream dying on cue cannot be served: the element's state is faked as one leaves it.
        page.evaluate_bounded(
            "(() => { const video = document.querySelector('#player video');
             Object.defineProperty(video, 'error', { get: () => ({ code: 2 }) });
             Object.defineProperty(video, 'readyState', { get: () => 1 });
             Object.defineProperty(video, 'paused', { get: () => false });
             video.dispatchEvent(new Event('error')); })()",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('#player [role=alert]') !== null
                 && document.querySelector('{GROUP}').dataset.controls === undefined && {} === '1'",
                bar_opacity_js()
            ),
            "the alert to bring the controls back",
        )
        .await
        .unwrap();
        let status: String = page
            .evaluate_bounded("document.querySelector('#player [role=status]').textContent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            status.is_empty(),
            "the status reads {status:?} beside the error"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 1249: focus moved into faded controls by code, with no key, shows them.
#[test]
fn focus_from_code_shows_the_faded_controls() {
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;
        clock::hold(page, &[IDLE_MS]).await.unwrap();
        wait::for_js_true(page, LONG_DURATION, "the duration")
            .await
            .unwrap();
        pointer::click(page, PLAY).await.unwrap();
        fade(page).await;
        page.evaluate_bounded(format!("document.querySelector('{FULLSCREEN}').focus()"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{GROUP}').dataset.controls === undefined && {} === '1'",
                bar_opacity_js()
            ),
            "focus from code to show the row",
        )
        .await
        .unwrap();
        // As after a key, they stay while focus is in them: past the idle timer.
        clock::fire_all(page, IDLE_MS).await.unwrap();
        clock::settle(page).await.unwrap();
        let faded: bool = page
            .evaluate_bounded(faded_js())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!faded, "the row faded with focus in it");
        fixture.close().await.unwrap();
    });
}

/// Todo 3660: a faded bar takes no hit, so a tap on it reaches the picture and only shows the controls.
#[test]
fn a_tap_on_the_faded_bar_does_not_press_its_button() {
    const MUTE: &str = "#player button[aria-label=Mute]";
    block_on(async {
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        clock::hold(page, &[IDLE_MS]).await.unwrap();
        wait::for_js_true(page, LONG_DURATION, "the duration")
            .await
            .unwrap();
        pointer::click(page, PLAY).await.unwrap();
        wait::for_js_true(page, PLAYING, "playing").await.unwrap();
        fade(page).await;
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const r = document.querySelector('{MUTE}').getBoundingClientRect();
                 return document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)
                     === document.querySelector('#player video'); }})()"
            ),
            "the picture, not the faded button, under the tap",
        )
        .await
        .unwrap();
        let at = pointer::centre_of(page, MUTE).await.unwrap();
        pointer::touch_drag(page, at, at, 0).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!({})", faded_js()),
            "a tap on the bar to show the controls",
        )
        .await
        .unwrap();
        clock::settle(page).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "!document.querySelector('#player video').muted
                 && document.querySelector('{MUTE}') !== null && {PLAYING}"
            ),
            "the tap to leave the sound on",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3656: Space on a focused button starts playing from paused, and the bar stays while focus is in it.
#[test]
fn a_space_on_the_focused_play_button_keeps_the_controls_shown() {
    block_on(async {
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        clock::hold(page, &[IDLE_MS]).await.unwrap();
        wait::for_js_true(page, LONG_DURATION, "the duration")
            .await
            .unwrap();
        page.evaluate_bounded(format!("document.querySelector('{PLAY}').focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(page, PLAYING, "Space to play")
            .await
            .unwrap();
        clock::until_armed(page, IDLE_MS, 1, "the idle timer")
            .await
            .unwrap();
        clock::fire_all(page, IDLE_MS).await.unwrap();
        clock::settle(page).await.unwrap();
        let faded: bool = page
            .evaluate_bounded(faded_js())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!faded, "the row faded with focus on the play button");
        fixture.close().await.unwrap();
    });
}

/// Todo 3656: a mouse press leaves a ring-less focus on its button, so a mouse leaving still fades the bar.
#[test]
fn a_mouse_press_then_leave_fades_the_bar_with_focus_on_the_button() {
    block_on(async {
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        clock::hold(page, &[IDLE_MS]).await.unwrap();
        wait::for_js_true(page, LONG_DURATION, "the duration")
            .await
            .unwrap();
        pointer::click(page, PLAY).await.unwrap();
        wait::for_js_true(page, PLAYING, "playing").await.unwrap();
        page.evaluate_bounded(format!(
            "document.querySelector('{GROUP}').dispatchEvent(
                 new PointerEvent('pointerleave', {{ pointerType: 'mouse' }}))"
        ))
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && document.querySelector('{BAR}').contains(document.activeElement)",
                faded_js()
            ),
            "the bar to fade with focus on its button",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1418: a press the browser cancels ends too, so focus from code shows the row after it.
#[test]
fn a_cancelled_press_does_not_stop_focus_showing_the_controls() {
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;
        clock::hold(page, &[IDLE_MS]).await.unwrap();
        wait::for_js_true(page, LONG_DURATION, "the duration")
            .await
            .unwrap();
        pointer::click(page, PLAY).await.unwrap();
        wait::for_js_true(page, PLAYING, "playing").await.unwrap();
        page.evaluate_bounded(
            "(() => { const video = document.querySelector('#player video');
             for (const type of ['pointerdown', 'pointercancel'])
                 video.dispatchEvent(new PointerEvent(type, { bubbles: true, pointerType: 'touch' })); })()",
        )
        .await
        .unwrap();
        fade(page).await;
        page.evaluate_bounded(format!("document.querySelector('{FULLSCREEN}').focus()"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("!({})", faded_js()),
            "focus from code to show the row",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
