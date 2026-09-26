//! `Video`: fullscreen takes the whole player and gives it back, the captions
//! button shows the subtitle track, and F and C act inside the player.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::keyboard::{self, Key};
use e2e::passes::pointer;
use e2e::{Fixture, Suite, Viewport, wait};

const PLAY: &str = "#player [role=toolbar] button";
const FULLSCREEN: &str = "#player button[aria-label=Fullscreen]";
const CAPTIONS: &str = "#player button[aria-label=Captions]";

const F: Key = Key {
    key: "f",
    code: "KeyF",
    vk: 70,
    text: Some("f"),
};
const C: Key = Key {
    key: "c",
    code: "KeyC",
    vk: 67,
    text: Some("c"),
};

#[test]
fn it_meets_the_baseline() {
    Suite::new("video", "/video")
        .focusable(PLAY)
        .targets(PLAY)
        .run();
}

/// WCAG 1.4.10: at a phone's 412px and at 320px the whole row fits in the player,
/// the volume slider and then the total time giving way.
#[test]
fn the_controls_fit_a_narrow_player() {
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        for (width, volume, total) in [(412, "none", "inline"), (320, "none", "none")] {
            page.evaluate(format!(
                "document.querySelector('#player').style.width = '{width}px'"
            ))
            .await
            .unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const player = document.querySelector('#player [role=group]').getBoundingClientRect();
                     const button = document.querySelector('{FULLSCREEN}').getBoundingClientRect();
                     const style = (s) => getComputedStyle(document.querySelector('#player ' + s)).display;
                     return player.width === {width} && button.right <= player.right && button.left >= player.left
                         && style('[data-slot=volume]') === '{volume}'
                         && style('[data-slot=time] > span') === '{total}'; }})()"
                ),
                &format!("the row to fit {width}px"),
            )
            .await
            .unwrap();
        }
        fixture.close().await.unwrap();
    });
}

fn track_mode() -> &'static str {
    "document.querySelector('#player video').textTracks[0].mode"
}

const STATE: &str = "(document.querySelector('#player [role=group]').dataset.fullscreen ?? 'none')";

/// Headless Chromium refuses the Fullscreen API, so this also covers the fallback:
/// the player as a fixed box over the page, which Escape and F leave.
#[test]
fn fullscreen_takes_the_player_and_escape_or_f_gives_it_back() {
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "document.querySelector('#player [data-slot=time]').textContent === '0:00 / 0:04'",
            "the duration",
        )
        .await
        .unwrap();
        pointer::click(page, FULLSCREEN).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{STATE} !== 'none' \
                 && document.querySelector('#player button[aria-label=\"Exit fullscreen\"]') !== null"
            ),
            "the player to fill the screen",
        )
        .await
        .unwrap();
        let state: String = page.evaluate(STATE).await.unwrap().into_value().unwrap();
        if state == "drawn" {
            wait::for_js_true(
                page,
                "(() => { const r = document.querySelector('#player [role=group]').getBoundingClientRect(); \
                 return r.top === 0 && r.left === 0 && r.width === innerWidth && r.height === innerHeight; })()",
                "the fixed box to cover the viewport",
            )
            .await
            .unwrap();
            keyboard::press(page, keyboard::ESCAPE).await.unwrap();
            wait::for_js_true(page, &format!("{STATE} === 'none'"), "Escape to leave")
                .await
                .unwrap();
            keyboard::press(page, F).await.unwrap();
            wait::for_js_true(page, &format!("{STATE} !== 'none'"), "F to enter")
                .await
                .unwrap();
        }

        keyboard::press(page, F).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{STATE} === 'none' && document.fullscreenElement === null"),
            "F to leave fullscreen",
        )
        .await
        .unwrap();

        fixture.close().await.unwrap();
    });
}

#[test]
fn the_captions_button_and_c_toggle_the_subtitle_track() {
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{} === 'disabled'", track_mode()),
            "the track off at the start",
        )
        .await
        .unwrap();
        pointer::click(page, CAPTIONS).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} === 'showing' && document.querySelector('{CAPTIONS}').getAttribute('aria-pressed') === 'true'",
                track_mode()
            ),
            "the button to show the track",
        )
        .await
        .unwrap();

        keyboard::press(page, C).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} === 'hidden' && document.querySelector('{CAPTIONS}').getAttribute('aria-pressed') === 'false'",
                track_mode()
            ),
            "C to hide it",
        )
        .await
        .unwrap();

        let plain: bool = page
            .evaluate("document.querySelector('#plain button[aria-label=Captions]') === null")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(plain, "a player without tracks has no captions button");

        fixture.close().await.unwrap();
    });
}

async fn reads<D: Driver>(
    d: &mut D,
    what: &str,
    selector: &str,
    name: &str,
    want: &str,
) -> Result<()> {
    eventually(d, what, async |d| {
        Ok(d.attr(selector, name).await?.as_deref() == Some(want))
    })
    .await
}

/// Shared by `Audio` and `Video` (todo 1234): the state script reaches Rust on
/// every renderer that plays media, and Blitz shows the fallback instead.
async fn the_player_follows_its_element<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() == Platform::Native {
        eventually(d, "the fallback", async |d| {
            Ok(d.exists("#player [data-slot=message] a").await?
                && !d.exists("#player [role=toolbar]").await?)
        })
        .await?;
        return Ok(());
    }
    eventually(d, "the duration", async |d| {
        Ok(d.text("#player [data-slot=time]").await? == "0:00 / 0:04")
    })
    .await?;
    d.click(PLAY).await?;
    reads(d, "the button to offer Pause", PLAY, "aria-label", "Pause").await?;
    // Each `timeupdate` that passes the throttle moves the time on.
    eventually(d, "the time to move", async |d| {
        let time = d.text("#player [data-slot=time]").await?;
        Ok(time != "0:00 / 0:04")
    })
    .await?;
    reads(d, "the end to offer Play again", PLAY, "aria-label", "Play").await
}

e2e::scenario!(
    an_audio_player_follows_its_element,
    "/audio",
    the_player_follows_its_element
);
e2e::scenario!(
    a_video_player_follows_its_element,
    "/video",
    the_player_follows_its_element
);

/// Native or pseudo, whichever the renderer grants; Blitz has no controls to press.
async fn fullscreen_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() == Platform::Native {
        return Ok(());
    }
    eventually(d, "the controls", async |d| d.exists(FULLSCREEN).await).await?;
    d.click(FULLSCREEN).await?;
    eventually(d, "the player to fill the screen", async |d| {
        Ok(d.attr("#player [role=group]", "data-fullscreen")
            .await?
            .is_some())
    })
    .await?;
    d.click("#player button[aria-label='Exit fullscreen']")
        .await?;
    eventually(d, "the player back in the page", async |d| {
        Ok(d.attr("#player [role=group]", "data-fullscreen")
            .await?
            .is_none())
    })
    .await
}

e2e::scenario!(the_fullscreen_button_toggles, "/video", fullscreen_toggles);

fn pseudo_state(fullscreen: Option<String>) -> bool {
    fullscreen.as_deref() == Some("drawn")
}

/// The drawn box covers the page, so a Tab out of the player gives the page back;
/// a Tab between its controls does not. The route refuses the Fullscreen API, so
/// WebKitGTK, which grants it, draws the box too.
async fn tab_out_leaves_pseudo<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() == Platform::Native {
        return Ok(());
    }
    eventually(d, "the refusal and the controls", async |d| {
        Ok(d.exists("#player[data-refused]").await? && d.exists(FULLSCREEN).await?)
    })
    .await?;
    d.click(FULLSCREEN).await?;
    eventually(d, "the drawn box", async |d| {
        Ok(pseudo_state(
            d.attr("#player [role=group]", "data-fullscreen").await?,
        ))
    })
    .await?;
    d.focus(PLAY).await?;
    d.press(keyboard::TAB).await?;
    d.idle().await;
    let inside = d.attr("#player [role=group]", "data-fullscreen").await?;
    anyhow::ensure!(pseudo_state(inside), "a Tab inside the player left the box");
    d.focus("#player button[aria-label='Exit fullscreen']")
        .await?;
    d.press(keyboard::TAB).await?;
    eventually(d, "a Tab out to give the page back", async |d| {
        Ok(d.attr("#player [role=group]", "data-fullscreen")
            .await?
            .is_none())
    })
    .await
}

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

async fn faded<D: Driver>(d: &mut D) -> Result<bool> {
    Ok(d.attr("#player [role=group]", "data-controls")
        .await?
        .as_deref()
        == Some("hidden")
        && d.style("#player [data-slot=controls]", "opacity").await? == "0")
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
    if d.platform() == Platform::Native {
        return Ok(());
    }
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
    within(d, 8, "the controls to fade while playing", faded).await?;

    d.click("#player video").await?;
    within(d, 3, "a tap on the picture to show them", controls_shown).await?;
    reads(
        d,
        "the tap to leave playing alone",
        PLAY,
        "aria-label",
        "Pause",
    )
    .await?;
    within(d, 8, "the controls to fade again", faded).await?;

    d.press(keyboard::TAB).await?;
    within(d, 3, "keyboard focus to show them", controls_shown).await?;
    let started = std::time::Instant::now();
    while started.elapsed().as_secs() < 4 {
        anyhow::ensure!(
            controls_shown(d).await?,
            "the controls faded under keyboard focus"
        );
        d.idle().await;
    }
    d.press(keyboard::ESCAPE).await
}

e2e::scenario!(
    in_fullscreen_the_controls_fade_and_come_back,
    "/video/long",
    fullscreen_controls_fade
);

e2e::scenario!(
    a_tab_out_of_the_drawn_fullscreen_leaves_it,
    "/video/refused",
    tab_out_leaves_pseudo
);
