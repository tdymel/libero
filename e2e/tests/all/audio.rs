//! `Audio` and `use_media`: the controls follow the element, keys act only
//! inside the player, and a broken source says so.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, Key};
use e2e::passes::{pointer, target_size};
use e2e::{Fixture, Suite, Viewport, js, wait};

const PLAY: &str = "#player [data-slot=controls] button";
/// The other Tab stops `PLAY`'s first match leaves out (todo 1823).
const SEEK: &str = "#player [data-slot=seek] [role=slider]";
const CHEVRON: &str = "#player [data-slot=volume] button[aria-haspopup=dialog]";
const SPEED: &str = "#player [data-slot=controls] button[aria-label^='Playback speed']";

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
        .focusable(SEEK)
        .focusable(CHEVRON)
        .focusable(SPEED)
        .targets(PLAY)
        .run();
}

/// Todo 1394: `volume_parts` reaches the portaled card and its slider, past the 8rem default.
#[test]
fn volume_parts_style_the_portaled_menu() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let chevron = "#inline [data-slot=volume] button[aria-haspopup=dialog]";
        wait::for_js_true(
            page,
            &format!("document.querySelector('{chevron}') !== null"),
            "the controls",
        )
        .await
        .unwrap();
        broken_player_settled(page).await;
        pointer::click(page, chevron).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const card = document.querySelector('[data-slot=volume-card]');
             const slider = card?.querySelector(':scope > [data-slot=volume-slider]');
             return !!slider && getComputedStyle(card).paddingTop === '13px'
                 && Math.abs(slider.getBoundingClientRect().width - 160) < 1; })()",
            "the card's padding and the slider's width from volume_parts",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("volume parts").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1465: a control's tooltip opens only after 2s of hover, as Video's does; the delay
/// runs on the held clock (todo 1824).
#[test]
fn a_control_tooltip_waits_two_seconds_on_hover() {
    const TIP: &str = "[...document.querySelectorAll('[role=tooltip]')].some((e) => !e.hidden && e.textContent.includes('Play'))";
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{PLAY}') !== null"),
            "the controls",
        )
        .await
        .unwrap();
        e2e::clock::hold(page, &[TOOLTIP_DELAY_MS]).await.unwrap();
        pointer::hover(page, PLAY).await.unwrap();
        e2e::clock::until_armed(page, TOOLTIP_DELAY_MS, 1, "the 2 s hover delay")
            .await
            .unwrap();
        e2e::clock::settle(page).await.unwrap();
        let early: bool = e2e::js(page, TIP).await;
        assert!(!early, "the tooltip opened before its delay ran out");
        e2e::clock::fire(page, TOOLTIP_DELAY_MS).await.unwrap();
        wait::for_js_true(page, TIP, "the tooltip after the delay")
            .await
            .unwrap();
        fixture.console.assert_clean("a control's tooltip").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A media control's tooltip delay, as Video's test holds it.
const TOOLTIP_DELAY_MS: u32 = 2000;

/// WCAG 1.4.10, todos 1325 and 1385: a bubble of at most 22rem whose row stays
/// one line down to 160px (320px at 200% zoom), nothing leaving the player; the
/// time goes at 240px, and from 208px the buttons shrink, down to 24px, rather
/// than the volume going (todo 1393). A shrink-wrapping parent does not collapse it.
#[test]
fn the_controls_fit_a_narrow_player() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        for (width, bubble, shown) in [
            (500, 352, 5),
            (352, 352, 5),
            (320, 320, 5),
            (240, 240, 4),
            (208, 208, 4),
            (160, 160, 4),
        ] {
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
                         .filter((e) => e.getBoundingClientRect().width > 0);
                     const seek = document.querySelector('#player [data-slot=seek]').getBoundingClientRect();
                     const fits = row.every((e) => {{ const r = e.getBoundingClientRect();
                         return r.left >= player.left && r.right <= player.right; }});
                     const lines = new Set(row.map((e) => {{ const r = e.getBoundingClientRect(); return Math.round(r.top + r.height / 2); }})).size;
                     const buttons = [...document.querySelectorAll('#player [data-slot=controls] button')];
                     const volume = document.querySelectorAll('#player [data-slot=volume] button');
                     const sized = buttons.every((b) => b.getBoundingClientRect().height >= 24)
                         && [...volume].every((b) => b.getBoundingClientRect().width > 0);
                     return row.length === {shown} && seek.width >= 32 && player.width === {bubble} && fits && lines === 1 && sized; }})()"
                ),
                &format!("the row to fit {width}px"),
            )
            .await
            .unwrap();
            // Every button's own box or spacing, not the height alone (todo 1823).
            let buttons = "#player [data-slot=controls] button";
            let measured = target_size::measure_all(page, buttons).await.unwrap();
            target_size::assert_sizes(&format!("{buttons} at {width}px"), &measured).unwrap();
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

/// Todo 1377: the bars follow the decoded sound. Silence decodes flat, which
/// the bars drawn from the URL never are.
#[test]
fn the_bars_follow_the_decoded_sound() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        wait::for_js_true(
            &fixture.page,
            "(() => { const bars = [...document.querySelectorAll('#player [data-slot=bars] > span')];
             return bars.length === 28 && bars.every((bar) => bar.style.height === '20%'); })()",
            "every bar of the silent file at its floor",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Shift and the key that types `?` on a German layout, ß's.
const QUESTION_DE: Key = Key {
    key: "?",
    code: "Minus",
    vk: 219,
    text: Some("?"),
};

/// Todo 1410: Shift+? lists the player's keys, by the `?` it types on any layout;
/// no captions or fullscreen rows, as Audio has neither.
#[test]
fn shift_question_lists_the_keys() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{PLAY}') !== null"),
            "the controls",
        )
        .await
        .unwrap();
        page.evaluate(format!("document.querySelector('{PLAY}').focus()"))
            .await
            .unwrap();
        keyboard::press_with(page, QUESTION_DE, keyboard::SHIFT)
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "(() => { const text = document.querySelector('[role=dialog]')?.textContent ?? '';
             return text.includes('Back 10 seconds') && text.includes('Show these shortcuts')
                 && !text.includes('Captions') && !text.includes('Fullscreen'); })()",
            "the help with Audio's rows",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("help").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The broken player's alert grows the page above the players under it; a click aimed before
/// it appears lands on its text (todo 1673).
async fn broken_player_settled(page: &chromiumoxide::Page) {
    wait::for_js_true(
        page,
        "document.querySelector('#broken [role=alert]') !== null",
        "the broken player's alert, which moves the players under it",
    )
    .await
    .unwrap();
}

/// JS: `player`'s Play is disabled, a Tab stop still, and described by its error alert.
pub(crate) fn play_explained(player: &str) -> String {
    format!(
        "(() => {{ const alert = document.querySelector('{player} [role=alert]');
         const play = document.querySelector('{player} [data-slot=controls] button');
         return alert !== null && alert.id !== '' && play.getAttribute('aria-describedby') === alert.id
             && play.getAttribute('aria-disabled') === 'true' && play.tabIndex === 0; }})()"
    )
}

/// Todo 2679: a player mounted with its first `<source>` still to try shows no error alert,
/// not even for a moment, on its way to the source that plays.
#[test]
fn a_healthy_player_mounting_never_shows_the_error() {
    block_on(async {
        let fixture = Fixture::open("/audio/late", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(
            "window.alerted = false;
             new MutationObserver(() => {
                 if (document.querySelector('#late [role=alert]')) window.alerted = true;
             }).observe(document.body, { subtree: true, childList: true })",
        )
        .await
        .unwrap();
        pointer::click(page, "#mount").await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '0:04'", time_text("late")),
            "the late player's 4 s file",
        )
        .await
        .unwrap();
        let alerted: bool = js(page, "window.alerted").await;
        assert!(
            !alerted,
            "the healthy player showed the error alert while mounting"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 2654: every `<source>` failing sets no MediaError, yet the player says it failed.
#[test]
fn every_source_failing_shows_the_error() {
    block_on(async {
        let fixture = Fixture::open("/audio/sources-failing", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#failing [role=alert]') !== null
             && document.querySelector('#failing audio').error === null",
            "the alert for sources that all failed, with no MediaError on the element",
        )
        .await
        .unwrap();
        wait::for_js_true(
            &fixture.page,
            &play_explained("#failing"),
            "Play disabled and described by the alert (todo 2680)",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1385: the speaker mutes and unmutes; the chevron opens the volume
/// slider in a dialog, focused, and Escape returns to the chevron.
#[test]
fn the_speaker_mutes_and_the_volume_sits_in_a_menu() {
    const SPEAKER: &str = "#muted [data-slot=volume] button:not([aria-haspopup])";
    const CHEVRON: &str = "#muted [data-slot=volume] button[aria-haspopup]";
    const DIALOG: &str = "[role=dialog][aria-label=Volume]";
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let label = |selector: &str, name: &str| {
            format!("document.querySelector('{selector}').getAttribute('aria-label') === '{name}'")
        };

        wait::for_js_true(
            page,
            &format!(
                "{} && {}",
                audio("muted", "a.muted"),
                label(SPEAKER, "Unmute")
            ),
            "a muted player to offer Unmute",
        )
        .await
        .unwrap();
        broken_player_settled(page).await;
        pointer::click(page, SPEAKER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{} && {}",
                audio("muted", "!a.muted"),
                label(SPEAKER, "Mute")
            ),
            "the speaker to unmute",
        )
        .await
        .unwrap();
        pointer::click(page, SPEAKER).await.unwrap();
        wait::for_js_true(page, &audio("muted", "a.muted"), "the speaker to mute")
            .await
            .unwrap();

        pointer::click(page, CHEVRON).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.activeElement?.closest('{DIALOG}') !== null
                 && document.activeElement.getAttribute('role') === 'slider'
                 && document.querySelector('{DIALOG} [data-slot=track]').getBoundingClientRect().width >= 100
                 && document.querySelector('{CHEVRON}').getAttribute('aria-expanded') === 'true'"
            ),
            "the volume dialog open, its slider focused",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait::for_js_true(
            page,
            &audio("muted", "!a.muted && Math.abs(a.volume - 0.95) < 1e-6"),
            "a moved volume to unmute",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{DIALOG}') === null
                 && document.activeElement === document.querySelector('{CHEVRON}')"
            ),
            "Escape to close the dialog and return to the chevron",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("muted").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1385: the bars fill as the time moves, from the start side, and a
/// click on them seeks; right to left the start is the right. The slider's own
/// `SliderTrack::Bars` draws them (todo 2050).
#[test]
fn the_bars_follow_the_time_and_a_click_seeks() {
    block_on(async {
        let fixture = Fixture::open("/audio", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        for (id, from_right) in [("player", false), ("rtl", true)] {
            let bars = format!(
                "document.querySelectorAll('#{id} [data-slot=track] > [data-slot=bars] > span')"
            );
            wait::for_js_true(
                page,
                &format!(
                    "{} === '0:04' && {bars}.length === 28
                     && document.querySelectorAll('#{id} [data-slot=bars] > [data-state=filled]').length === 0",
                    time_text(id)
                ),
                "28 bars, none played",
            )
            .await
            .unwrap();
            let track: (f64, f64, f64) = page
                .evaluate(format!(
                    "(() => {{ const r = document.querySelector('#{id} [data-slot=bars]').getBoundingClientRect();
                     return [r.left, r.width, r.top + r.height / 2]; }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            let fraction = if from_right { 0.25 } else { 0.75 };
            let at = pointer::Point {
                x: track.0 + track.1 * fraction,
                y: track.2,
            };
            pointer::click_at(page, at).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "{} && (() => {{ const b = [...{bars}];
                     const played = b.filter((e) => e.dataset.state === 'filled');
                     const first = b[0].getBoundingClientRect(), last = b[27].getBoundingClientRect();
                     return played.length >= 18 && played.length <= 24 && played[0] === b[0]
                         && (first.left > last.left) === {from_right}; }})()",
                    audio(id, "Math.abs(a.currentTime - 3) < 0.5")
                ),
                &format!("a click three quarters in to seek and fill, {id}"),
            )
            .await
            .unwrap();
        }
        fixture.console.assert_clean("bars").unwrap();
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
        // Near the end, not 4 s of real play (todo 1825).
        let _: f64 = e2e::js(page, audio("player", "a.currentTime = a.duration - 0.1")).await;
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

        // Focus is on the play button now: Space presses it, once. Counted at the element:
        // two toggles in one turn both read "paused" and both play (todo 1825).
        let _: bool = e2e::js(
            page,
            audio(
                "player",
                "(window.__calls = [], ['play', 'pause'].forEach((name) => { \
                   a[name] = function () { window.__calls.push(name); \
                     return HTMLMediaElement.prototype[name].call(this); }; }), true)",
            ),
        )
        .await;
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(page, &audio("player", "!a.paused"), "Space to play")
            .await
            .unwrap();
        e2e::clock::settle(page).await.unwrap();
        let (calls, label): (Vec<String>, String) = e2e::js(
            page,
            format!(
                "[window.__calls, document.querySelector('{PLAY}').getAttribute('aria-label')]"
            ),
        )
        .await;
        assert!(
            calls == ["play"] && label == "Pause",
            "Space made the calls {calls:?}, the button offers {label}"
        );

        fixture.console.assert_clean("playing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// L jumps ahead and M mutes, but only with focus inside the player.
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
        keyboard::press(page, M).await.unwrap();
        keyboard::tab_to(page, PLAY, 5).await.unwrap();
        let time: f64 = page
            .evaluate(audio("player", "a.currentTime"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(time, 0.0, "L outside the player jumped");
        let muted: bool = page
            .evaluate(audio("player", "a.muted"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!muted, "M outside the player muted");

        keyboard::press(page, M).await.unwrap();
        // Space on the seek slider plays; the hotkeys stay off Space, which a button keeps.
        page.evaluate("document.querySelector('#player [data-slot=seek] [role=slider]').focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &audio("player", "!a.paused && a.muted"),
            "Space on the slider to play, M to mute",
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
