//! `Audio` and `use_media`: the controls follow the element, keys act only
//! inside the player, and a broken source says so.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, Key};
use e2e::passes::pointer;
use e2e::{Fixture, Suite, Viewport, wait};

const PLAY: &str = "#player [role=toolbar] button";

const M: Key = Key {
    key: "m",
    code: "KeyM",
    vk: 77,
    text: Some("m"),
};
const L: Key = Key {
    key: "l",
    code: "KeyL",
    vk: 76,
    text: Some("l"),
};

#[test]
fn it_meets_the_baseline() {
    Suite::new("audio", "/audio")
        .focusable(PLAY)
        .targets(PLAY)
        .run();
}

fn audio(id: &str, js: &str) -> String {
    format!("(() => {{ const a = document.querySelector('#{id} audio'); return {js}; }})()")
}

fn time_text(id: &str) -> String {
    format!("document.querySelector('#{id} [data-slot=time]').textContent")
}

/// A press plays, the button and the time follow the element; Space on the
/// focused button toggles once, not twice.
#[test]
fn the_controls_follow_the_element() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{} === '0:00 / 0:04'", time_text("player")),
            "the duration",
        )
        .await
        .unwrap();
        pointer::click(page, PLAY).await.unwrap();
        wait::for_js_true(page, &audio("player", "!a.paused"), "playing")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('{PLAY}').getAttribute('aria-label') === 'Pause'"),
            "the button to offer Pause",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &audio("player", "a.ended"), "the end")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('{PLAY}').getAttribute('aria-label') === 'Play'"),
            "the button to offer Play again",
        )
        .await
        .unwrap();

        // Focus is on the play button now: Space presses it, once.
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(page, &audio("player", "!a.paused"), "Space to play")
            .await
            .unwrap();

        fixture.console.assert_clean("playing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// M mutes and L jumps ahead, but only with focus inside the player.
#[test]
fn keys_act_inside_the_player_only() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{} === '0:00 / 0:04'", time_text("player")),
            "the duration",
        )
        .await
        .unwrap();
        // Outside: nothing.
        keyboard::press(page, M).await.unwrap();
        keyboard::tab_to(page, PLAY, 5).await.unwrap();
        let muted: bool = page
            .evaluate(audio("player", "a.muted"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!muted, "M outside the player muted it");

        keyboard::press(page, M).await.unwrap();
        wait::for_js_true(page, &audio("player", "a.muted"), "M to mute")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#player button[aria-label=Unmute]') !== null",
            "the mute button to offer Unmute",
        )
        .await
        .unwrap();

        // Space on a slider plays; the hotkeys stay off Space, which a button keeps.
        page.evaluate("document.querySelector('#player [data-slot=volume] [role=slider]').focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &audio("player", "!a.paused"),
            "Space on a slider to play",
        )
        .await
        .unwrap();

        keyboard::press(page, L).await.unwrap();
        wait::for_js_true(
            page,
            &audio("player", "a.currentTime >= 3.9"),
            "L to jump, clamped to the end",
        )
        .await
        .unwrap();

        fixture.close().await.unwrap();
    });
}

#[test]
fn a_broken_source_is_announced_and_the_hook_reads_a_bare_element() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "document.querySelector('#broken [role=alert]')?.textContent === 'This media could not be played.'",
            "the error message",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#custom-state').textContent === 'supported=true duration=4'",
            "use_media to read the bare element",
        )
        .await
        .unwrap();

        fixture.close().await.unwrap();
    });
}
