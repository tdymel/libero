//! `Switch`: a hidden checkbox with `role="switch"` in the field slots.

use e2e::browser::{Scheme, block_on, emulate_media};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("switch", "/switch")
        .focusable("#plain")
        .focusable("#error")
        .focusable("#aria")
        .focusable("#readonly")
        .focusable("#card")
        // The 22px-tall track is the whole target of `aria_label`-only "Wi-Fi".
        // The control, not the track, so its own hidden input is no neighbour.
        .targets_spaced("span:has(> #aria)")
        .run();
}

/// Contrast of two computed colours, both opaque.
const RATIO: &str = "((a, b) => {
    const lum = v => {
        const [r, g, b] = v.match(/[\\d.]+/g).slice(0, 3).map(Number).map(c => c / 255)
            .map(c => c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
    };
    const [x, y] = [lum(a), lum(b)];
    return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
})";

/// WCAG 1.4.11: an off switch is its track and thumb alone, so the track must
/// part from the page and the thumb from the track at 3:1. The off track is
/// `muted.3`, 1.3:1 on a white page; `muted.6` reaches 3.3:1.
#[test]
#[ignore = "todo 490: the off-state colour is the Maintainer's call"]
fn an_off_switch_parts_from_the_page() {
    block_on(async {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open("/switch", Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            emulate_media(page, scheme, None).await.unwrap();
            let pairs = format!(
                "(() => {{
                    const track = document.querySelector('#aria ~ [aria-hidden]');
                    let page = track.parentElement;
                    while (getComputedStyle(page).backgroundColor === 'rgba(0, 0, 0, 0)') page = page.parentElement;
                    const [t, h, p] = [track, track.firstElementChild, page]
                        .map(el => getComputedStyle(el).backgroundColor);
                    return [{RATIO}(t, p), {RATIO}(h, t)];
                }})()"
            );
            // The scheme flips through a media query; wait for the page to follow.
            wait::for_js_true(
                page,
                &format!("(() => {{ const r = {pairs}; if (JSON.stringify(r) !== window.__last) {{ window.__last = JSON.stringify(r); return false; }} return true; }})()"),
                "settled colours",
            )
            .await
            .unwrap();
            let [track, thumb]: [f64; 2] =
                page.evaluate(pairs).await.unwrap().into_value().unwrap();
            eprintln!(
                "{}: track {track:.2}:1 on the page, thumb {thumb:.2}:1 on the track",
                scheme.name()
            );
            assert!(
                track >= 3.0,
                "{}: the off track reads {track:.2}:1 on the page",
                scheme.name()
            );
            assert!(
                thumb >= 3.0,
                "{}: the off thumb reads {thumb:.2}:1 on its track",
                scheme.name()
            );
            fixture.close().await.unwrap();
        }
    });
}

/// Forced colours drop every background, so the track and thumb need an
/// outline the palette paints, or an off switch is invisible (todo 493).
#[test]
fn track_and_thumb_keep_an_outline_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/switch", Viewport::Desktop).await.unwrap();
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
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        let bare: Vec<String> = page
            .evaluate(
                "(() => {
                    const track = document.querySelector('#plain ~ [aria-hidden]');
                    return [['track', track], ['thumb', track.firstElementChild]]
                        .filter(([, el]) => {
                            const s = getComputedStyle(el);
                            return s.outlineStyle === 'none' || parseFloat(s.outlineWidth) < 1
                                || s.outlineColor === 'rgba(0, 0, 0, 0)';
                        })
                        .map(([name]) => name);
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(bare.is_empty(), "no forced-colours outline on: {bare:?}");
        fixture.close().await.unwrap();
    });
}

fn changes() -> &'static str {
    "document.querySelector('#changes').dataset.changes"
}

fn checked(id: &str) -> String {
    format!("document.getElementById('{id}').checked")
}

/// APG's switch takes Space and Enter; the label and the track each toggle
/// it once. The browser's own flip is cancelled, so each press is one change.
#[test]
fn space_enter_the_label_and_the_track_each_toggle_it_once() {
    block_on(async {
        let fixture = Fixture::open("/switch", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#plain", 10).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(page, &checked("plain"), "Space to turn it on")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!{}", checked("plain")),
            "Enter to turn it off",
        )
        .await
        .unwrap();
        pointer::click(page, "label[for=plain]").await.unwrap();
        wait::for_js_true(page, &checked("plain"), "the label click")
            .await
            .unwrap();
        pointer::click(page, "#plain ~ [aria-hidden]")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{} === '4'", changes()), "the track click")
            .await
            .unwrap();
        let on: bool = page
            .evaluate(checked("plain"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!on, "four toggles left the switch on");

        fixture.console.assert_clean("toggling the switch").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Readonly stays focusable but refuses every activation. The plain switch's
/// Space afterwards is the control: once it lands, the readonly presses have.
#[test]
fn a_readonly_switch_refuses_keys_and_clicks() {
    block_on(async {
        let fixture = Fixture::open("/switch", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#readonly", 10).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        pointer::click(page, "label[for=readonly]").await.unwrap();
        pointer::click(page, "#readonly ~ [aria-hidden]")
            .await
            .unwrap();

        keyboard::tab_to(page, "#plain", 10).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} !== '0'", changes()),
            "the plain switch's press",
        )
        .await
        .unwrap();
        let (count, on): (String, bool) = page
            .evaluate(format!("[{}, {}]", changes(), checked("readonly")))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(count, "1", "the readonly switch called onchange");
        assert!(on, "the readonly switch moved");

        fixture.close().await.unwrap();
    });
}

/// A card is its own hit area: a click on its description toggles the switch.
#[test]
fn a_card_toggles_from_anywhere_on_it() {
    block_on(async {
        let fixture = Fixture::open("/switch", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::click(page, "#card-description").await.unwrap();
        wait::for_js_true(page, &checked("card"), "the card click")
            .await
            .unwrap();

        fixture.console.assert_clean("clicking the card").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A track click focuses the input as a native click would, so its blur
/// shows the rules.
#[test]
fn a_track_click_focuses_the_switch() {
    block_on(async {
        let fixture = Fixture::open("/switch", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::click(page, "#plain ~ [aria-hidden]")
            .await
            .unwrap();
        wait::for_js_true(page, &checked("plain"), "the track click")
            .await
            .unwrap();
        let focused: String = page
            .evaluate("document.activeElement.id")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(focused, "plain", "the track click left focus elsewhere");
        fixture.close().await.unwrap();
    });
}

/// The track shows `not-allowed` like the rest of a disabled switch.
#[test]
fn a_disabled_track_shows_it_takes_no_click() {
    block_on(async {
        let fixture = Fixture::open("/switch", Viewport::Desktop).await.unwrap();
        let cursors: [String; 2] = fixture
            .page
            .evaluate(
                "['#plain', '#disabled'].map(id => \
                 getComputedStyle(document.querySelector(`${id} ~ [aria-hidden]`)).cursor)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(cursors, ["pointer", "not-allowed"]);
        fixture.close().await.unwrap();
    });
}
