//! `Video`: fullscreen takes the whole player and gives it back, the captions
//! button shows the subtitle track, and F and C act inside the player.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually};
use e2e::passes::keyboard::{self, Key};
use e2e::passes::pointer;
use e2e::{Fixture, Suite, Viewport, wait};

const PLAY: &str = "#player [data-slot=controls] button";
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
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
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
    const CONTRAST: &str = r#"(() => {
        const rgb = (s) => {
            const m = s.match(/color\(srgb ([\d.]+) ([\d.]+) ([\d.]+)(?: \/ ([\d.]+))?\)/);
            if (m) return [m[1] * 255, m[2] * 255, m[3] * 255, m[4] === undefined ? 1 : +m[4]];
            const n = s.match(/rgba?\(([^)]+)\)/)[1].split(/[ ,\/]+/).map(Number);
            return [n[0], n[1], n[2], n[3] === undefined ? 1 : n[3]];
        };
        const over = (c, base) => [0, 1, 2].map((i) => c[i] * c[3] + base[i] * (1 - c[3]));
        const lum = (c) => {
            const [r, g, b] = c.slice(0, 3).map((v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; });
            return 0.2126 * r + 0.7152 * g + 0.0722 * b;
        };
        const q = (s) => document.querySelector('#player ' + s);
        const style = (s) => getComputedStyle(q(s));
        const bar = style('[data-slot=controls]');
        if (bar.position !== 'absolute') return [0, 0, 0, 0, 0];
        const base = over(rgb(bar.backgroundImage.match(/rgba?\([^)]+\)/)[0]), [255, 255, 255]);
        const ratio = (s) => {
            const [x, y] = [lum(over(rgb(s), base)), lum(base)];
            return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
        };
        return [
            ratio(style('[data-slot=time]').color),
            ratio(style("button[aria-label^='Playback speed']").color),
            ratio(style('[data-slot=controls] button').color),
            ratio(style('[data-slot=seek] [data-slot=track]').backgroundColor),
            ratio(style('[data-slot=seek] [role=slider]').borderTopColor),
        ];
    })()"#;
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
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
            let [time, speed, icon, track, thumb]: [f64; 5] =
                page.evaluate(CONTRAST).await.unwrap().into_value().unwrap();
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

        // Todo 1324: without tracks the button stays, disabled but focusable and explained.
        let plain: bool = page
            .evaluate(
                "(() => { const b = document.querySelector('#plain button[aria-label=Captions]');
                 b.focus(); b.click();
                 return document.activeElement === b && b.getAttribute('aria-disabled') === 'true'
                     && b.getAttribute('aria-pressed') === 'false'
                     && document.getElementById(b.getAttribute('aria-describedby')).textContent === 'No captions for this video'; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            plain,
            "a player without tracks has a disabled, explained captions button"
        );

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

/// Native or pseudo, whichever the renderer grants; Blitz has no controls to press.
async fn fullscreen_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() == Platform::Native {
        return Ok(());
    }
    eventually(d, "the controls", async |d| d.exists(FULLSCREEN).await).await?;
    let filled = async |d: &mut D| {
        d.click(FULLSCREEN).await?;
        eventually(d, "the player to fill the screen", async |d| {
            Ok(d.attr("#player [role=group]", "data-fullscreen")
                .await?
                .is_some())
        })
        .await
    };
    // Android's native fullscreen too, which ignored Escape before 1256.
    filled(d).await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "Escape to give the player back", async |d| {
        Ok(d.attr("#player [role=group]", "data-fullscreen")
            .await?
            .is_none())
    })
    .await?;
    // Back leaves it too, rather than the app (1275).
    if d.platform() == Platform::Android {
        filled(d).await?;
        d.press_back().await?;
        eventually(d, "Back to give the player back", async |d| {
            Ok(d.attr("#player [role=group]", "data-fullscreen")
                .await?
                .is_none())
        })
        .await?;
    }
    filled(d).await?;
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
    within(d, 3, "a press on the picture to show them", controls_shown).await?;
    // A click pauses too; a tap on faded controls only shows them (todo 1362).
    if d.platform() != Platform::Android {
        reads(d, "the click to pause", PLAY, "aria-label", "Play").await?;
        d.click(PLAY).await?;
    }
    reads(d, "playing", PLAY, "aria-label", "Pause").await?;
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

/// Todo 1342: in the page too the bar fades while playing untouched, comes back
/// on a tap, and stays while paused.
async fn inline_controls_fade<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.platform() == Platform::Native {
        return Ok(());
    }
    eventually(d, "the duration", async |d| {
        Ok(d.text("#player [data-slot=time]").await? == "0:00 / 0:15")
    })
    .await?;
    d.click(PLAY).await?;
    within(d, 8, "the controls to fade while playing", faded).await?;
    d.click("#player video").await?;
    within(d, 3, "a press on the picture to show them", controls_shown).await?;
    // A click pauses as well; a tap on faded controls only shows them (todo 1362).
    if d.platform() == Platform::Android {
        d.click(PLAY).await?;
    }
    reads(d, "playing to pause", PLAY, "aria-label", "Play").await?;
    let started = std::time::Instant::now();
    while started.elapsed().as_secs() < 4 {
        anyhow::ensure!(controls_shown(d).await?, "the controls faded while paused");
        d.idle().await;
    }
    Ok(())
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

/// Todo 1324, as the docs demo: a default captions track loads its cues, starts
/// shown, and the button hides and shows it again; pressed, a bar marks its icon (1370).
#[test]
fn a_default_captions_track_loads_and_toggles() {
    block_on(async {
        let fixture = Fixture::open("/video/captions", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let state = |mode: &str, pressed: &str| {
            let mark = if pressed == "true" { "block" } else { "none" };
            format!(
                "(() => {{ const t = document.querySelector('#player video').textTracks[0];
                 const button = document.querySelector('{CAPTIONS}');
                 return t.cues !== null && t.cues.length === 1 && t.mode === '{mode}'
                     && button.getAttribute('aria-pressed') === '{pressed}'
                     && getComputedStyle(button.querySelector('[data-mark]')).display === '{mark}'; }})()"
            )
        };
        wait::for_js_true(page, &state("showing", "true"), "the cues, shown")
            .await
            .unwrap();
        pointer::click(page, CAPTIONS).await.unwrap();
        wait::for_js_true(page, &state("hidden", "false"), "the button to hide them")
            .await
            .unwrap();
        pointer::click(page, CAPTIONS).await.unwrap();
        wait::for_js_true(page, &state("showing", "true"), "the button to show them")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

e2e::scenario!(
    a_tab_out_of_the_drawn_fullscreen_leaves_it,
    "/video/refused",
    tab_out_leaves_pseudo
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

/// Todo 1370: in fullscreen the page's portal is outside the player, which hides
/// it under the native API; the speed menu opens inside the player and works.
#[test]
fn the_speed_menu_works_in_fullscreen() {
    const SPEED: &str = "#player button[aria-label^='Playback speed']";
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{FULLSCREEN}') !== null"),
            "the controls",
        )
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
        pointer::click(page, SPEED).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const item = [...document.querySelectorAll('[role=menuitemradio]')]
                 .find((e) => e.textContent.includes('1.5×'));
             if (!item || !document.querySelector('#player [role=group]').contains(item)) return false;
             const r = item.getBoundingClientRect();
             return item.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)); })()",
            "the menu inside the player, on top",
        )
        .await
        .unwrap();
        page.evaluate(
            "[...document.querySelectorAll('[role=menuitemradio]')].find((e) => e.textContent.includes('1.5×')).dataset.pick = ''",
        )
        .await
        .unwrap();
        pointer::click(page, "[data-pick]").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('#player video').playbackRate === 1.5
                 && document.querySelector(\"{SPEED}\").textContent.includes('1.5×')"
            ),
            "the rate at 1.5×",
        )
        .await
        .unwrap();
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
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
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

/// The cue box Chromium draws in the `<video>`'s own shadow tree, by its bottom edge.
async fn cue_bottom(page: &chromiumoxide::Page) -> Option<f64> {
    use chromiumoxide::cdp::browser_protocol::dom::{GetBoxModelParams, GetDocumentParams, Node};
    fn find(node: &Node) -> Option<&Node> {
        let attrs = node.attributes.as_deref().unwrap_or_default();
        if attrs
            .chunks(2)
            .any(|pair| pair[0] == "pseudo" && pair[1] == "-webkit-media-text-track-display")
        {
            return Some(node);
        }
        [&node.children, &node.shadow_roots, &node.pseudo_elements]
            .into_iter()
            .flatten()
            .flatten()
            .find_map(find)
    }
    let document = page
        .execute(GetDocumentParams::builder().depth(-1).pierce(true).build())
        .await
        .ok()?;
    let cue = find(&document.result.root)?.backend_node_id;
    let model = page
        .execute(GetBoxModelParams::builder().backend_node_id(cue).build())
        .await
        .ok()?;
    // Corners clockwise from the top left: the bottom edge's y is the sixth number.
    model.result.model.border.inner().get(5).copied()
}

/// Todos 1371 and 1389: the shown captions sit above the bar, not under it, at xl too.
#[test]
fn the_captions_clear_the_bar() {
    block_on(async {
        for route in ["/video/captions", "/video/captions/xl"] {
            captions_clear_the_bar(route).await;
        }
    });
}

async fn captions_clear_the_bar(route: &str) {
    let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
    let page = &fixture.page;
    wait::for_js_true(
        page,
        "document.querySelector('#player video').textTracks[0].cues?.length === 1",
        "the cues",
    )
    .await
    .unwrap();
    pointer::click(page, PLAY).await.unwrap();
    let seek_top: f64 = page
        .evaluate("document.querySelector('#player [data-slot=seek]').getBoundingClientRect().top")
        .await
        .unwrap()
        .into_value()
        .unwrap();
    let started = std::time::Instant::now();
    loop {
        let bottom = cue_bottom(page).await;
        if bottom.is_some_and(|bottom| bottom <= seek_top) {
            break;
        }
        assert!(
            started.elapsed().as_secs() < 5,
            "{route}: the cue ends at {bottom:?}, below the seek row's top {seek_top}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    fixture.close().await.unwrap();
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

/// Polls `js` on `page` for `secs`, past `wait`'s budget: the controls fade after 3 s.
async fn until(page: &chromiumoxide::Page, js: &str, secs: u64, what: &str) {
    let started = std::time::Instant::now();
    loop {
        let done: bool = page
            .evaluate(js)
            .await
            .ok()
            .and_then(|value| value.into_value().ok())
            .unwrap_or(false);
        if done {
            return;
        }
        assert!(
            started.elapsed().as_secs() < secs,
            "gave up waiting for {what}"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

const FADED: &str = "document.querySelector('#player [role=group]').dataset.controls === 'hidden'
    && getComputedStyle(document.querySelector('#player [data-slot=controls]')).opacity === '0'";
const PLAYING: &str = "!document.querySelector('#player video').paused";
const PAUSED: &str = "document.querySelector('#player video').paused";
const LONG_DURATION: &str =
    "document.querySelector('#player [data-slot=time]').textContent === '0:00 / 0:15'";

/// Todo 1349: in fullscreen a control's tooltip opens inside the player, on top.
#[test]
fn a_tooltip_shows_in_fullscreen() {
    block_on(async {
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector('{FULLSCREEN}') !== null"),
            "the controls",
        )
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
        pointer::hover(page, PLAY).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const tip = [...document.querySelectorAll('[role=tooltip]')].find((e) => e.textContent.includes('Play'));
             if (!tip || !document.querySelector('#player [role=group]').contains(tip)) return false;
             const r = tip.getBoundingClientRect();
             return r.width > 0 && tip.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)); })()",
            "the tooltip inside the player, on top",
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
        let start = std::time::Instant::now();
        pointer::hover(page, PLAY).await.unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let early: bool = page.evaluate(TIP).await.unwrap().into_value().unwrap();
        assert!(!early, "the tooltip opened within a second");
        wait::for_js_true(page, TIP, "the tooltip after the delay")
            .await
            .unwrap();
        let waited = start.elapsed().as_millis();
        assert!(waited >= 1900, "opened after {waited}ms");
        fixture.close().await.unwrap();
    });
}

/// Todo 1362: a click on the picture plays and pauses; a tap on faded controls only
/// shows them, and once shown a tap pauses.
#[test]
fn a_click_on_the_picture_plays_or_pauses() {
    block_on(async {
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
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
        until(page, FADED, 8, "the controls to fade").await;
        let at = pointer::centre_of(page, "#player video").await.unwrap();
        pointer::touch_drag(page, at, at, 0).await.unwrap();
        until(
            page,
            &format!("!({FADED}) && {PLAYING}"),
            3,
            "a tap to show the controls and keep playing",
        )
        .await;
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
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, LONG_DURATION, "the duration")
            .await
            .unwrap();
        pointer::click(page, PLAY).await.unwrap();
        until(page, FADED, 8, "the controls to fade").await;
        page.evaluate(format!("document.querySelector('{FULLSCREEN}').focus()"))
            .await
            .unwrap();
        until(
            page,
            "document.querySelector('#player [role=group]').dataset.controls === undefined
             && getComputedStyle(document.querySelector('#player [data-slot=controls]')).opacity === '1'",
            3,
            "focus from code to show the row",
        )
        .await;
        // As after a key, they stay while focus is in them.
        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        until(page, &format!("!({FADED})"), 1, "the row to stay").await;
        fixture.close().await.unwrap();
    });
}

/// Todo 1418: a press the browser cancels ends too, so focus from code shows the row after it.
#[test]
fn a_cancelled_press_does_not_stop_focus_showing_the_controls() {
    block_on(async {
        let fixture = Fixture::open("/video/long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
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
        until(page, FADED, 8, "the controls to fade").await;
        page.evaluate(format!("document.querySelector('{FULLSCREEN}').focus()"))
            .await
            .unwrap();
        until(
            page,
            &format!("!({FADED})"),
            3,
            "focus from code to show the row",
        )
        .await;
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
        let fixture = Fixture::open("/video", Viewport::Desktop).await.unwrap();
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
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
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
