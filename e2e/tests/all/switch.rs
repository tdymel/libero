//! `Switch`: a hidden checkbox with `role="switch"` in the field slots.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
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

/// WCAG 1.4.11: an off track parts from the page and the thumb from the track at 3:1. The
/// off track is `muted.6` (490); `muted.3` was 1.3:1.
#[test]
fn an_off_switch_parts_from_the_page() {
    crate::boundary::assert_boundaries(
        "/switch",
        "const track = document.querySelector('#aria ~ [aria-hidden]');
         const bg = el => CSS(el, 'backgroundColor');
         return [
             ['off track on the page', RATIO(bg(track), PAGE(track))],
             ['off thumb on the track', RATIO(bg(track.firstElementChild), bg(track))],
         ];",
    );
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

/// `data-<name>` of `selector` reads `expected`, eventually.
async fn reads<D: Driver>(d: &mut D, selector: &str, name: &str, expected: &str) -> Result<()> {
    eventually(d, &format!("{selector} {name}={expected}"), async |d| {
        Ok(d.attr(selector, &format!("data-{name}")).await?.as_deref() == Some(expected))
    })
    .await
}

/// Space, Enter outside a `Form` (todo 648), the label and the track each
/// toggle it once. The browser's own flip is cancelled, so each is one change.
async fn each_toggles_once<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#plain").await?;
    d.press(keyboard::SPACE).await?;
    reads(d, "#changes", "changes", "1").await?;
    reads(d, "#changes", "on", "true").await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#changes", "changes", "2").await?;
    reads(d, "#changes", "on", "false").await?;
    ensure!(
        d.is_focused("#plain").await?,
        "Enter moved focus off the switch"
    );
    d.click("label[for=plain]").await?;
    reads(d, "#changes", "changes", "3").await?;
    d.click("#plain ~ [aria-hidden]").await?;
    reads(d, "#changes", "changes", "4").await?;
    reads(d, "#changes", "on", "false").await
}

e2e::scenario!(
    space_enter_the_label_and_the_track_each_toggle_it_once,
    "/switch",
    each_toggles_once
);

/// Todo 508: Enter submits the form around the switch, as it does around a
/// native checkbox, and leaves it as it was; Space still toggles.
async fn enter_submits_and_space_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#alerts").await?;
    d.press(keyboard::SPACE).await?;
    reads(d, "#submits", "on", "true").await?;
    d.press(keyboard::ENTER).await?;
    reads(d, "#submits", "submits", "1").await?;
    reads(d, "#submits", "on", "true").await
}

e2e::scenario!(
    enter_submits_the_form_and_space_toggles,
    "/switch/form",
    enter_submits_and_space_toggles
);

/// Todo 660: a raw `<form>` is a form too - read off the DOM, not libero's
/// `Form` context - so Enter submits it there as well.
#[test]
fn enter_submits_a_raw_form() {
    block_on(async {
        let fixture = Fixture::open("/switch/raw-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let submits = "document.querySelector('#submits').dataset.submits";

        keyboard::tab_to(page, "#alerts", 10).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(page, &checked("alerts"), "Space to turn it on")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(page, &format!("{submits} === '1'"), "Enter to submit")
            .await
            .unwrap();
        let on: bool = page
            .evaluate(checked("alerts"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(on, "Enter toggled the switch");

        fixture.console.assert_clean("Enter in a raw form").unwrap();
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
async fn a_track_click_focuses<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#plain ~ [aria-hidden]").await?;
    reads(d, "#changes", "on", "true").await?;
    eventually_focused(d, "#plain", "the track click").await
}

e2e::scenario!(
    a_track_click_focuses_the_switch,
    "/switch",
    a_track_click_focuses
);

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
