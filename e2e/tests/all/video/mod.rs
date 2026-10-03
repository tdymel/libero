//! `Video`: the controls, their order, menus and keys. Fullscreen, the fading
//! controls and the captions have modules of their own.

mod captions;
mod fade;
mod fullscreen;

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::contrast::COLOUR_JS;
use e2e::passes::keyboard::{self, Key};
use e2e::passes::pointer;
use e2e::{Fixture, Suite, Viewport, clock, wait};

const PLAY: &str = "#player [data-slot=controls] button";
const FULLSCREEN: &str = "#player button[aria-label=Fullscreen]";
const CAPTIONS: &str = "#player button[aria-label=Captions]";
const STATE: &str = "(document.querySelector('#player [role=group]').dataset.fullscreen ?? 'none')";

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

/// Every shown button and the time of `#player`'s bar lie inside the player, none
/// above its top; `one_row`: all on one line.
fn controls_fit(width: u32, one_row: bool) -> String {
    format!(
        "(() => {{ const player = document.querySelector('#player [role=group]').getBoundingClientRect();
         const shown = [...document.querySelectorAll('#player [data-slot=controls] > :not([data-slot=seek])')]
             .filter((e) => getComputedStyle(e).display !== 'none' && e.getBoundingClientRect().width > 0);
         const inside = shown.every((e) => {{ const r = e.getBoundingClientRect();
             return r.left >= player.left && r.right <= player.right && r.top >= player.top; }});
         const tops = new Set(shown.map((e) => Math.round(e.getBoundingClientRect().top + e.getBoundingClientRect().height / 2)));
         return player.width === {width} && inside && (!{one_row} || tops.size === 1); }})()"
    )
}

/// WCAG 1.4.10, todos 1325, 1342 and 1399: the seek track has a row of its own;
/// the buttons, mute and the volume menu's chevron included, stay one line at a
/// phone's 412px and at 320px, the total time giving way; at 160px (320px at 200%
/// zoom) the bar moves below the picture and wraps, but no control leaves the player.
#[test]
fn the_controls_fit_a_narrow_player() {
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        for (width, volume, total, one_row) in [
            (512, "flex", "inline", true),
            (412, "flex", "inline", true),
            (320, "flex", "none", true),
            (160, "flex", "none", false),
        ] {
            page.evaluate(format!(
                "document.querySelector('#player').style.width = '{width}px'"
            ))
            .await
            .unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const style = (s) => getComputedStyle(document.querySelector('#player ' + s)).display;
                     return {} && style('[data-slot=volume]') === '{volume}'
                         && style('[data-slot=time] > span') === '{total}'; }})()",
                    controls_fit(width, one_row)
                ),
                &format!("the row to fit {width}px"),
            )
            .await
            .unwrap();
        }
        fixture.close().await.unwrap();
    });
}

/// Todos 1328 and 1342: every control is its own Tab stop, in the order it is
/// drawn: the seek row, then YouTube's order of the buttons.
#[test]
fn every_control_is_a_tab_stop_in_visual_order() {
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{FULLSCREEN}') !== null"),
            "the controls",
        )
        .await
        .unwrap();
        page.evaluate("document.querySelector('#player [data-slot=seek] [role=slider]').focus()")
            .await
            .unwrap();
        let mut names = Vec::new();
        for _ in 0..7 {
            let name: String = page
                .evaluate("document.activeElement.getAttribute('aria-label')")
                .await
                .unwrap()
                .into_value()
                .unwrap();
            names.push(name);
            keyboard::press(page, keyboard::TAB).await.unwrap();
        }
        assert_eq!(
            names,
            [
                "Seek",
                "Play",
                "Mute",
                "Volume",
                "Captions",
                "Playback speed 1×",
                "Fullscreen"
            ]
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 1323: the speed menu sets the element's rate, and the button follows it.
#[test]
fn the_speed_menu_sets_the_rate() {
    const SPEED: &str = "#player button[aria-label^='Playback speed']";
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector(\"{SPEED}\") !== null"),
            "the speed button",
        )
        .await
        .unwrap();
        pointer::click(page, SPEED).await.unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('[role=menuitemradio]')].some((e) => e.textContent.includes('1.5×'))",
            "the speed menu",
        )
        .await
        .unwrap();
        // Todo 1388: open, the white "1×" sits on a faint white tint over the scrim, not the theme's light one.
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const s = getComputedStyle(document.querySelector(\"{SPEED}\"));
                 return s.color === 'rgb(255, 255, 255)' && s.backgroundColor === 'rgba(255, 255, 255, 0.2)'; }})()"
            ),
            "the open speed button on the scrim tint",
        )
        .await
        .unwrap();
        page.evaluate(
            "[...document.querySelectorAll('[role=menuitemradio]')].find((e) => e.textContent.includes('1.5×')).click()",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('#player video').playbackRate === 1.5
                 && document.querySelector(\"{SPEED}\").getAttribute('aria-label') === 'Playback speed 1.5×'
                 && document.querySelector(\"{SPEED}\").textContent.includes('1.5×')"
            ),
            "the rate and the button at 1.5×",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.3 and 1.4.11, todo 1342: the bar sits over the picture on a scrim
/// whose floor, even over a white frame, keeps its text 4.5:1 and its icons,
/// tracks and thumb 3:1, in light and dark.
#[test]
fn the_overlay_bar_keeps_its_contrast_over_a_white_picture() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    let contrast = format!(
        r#"(() => {{ {COLOUR_JS}
        const q = (s) => document.querySelector('#player ' + s);
        const style = (s) => getComputedStyle(q(s));
        const bar = style('[data-slot=controls]');
        if (bar.position !== 'absolute') return [0, 0, 0, 0, 0];
        const base = OVER(RGBA(bar.backgroundImage.match(/rgba?\([^)]+\)/)[0]), [255, 255, 255, 1]);
        const ratio = (s) => CONTRAST(OVER(RGBA(s), base), base);
        return [
            ratio(style('[data-slot=time]').color),
            ratio(style("button[aria-label^='Playback speed']").color),
            ratio(style('[data-slot=controls] button').color),
            ratio(style('[data-slot=seek] [data-slot=track]').backgroundColor),
            ratio(style('[data-slot=seek] [role=slider]').borderTopColor),
        ];
    }})()"#
    );
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{FULLSCREEN}') !== null"),
            "the controls",
        )
        .await
        .unwrap();
        for scheme in ["light", "dark"] {
            page.execute(
                SetEmulatedMediaParams::builder()
                    .features(vec![MediaFeature::new("prefers-color-scheme", scheme)])
                    .build(),
            )
            .await
            .unwrap();
            let [time, speed, icon, track, thumb]: [f64; 5] = page
                .evaluate(contrast.as_str())
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(time >= 4.5, "{scheme}: the time is {time:.2}:1");
            assert!(speed >= 4.5, "{scheme}: the speed is {speed:.2}:1");
            assert!(icon >= 3.0, "{scheme}: the icons are {icon:.2}:1");
            assert!(track >= 3.0, "{scheme}: the seek track is {track:.2}:1");
            assert!(thumb >= 3.0, "{scheme}: the thumb outline is {thumb:.2}:1");
        }
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.11: forced colours paint the black letterbox as Canvas, so the
/// picture's box needs a line of its own.
#[test]
fn the_picture_has_an_edge_under_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
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
            "getComputedStyle(document.querySelector('#player video')).outlineStyle === 'solid'",
            "an outline on the picture",
        )
        .await
        .unwrap();
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
                && !d.exists("#player [data-slot=controls]").await?)
        })
        .await?;
        return Ok(());
    }
    // Video's "0:00 / 0:04", Audio's "0:04".
    eventually(d, "the duration", async |d| {
        Ok(d.text("#player [data-slot=time]").await?.ends_with("0:04"))
    })
    .await?;
    let start = d.text("#player [data-slot=time]").await?;
    d.click(PLAY).await?;
    reads(d, "the button to offer Pause", PLAY, "aria-label", "Pause").await?;
    // Each `timeupdate` that passes the throttle moves the time on.
    eventually(d, "the time to move", async |d| {
        let time = d.text("#player [data-slot=time]").await?;
        Ok(time != start)
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

/// Todo 1369: a centring flex row shrink-wraps its child, as the docs demo's
/// preview; the player still takes the row's width, or 40rem in a wrapper the
/// row sizes, keeps its ratio, and its buttons stay one line. Without
/// `aspect_ratio` the box is 16:9 too: with the metadata loaded, before any
/// (`preload: None`), and with a failing source (1386), so it never jumps.
#[test]
fn a_shrink_wrapping_parent_keeps_the_box() {
    block_on(async {
        let fixture = Fixture::open("/video/centered", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelector('#unsized [data-slot=time]').textContent === '0:00 / 0:04'",
            "the unsized player's metadata",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#failing [role=alert]') !== null",
            "the failing source's alert",
        )
        .await
        .unwrap();
        for (row, width) in [
            ("#row", 512),
            ("#wrapped", 640),
            ("#unsized", 512),
            ("#waiting", 512),
            ("#failing", 512),
        ] {
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const player = document.querySelector('{row} [role=group]');
                     const video = player.querySelector('video').getBoundingClientRect();
                     const tops = new Set([...player.querySelectorAll('[data-slot=controls] button')]
                         .map((b) => Math.round(b.getBoundingClientRect().top)));
                     return player.getBoundingClientRect().width === {width}
                         && Math.round(video.height) === Math.round({width} * 9 / 16) && tops.size === 1; }})()"
                ),
                &format!("{row}'s player {width}px wide, 16:9, one row of buttons"),
            )
            .await
            .unwrap();
        }
        fixture.close().await.unwrap();
    });
}

/// Todo 1399: the speaker mutes and unmutes; the chevron opens the volume slider
/// in a dialog, inside the player in fullscreen, focused, and a moved volume unmutes.
#[test]
fn the_speaker_mutes_and_the_volume_sits_in_a_menu() {
    const SPEAKER: &str = "#player [data-slot=volume] button:not([aria-haspopup])";
    const CHEVRON: &str = "#player [data-slot=volume] button[aria-haspopup]";
    const DIALOG: &str = "[role=dialog][aria-label=Volume]";
    const VIDEO: &str = "document.querySelector('#player video')";
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;
        let speaker = |name: &str| {
            format!("document.querySelector('{SPEAKER}')?.getAttribute('aria-label') === '{name}'")
        };
        wait::for_js_true(page, &speaker("Mute"), "the speaker")
            .await
            .unwrap();
        pointer::click(page, SPEAKER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{VIDEO}.muted && {}", speaker("Unmute")),
            "the speaker to mute",
        )
        .await
        .unwrap();
        pointer::click(page, SPEAKER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!{VIDEO}.muted && {}", speaker("Mute")),
            "the speaker to unmute",
        )
        .await
        .unwrap();
        pointer::click(page, SPEAKER).await.unwrap();
        wait::for_js_true(page, &format!("{VIDEO}.muted"), "the speaker to mute again")
            .await
            .unwrap();

        pointer::click(page, FULLSCREEN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{STATE} !== 'none'"),
            "the player to fill the screen",
        )
        .await
        .unwrap();
        pointer::click(page, CHEVRON).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const dialog = document.querySelector('{DIALOG}');
                 if (!dialog || !document.querySelector('#player [role=group]').contains(dialog)) return false;
                 const r = dialog.getBoundingClientRect();
                 return dialog.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2))
                     && document.activeElement?.closest('{DIALOG}') !== null
                     && document.activeElement.getAttribute('role') === 'slider'
                     && document.querySelector('{CHEVRON}').getAttribute('aria-expanded') === 'true'; }})()"
            ),
            "the volume dialog inside the player, on top, its slider focused",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!{VIDEO}.muted && Math.abs({VIDEO}.volume - 0.95) < 1e-6"),
            "a moved volume to unmute",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{DIALOG}') === null
                 && document.activeElement === document.querySelector('{CHEVRON}')
                 && {STATE} !== 'none'"
            ),
            "Escape to close the dialog only and return to the chevron",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1426: at xs the speaker and the volume chevron each own 24px or more of the row.
#[test]
fn at_xs_the_speaker_and_the_chevron_each_own_a_24px_target() {
    block_on(async {
        let fixture = Fixture::open("/video/captions/xs", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let owned =
            "(() => { const b = l => document.querySelector(`#player button[aria-label=${l}]`);
            const mute = b('Mute'), chevron = b('Volume');
            if (!mute || !chevron) return null;
            const m = mute.getBoundingClientRect(), c = chevron.getBoundingClientRect();
            const y = m.top + m.height / 2, own = { Mute: 0, Volume: 0 };
            for (let x = m.left - 12; x <= c.right + 12; x += 0.5) {
                const hit = document.elementFromPoint(x, y)?.closest('button');
                if (hit === mute) own.Mute += 0.5;
                if (hit === chevron) own.Volume += 0.5;
            }
            return JSON.stringify(own); })()";
        wait::for_js_true(page, &format!("{owned} !== null"), "the volume buttons")
            .await
            .unwrap();
        let measured: String = page.evaluate(owned).await.unwrap().into_value().unwrap();
        let own: serde_json::Value = serde_json::from_str(&measured).unwrap();
        for button in ["Mute", "Volume"] {
            assert!(own[button].as_f64().unwrap() >= 24.0, "{measured}");
        }
        fixture.close().await.unwrap();
    });
}

/// Todo 1371: a press on the seek track jumps there, without a drag.
#[test]
fn a_press_on_the_seek_track_seeks() {
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
        let track: pointer::Point = page
            .evaluate(
                "(() => { const r = document.querySelector('#player [data-slot=seek] [data-slot=track]').getBoundingClientRect();
                 return { x: r.x + r.width * 0.75, y: r.y + r.height / 2 }; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        pointer::click_at(page, track).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#player video').currentTime === 3",
            "the press to seek to 3 s",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1439: a control's tooltip opens only after 2s of hover.
#[test]
fn a_control_tooltip_waits_two_seconds_on_hover() {
    const TIP: &str = "[...document.querySelectorAll('[role=tooltip]')].some((e) => !e.hidden && e.textContent.includes('Play'))";
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{PLAY}') !== null"),
            "the controls",
        )
        .await
        .unwrap();
        clock::hold(page, &[2000]).await.unwrap();
        pointer::hover(page, PLAY).await.unwrap();
        // The tooltip waits on one 2 s timer, and not before it fires.
        clock::until_armed(page, 2000, 1, "the 2 s open timer")
            .await
            .unwrap();
        clock::settle(page).await.unwrap();
        let early: bool = page.evaluate(TIP).await.unwrap().into_value().unwrap();
        assert!(!early, "the tooltip opened before its 2 s timer");
        clock::fire(page, 2000).await.unwrap();
        wait::for_js_true(page, TIP, "the tooltip after the delay")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 1379 and 1378: `muted` mutes the element; the speed reads "1×" in a right-to-left page.
#[test]
fn muted_mutes_and_the_speed_reads_left_to_right() {
    block_on(async {
        let fixture = Fixture::open("/video/variants", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelector('#muted video').muted
             && document.querySelector('#muted [data-slot=volume] button').getAttribute('aria-label') === 'Unmute'",
            "the muted player to be muted and offer Unmute",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "(() => { const b = document.querySelector(\"#rtl button[aria-label^='Playback speed']\");
             return b !== null && getComputedStyle(b).direction === 'ltr'
                 && getComputedStyle(b.closest('#rtl')).direction === 'rtl'; })()",
            "the speed button left to right in a right-to-left page",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Shift and the key that types `?` on a US layout.
const QUESTION: Key = Key {
    key: "?",
    code: "Slash",
    vk: 191,
    text: Some("?"),
};

/// Todo 1410: Shift+? lists the player's keys in a `ShortcutHelp`, only with
/// focus inside it; in fullscreen the dialog shows inside the player.
#[test]
fn shift_question_lists_the_keys_inside_the_player() {
    const DIALOG: &str = "[role=dialog]";
    block_on(async {
        let _fullscreen;
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        _fullscreen = e2e::frames::keep_fullscreen().await;
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{FULLSCREEN}') !== null"),
            "the controls",
        )
        .await
        .unwrap();

        page.evaluate("document.activeElement?.blur()")
            .await
            .unwrap();
        keyboard::press_with(page, QUESTION, keyboard::SHIFT)
            .await
            .unwrap();
        clock::settle(page).await.unwrap();
        let stray: bool = page
            .evaluate(format!("document.querySelector('{DIALOG}') !== null"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!stray, "Shift+? outside the player opened the help");

        page.evaluate(format!("document.querySelector('{PLAY}').focus()"))
            .await
            .unwrap();
        keyboard::press_with(page, QUESTION, keyboard::SHIFT)
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const d = document.querySelector('{DIALOG}'); const text = d?.textContent ?? '';
                 return text.includes('Keyboard shortcuts') && text.includes('Play or pause')
                     && text.includes('Captions on or off') && text.includes('Fullscreen on or off'); }})()"
            ),
            "the help with the player's rows",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('{DIALOG}') === null
                 && document.activeElement === document.querySelector('{PLAY}')"
            ),
            "Escape to close the help and return to play",
        )
        .await
        .unwrap();

        pointer::click(page, FULLSCREEN).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#player [role=group]').hasAttribute('data-fullscreen')",
            "fullscreen",
        )
        .await
        .unwrap();
        keyboard::press_with(page, QUESTION, keyboard::SHIFT)
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('#player [role=group] {DIALOG}')?.textContent.includes('Play or pause') === true"
            ),
            "the help inside the fullscreen player",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("help").unwrap();
        fixture.close().await.unwrap();
    });
}
