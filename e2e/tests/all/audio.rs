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

/// WCAG 1.4.10: from 360px down to 320px the whole row fits in the player, the
/// volume slider giving way at 352px and the mute button staying.
#[test]
fn the_controls_fit_a_narrow_player() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        for (width, volume) in [(360, true), (353, true), (352, false), (320, false)] {
            page.evaluate(format!(
                "document.querySelector('#player').style.width = '{width}px'"
            ))
            .await
            .unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const player = document.querySelector('#player [role=group]').getBoundingClientRect();
                     const row = [...document.querySelectorAll('#player [data-slot=controls] > *')]
                         .filter((e) => getComputedStyle(e).display !== 'none');
                     const fits = row.every((e) => {{ const r = e.getBoundingClientRect();
                         return r.left >= player.left && r.right <= player.right; }});
                     const shown = getComputedStyle(document.querySelector('#player [data-slot=volume]')).display !== 'none';
                     const mute = document.querySelector('#player button[aria-label=Mute]');
                     return player.width === {width} && fits && shown === {volume}
                         && mute !== null && getComputedStyle(mute).display !== 'none'; }})()"
                ),
                &format!("the row to fit {width}px"),
            )
            .await
            .unwrap();
        }
        fixture.close().await.unwrap();
    });
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
            &format!(
                "{} === '0:00 / 0:04' && {}",
                time_text("player"),
                audio("player", "a.currentSrc.startsWith('data:audio/wav')")
            ),
            "the duration, from the `src` after an unplayable source",
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

/// Volume 0 counts as silent: the button offers Unmute, which brings back the
/// last audible volume; a volume moved up while muted unmutes.
#[test]
fn the_mute_button_follows_silence() {
    const VOLUME: &str = "#player [data-slot=volume] [role=slider]";
    let mute_offers = |label: &str| {
        format!("document.querySelector('#player button[aria-label={label}]') !== null")
    };
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
        page.evaluate(format!("document.querySelector('{VOLUME}').focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        keyboard::press(page, keyboard::HOME).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && {}",
                audio("player", "a.volume === 0 && !a.muted"),
                mute_offers("Unmute")
            ),
            "volume 0 to offer Unmute",
        )
        .await
        .unwrap();

        pointer::click(page, "#player button[aria-label=Unmute]")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && {}",
                audio("player", "Math.abs(a.volume - 0.95) < 1e-6 && !a.muted"),
                mute_offers("Mute")
            ),
            "Unmute to bring back the last audible volume",
        )
        .await
        .unwrap();

        pointer::click(page, "#player button[aria-label=Mute]")
            .await
            .unwrap();
        wait::for_js_true(page, &audio("player", "a.muted"), "Mute to mute")
            .await
            .unwrap();
        page.evaluate(format!("document.querySelector('{VOLUME}').focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && {}",
                audio("player", "a.volume === 1 && !a.muted"),
                mute_offers("Mute")
            ),
            "a volume moved up to unmute",
        )
        .await
        .unwrap();

        // M from volume 0 restores as the button does.
        keyboard::press(page, keyboard::HOME).await.unwrap();
        keyboard::press(page, M).await.unwrap();
        wait::for_js_true(
            page,
            &audio("player", "a.volume === 1 && !a.muted"),
            "M at volume 0 to bring back the volume",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("muting").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A file swapped after mount behind `sources` is loaded, not the first one kept.
#[test]
fn a_swapped_source_is_loaded() {
    block_on(async {
        let fixture = Fixture::open("/audio/swap", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{} === '0:00 / 0:04'", time_text("swap")),
            "the 4 s file",
        )
        .await
        .unwrap();
        pointer::click(page, "#swap-button").await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '0:00 / 0:02'", time_text("swap")),
            "the 2 s file",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("swapping").unwrap();
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
