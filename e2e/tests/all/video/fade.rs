//! `Video`'s controls fade while playing untouched, in the page and in fullscreen, and
//! come back on a tap, a key or focus.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, clock, wait};

use super::{FULLSCREEN, PLAY, reads};

/// Polls `probe` for `secs`, past the driver's budget: the controls fade after 3 s.
async fn within<D: Driver>(
    d: &mut D,
    secs: u64,
    what: &str,
    mut probe: impl AsyncFnMut(&mut D) -> Result<bool>,
) -> Result<()> {
    let started = std::time::Instant::now();
    while !probe(d).await? {
        anyhow::ensure!(
            started.elapsed().as_secs() < secs,
            "{:?}: gave up waiting for {what}",
            d.platform()
        );
        d.idle().await;
    }
    Ok(())
}

/// The controls' idle timer (libero's CONTROLS_IDLE_MS).
const IDLE_MS: u32 = 3000;

/// How long the controls may take to show: 3 s, before the real idle timer fades them again;
/// a held clock cannot, so the poll budget (a background tab's fade-in runs at ~1 frame/s).
fn show_secs<D: Driver>(d: &D, held: bool) -> u64 {
    match held {
        true => d.budget().as_secs(),
        false => 3,
    }
}

/// Waits for the controls to fade: on a held clock fires the idle timer once it is armed,
/// elsewhere waits out the real 3 s.
async fn until_faded<D: Driver>(d: &mut D, held: bool, what: &str) -> Result<()> {
    if held {
        eventually(d, "the idle timer to arm", async |d| {
            Ok(d.armed(IDLE_MS).await? > 0)
        })
        .await?;
        d.fire_timers(IDLE_MS).await?;
    }
    within(d, 8, what, faded).await
}

/// The controls stay shown: on a held clock past every pending idle timer and the frame
/// after it, elsewhere for 4 s of real time.
async fn stay_shown<D: Driver>(d: &mut D, held: bool, why: &str) -> Result<()> {
    if held {
        d.fire_timers(IDLE_MS).await?;
        d.settle().await?;
        anyhow::ensure!(controls_shown(d).await?, "the controls faded {why}");
        return Ok(());
    }
    let started = std::time::Instant::now();
    while started.elapsed().as_secs() < 4 {
        anyhow::ensure!(controls_shown(d).await?, "the controls faded {why}");
        d.idle().await;
    }
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
    let held = d.hold_timers(&[IDLE_MS]).await?;
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
    until_faded(d, held, "the controls to fade while playing").await?;

    d.click("#player video").await?;
    eventually(d, "a press on the picture to show them", controls_shown).await?;
    // A click pauses too; a tap on faded controls only shows them (todo 1362).
    if d.platform() != Platform::Android {
        reads(d, "the click to pause", PLAY, "aria-label", "Play").await?;
        d.click(PLAY).await?;
    }
    reads(d, "playing", PLAY, "aria-label", "Pause").await?;
    until_faded(d, held, "the controls to fade again").await?;

    d.press(keyboard::TAB).await?;
    within(
        d,
        show_secs(d, held),
        "keyboard focus to show them",
        controls_shown,
    )
    .await?;
    stay_shown(d, held, "under keyboard focus").await?;
    d.press(keyboard::ESCAPE).await
}

/// Todo 1342: in the page too the bar fades while playing untouched, comes back
/// on a tap, and stays while paused.
async fn inline_controls_fade<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() == Platform::Native {
        return Ok(());
    }
    let held = d.hold_timers(&[IDLE_MS]).await?;
    eventually(d, "the duration", async |d| {
        Ok(d.text("#player [data-slot=time]").await? == "0:00 / 0:15")
    })
    .await?;
    d.click(PLAY).await?;
    until_faded(d, held, "the controls to fade while playing").await?;
    d.click("#player video").await?;
    eventually(d, "a press on the picture to show them", controls_shown).await?;
    // A click pauses as well; a tap on faded controls only shows them (todo 1362).
    if d.platform() == Platform::Android {
        d.click(PLAY).await?;
    }
    reads(d, "playing to pause", PLAY, "aria-label", "Play").await?;
    stay_shown(d, held, "while paused").await
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
        page.evaluate(format!("document.querySelector('{FULLSCREEN}').focus()"))
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
            .evaluate(faded_js())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!faded, "the row faded with focus in it");
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
        page.evaluate(
            "(() => { const video = document.querySelector('#player video');
             for (const type of ['pointerdown', 'pointercancel'])
                 video.dispatchEvent(new PointerEvent(type, { bubbles: true, pointerType: 'touch' })); })()",
        )
        .await
        .unwrap();
        fade(page).await;
        page.evaluate(format!("document.querySelector('{FULLSCREEN}').focus()"))
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
