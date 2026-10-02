//! `ActionIcon`: an icon-only `<button>`, named by its required `aria_label`,
//! as a toggle, busy and disabled.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

/// The 16px and 20px sizes take presses in an invisible 24x24 box.
#[test]
fn it_meets_the_baseline() {
    Suite::new("action_icon", "/action-icon")
        .focusable("#plain")
        .focusable("#toggle")
        .focusable("#loading")
        .targets("#plain")
        .targets("#toggle")
        .targets("#sm")
        .targets("#xs")
        .run();
}

const CLICKS: &str = "document.getElementById('clicks').textContent";

/// Todo 1094: an `icon` glyph is sized and filled like a child svg.
#[test]
fn an_icon_glyph_is_sized_and_coloured_like_a_child_svg() {
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        let look = |id: &str| {
            format!(
                "(() => {{ const svg = document.querySelector('#{id} > svg'); \
                 const r = svg.getBoundingClientRect(); \
                 return [r.width, r.height, getComputedStyle(svg.querySelector('path')).fill].join(); }})()"
            )
        };
        let read = |id: &'static str| {
            let page = &fixture.page;
            let script = look(id);
            async move {
                let value: String = page.evaluate(script).await.unwrap().into_value().unwrap();
                value
            }
        };
        assert_eq!(read("data").await, read("plain").await);

        fixture.close().await.unwrap();
    });
}

/// A busy icon keeps its tab stop, and neither Enter nor a click runs it.
#[test]
fn a_loading_icon_stays_focusable_and_swallows_presses() {
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#loading", 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        pointer::click(page, "#loading").await.unwrap();
        // The plain icon counts too, so a count of 1 proves the busy one ran nothing.
        pointer::click(page, "#plain").await.unwrap();
        wait::for_js_true(page, &format!("{CLICKS} === '1'"), "the plain icon's click")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("pressing a busy icon")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1559: Android's WebView sends a tap on a disabled button's svg a click
/// (990); a click dispatched there stands in for it.
#[test]
fn a_click_on_a_disabled_icons_glyph_runs_nothing() {
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#disabled").await.unwrap();

        page.evaluate(
            "document.querySelector('#disabled > svg').dispatchEvent(\
             new MouseEvent('click', { bubbles: true }))",
        )
        .await
        .unwrap();
        // Events run in order, so once the toggle shows its press, the glyph's click is handled.
        pointer::click(page, "#toggle").await.unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('toggle').getAttribute('aria-pressed') === 'true'",
            "the toggle's press",
        )
        .await
        .unwrap();
        let clicks: String = page.evaluate(CLICKS).await.unwrap().into_value().unwrap();
        assert_eq!(clicks, "0", "the disabled icon ran its handler");

        fixture.close().await.unwrap();
    });
}

/// `aria-pressed` follows the toggle.
#[test]
fn a_toggle_reports_its_pressed_state() {
    const PRESSED: &str = "document.getElementById('toggle').getAttribute('aria-pressed')";
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{PRESSED} === 'false'"),
            "an unpressed toggle",
        )
        .await
        .unwrap();
        keyboard::tab_to(page, "#toggle", 10).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(page, &format!("{PRESSED} === 'true'"), "Space to press it")
            .await
            .unwrap();

        fixture.console.assert_clean("toggling an icon").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `[the icon's colour, its parent's]`.
const PLAIN_COLORS: &str = "(() => {
    const el = document.getElementById('plain');
    return [getComputedStyle(el).color, getComputedStyle(el.parentElement).color];
})()";

/// Todo 628: an unstyled icon took the UA's `buttontext`, black on a pinned
/// dark theme; it follows the text around it on either scheme.
#[test]
fn a_plain_icon_takes_the_surrounding_text_colour() {
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#plain").await.unwrap();

        for scheme in ["light", "dark"] {
            page.evaluate(format!(
                "document.documentElement.setAttribute('data-lsx-theme', '{scheme}')"
            ))
            .await
            .unwrap();
            let (icon, parent): (String, String) = page
                .evaluate(PLAIN_COLORS)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(icon, parent, "the {scheme} icon");
        }

        fixture.console.assert_clean("reading the colours").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 491: without `variant` a pressed toggle painted nothing at all.
#[test]
fn a_pressed_toggle_shows_the_on_state_ring_with_and_without_a_variant() {
    use crate::button::{
        assert_gray_in_forced_colours, assert_on_in_forced_colours, assert_on_marker,
    };
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        pointer::click(page, "#bare-toggle").await.unwrap();
        assert_on_marker(page, "#bare-toggle", "#plain").await;
        assert_on_marker(page, "#toggle", "#plain").await;
        e2e::browser::force_colours(page).await.unwrap();
        assert_on_in_forced_colours(page, "#bare-toggle", "#plain").await;
        assert_on_in_forced_colours(page, "#toggle", "#plain").await;
        assert_gray_in_forced_colours(page, "#disabled").await;
        fixture.close().await.unwrap();
    });
}

/// `[pointer hits it, cursor under the pointer, background and colour]`.
const DISABLED_LOOK: &str = "(() => {
    const el = document.querySelector('[data-disabled-probe]');
    const r = el.getBoundingClientRect();
    const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
    const s = getComputedStyle(el);
    return [el.contains(hit), getComputedStyle(hit).cursor, s.backgroundColor + ' ' + s.color];
})()";

/// Todo 586: `pointer-events: none` hid `cursor: not-allowed`. The hover still paints
/// nothing: the variant's `:hover` skips a disabled control.
pub async fn assert_disabled_look(page: &chromiumoxide::Page, selector: &str) {
    wait::for_visible(page, selector).await.unwrap();
    page.evaluate(format!(
        "document.querySelector('{selector}').setAttribute('data-disabled-probe', '')"
    ))
    .await
    .unwrap();
    let (_, _, rest): (bool, String, String) = page
        .evaluate(DISABLED_LOOK)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    pointer::hover(page, selector).await.unwrap();
    let (hit, cursor, hovered): (bool, String, String) = page
        .evaluate(DISABLED_LOOK)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(
        hit,
        "{selector}: the pointer passes through to what is underneath"
    );
    assert_eq!(cursor, "not-allowed", "{selector}");
    assert_eq!(hovered, rest, "{selector} changed colour under the pointer");
}

#[test]
fn a_disabled_icon_shows_not_allowed_and_no_hover() {
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        assert_disabled_look(&fixture.page, "#disabled").await;
        fixture.close().await.unwrap();
    });
}
