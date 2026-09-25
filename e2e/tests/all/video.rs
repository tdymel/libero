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
        if state == "pseudo" {
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
    fullscreen.as_deref() == Some("pseudo")
}

/// The drawn box covers the page, so a Tab out of the player gives the page back;
/// a Tab between its controls does not.
async fn tab_out_leaves_pseudo<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
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
    if !pseudo_state(d.attr("#player [role=group]", "data-fullscreen").await?) {
        // Granted for real (WebKitGTK): the renderer keeps focus in, nothing to test.
        return d.press(keyboard::ESCAPE).await;
    }
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

e2e::scenario!(
    a_tab_out_of_the_drawn_fullscreen_leaves_it,
    "/video",
    tab_out_leaves_pseudo
);
