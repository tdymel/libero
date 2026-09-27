//! `Audio` and `use_media`: the controls follow the element, keys act only
//! inside the player, and a broken source says so.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, Key};
use e2e::passes::pointer;
use e2e::{Fixture, Suite, Viewport, wait};

const PLAY: &str = "#player [data-slot=controls] button";

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

/// WCAG 1.4.10, todo 1325: a bubble of at most 22rem; from 352px down to 320px
/// the row, volume included, fits on one line; at 160px (320px at 200% zoom)
/// it may wrap, but nothing leaves the player. A shrink-wrapping parent does
/// not collapse it.
#[test]
fn the_controls_fit_a_narrow_player() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        for (width, bubble) in [(500, 352), (352, 352), (320, 320), (240, 240), (160, 160)] {
            page.evaluate(format!(
                "document.querySelector('#player').style.width = '{width}px'"
            ))
            .await
            .unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const player = document.querySelector('#player [role=group]').getBoundingClientRect();
                     const row = [...document.querySelectorAll('#player [data-slot=controls] > *')];
                     const fits = row.every((e) => {{ const r = e.getBoundingClientRect();
                         return r.left >= player.left && r.right <= player.right; }});
                     const lines = new Set(row.map((e) => {{ const r = e.getBoundingClientRect(); return Math.round(r.top + r.height / 2); }})).size;
                     const oneRow = {width} < 320 || lines === 1;
                     return row.length === 5 && player.width === {bubble} && fits && oneRow; }})()"
                ),
                &format!("the row to fit {width}px"),
            )
            .await
            .unwrap();
        }
        wait::for_js_true(
            page,
            "document.querySelector('#inline [role=group]').getBoundingClientRect().width === 352",
            "the bubble in an inline-flex parent to take its full 22rem",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// No mute button: a muted player shows its volume at 0, and moving it up unmutes.
#[test]
fn a_muted_player_shows_volume_0() {
    const VOLUME: &str = "#muted [data-slot=volume] [role=slider]";
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!(
                "{} && document.querySelector('{VOLUME}').getAttribute('aria-valuenow') === '0'
                 && document.querySelector('#muted button[aria-label=Mute], #muted button[aria-label=Unmute]') === null",
                audio("muted", "a.muted")
            ),
            "the muted volume at 0, no mute button",
        )
        .await
        .unwrap();
        page.evaluate(format!("document.querySelector('{VOLUME}').focus()"))
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            &audio("muted", "!a.muted && Math.abs(a.volume - 0.05) < 1e-6"),
            "a volume moved up to unmute",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("muted").unwrap();
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
                "{} === '0:04' && {}",
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

/// L jumps ahead, but only with focus inside the player; M mutes nothing, as
/// there is no mute button to show it.
#[test]
fn keys_act_inside_the_player_only() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{} === '0:04'", time_text("player")),
            "the duration",
        )
        .await
        .unwrap();
        // Outside: nothing.
        keyboard::press(page, L).await.unwrap();
        keyboard::tab_to(page, PLAY, 5).await.unwrap();
        let time: f64 = page
            .evaluate(audio("player", "a.currentTime"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(time, 0.0, "L outside the player jumped");

        keyboard::press(page, M).await.unwrap();
        // Space on the seek slider plays; the hotkeys stay off Space, which a button keeps.
        page.evaluate("document.querySelector('#player [data-slot=seek] [role=slider]').focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &audio("player", "!a.paused && !a.muted"),
            "Space on the slider to play, M to leave the sound on",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("{} !== '0:04'", time_text("player")),
            "the time to show the elapsed once playing",
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

/// The speed button steps 1x, 1.5x, 2x and back, its name following.
#[test]
fn the_speed_button_cycles() {
    const SPEED: &str = "#player [data-slot=controls] > button:last-child";
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        for rate in ["1.5", "2", "1"] {
            pointer::click(page, SPEED).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "{} && document.querySelector('{SPEED}').getAttribute('aria-label') === 'Playback speed {rate}×'
                     && document.querySelector('{SPEED}').textContent === '{rate}×'",
                    audio("player", &format!("a.playbackRate === {rate}"))
                ),
                &format!("the rate {rate}"),
            )
            .await
            .unwrap();
        }

        fixture.console.assert_clean("speed").unwrap();
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
            &format!("{} === '0:04'", time_text("swap")),
            "the 4 s file",
        )
        .await
        .unwrap();
        pointer::click(page, "#swap-button").await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '0:02'", time_text("swap")),
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
