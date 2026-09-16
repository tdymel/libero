//! `Button`: a busy button swallows keyboard activation, every mode shows a
//! ring on Tab, and link mode navigates through the router (todo 406).

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const PLAIN: &str = "#plain";
const BUSY: &str = "#busy";
const LINK: &str = "#to-landing";

/// Todo 586, in link mode: the one disabled `Button` the fixture has.
#[test]
fn a_disabled_link_shows_not_allowed_and_no_hover() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        crate::action_icon::assert_disabled_look(&fixture.page, "#disabled-link").await;
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("button", "/button")
        .focusable(PLAIN)
        .focusable(BUSY)
        .focusable(LINK)
        .targets(PLAIN)
        .targets(BUSY)
        .targets(LINK)
        .focusable("#long")
        .focusable("#on-filled")
        .focusable("#off-tonal")
        .focusable("#on-tonal")
        .focusable("#on-elevated")
        .focusable("#on-outlined")
        .focusable("#on-standard")
        .run();
}

fn presses(which: &str) -> String {
    format!("document.querySelector('#presses').dataset.{which}")
}

/// The plain button is the control: its presses land after the busy ones in
/// the same event queue, so a busy count still at 0 once they have landed
/// means the busy presses were swallowed, not merely not yet handled.
#[test]
fn a_loading_button_swallows_enter_and_space() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, PLAIN, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '2'", presses("plain")),
            "Enter and Space to activate the plain button",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, BUSY, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();

        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '3'", presses("plain")),
            "the plain button's press after the busy ones",
        )
        .await
        .unwrap();

        let busy: String = page
            .evaluate(presses("busy"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(busy, "0", "a loading button ran its onclick");

        fixture
            .console
            .assert_clean("pressing a busy button")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `[failure, ...]` of the element-children layout; empty when it holds.
const ELEMENT_CHILDREN: &str = "(() => {
    const box = id => document.getElementById(id).getBoundingClientRect();
    const out = [];
    const [search, icon, kbd] = [box('search-like'), box('search-icon'), box('search-kbd')];
    const pad = parseFloat(getComputedStyle(document.getElementById('search-like')).paddingRight);
    if (Math.abs(search.right - pad - 1 - kbd.right) > 1) out.push(['kbd not at the end', kbd.right, search.right]);
    if (icon.left - search.left > 24) out.push(['icon not at the start', icon.left, search.left]);
    const [swatch, fill] = [box('swatch'), box('swatch-fill')];
    if (fill.width < swatch.width - 2 || fill.height < swatch.height - 2)
        out.push(['swatch fill collapsed', fill.width, fill.height, swatch.width, swatch.height]);
    return out;
})()";

/// A caller that unwraps the label span gets its element children back as
/// the button's flex items: its gap, auto margin and percentage sizes reach
/// them. 481's span broke the docs search field and every docs colour swatch
/// (d1dc0f03); the docs callers use this same rule.
#[test]
fn an_unwrapped_label_keeps_the_callers_flex_layout() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let failures: Vec<serde_json::Value> = fixture
            .page
            .evaluate(ELEMENT_CHILDREN)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(failures.is_empty(), "{failures:?}");
        fixture.close().await.unwrap();
    });
}

/// The label against the background as painted, `null` while either is
/// translucent (the resting unfilled variants, or a transition under way).
const PAIR: &str = "(selector => {
    const style = getComputedStyle(document.querySelector(selector));
    const rgb = value => {
        const [r, g, b, a = 1] = value.match(/[\\d.]+/g).map(Number);
        return a < 1 ? null : [r, g, b];
    };
    const luminance = c => {
        const [r, g, b] = c.map(v => v / 255).map(v =>
            v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4);
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
    };
    const [fg, bg] = [rgb(style.color), rgb(style.backgroundColor)];
    if (!fg || !bg) return null;
    const [a, b] = [luminance(fg), luminance(bg)];
    return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
})";

/// Todo 452: an outlined `primary` label read 3.52:1 on its hover tint. The
/// resting unfilled buttons paint no background, so a ratio at all means the
/// pointer's rule applied; the wait also rides out any transition.
#[test]
fn a_hovered_button_s_label_reads_on_its_hover_fill() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        for selector in ["#outlined", "#standard-error", "#elevated", "#filled-muted"] {
            pointer::hover(page, selector).await.unwrap();
            let settled = format!(
                "(() => {{ const r = {PAIR}('{selector}'); \
                 if (r === null || r !== window.__last) {{ window.__last = r; return false; }} \
                 return true; }})()"
            );
            wait::for_js_true(page, &settled, "a settled hover pair")
                .await
                .unwrap();
            let ratio: f64 = page
                .evaluate(format!("{PAIR}('{selector}')"))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            eprintln!("{selector} hovered: {ratio:.2}:1");
            assert!(ratio >= 4.5, "{selector} hovered reads {ratio:.2}:1");
        }

        fixture.console.assert_clean("hovering buttons").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Whether `on` carries the house on-state line (a 2px `currentColor` gradient
/// bar, todo 646) and `off` does not.
fn marked(on: &str, off: &str) -> String {
    format!(
        "(() => {{ const s = q => getComputedStyle(document.querySelector(q)); \
         const [on, off] = [s('{on}'), s('{off}')]; \
         const img = on.backgroundImage.replaceAll(on.color, 'currentcolor').toLowerCase(); \
         return img === 'linear-gradient(currentcolor, currentcolor)' \
             && on.backgroundSize.includes('2px') \
             && !off.backgroundImage.includes('gradient'); }})()"
    )
}

/// Todo 491: the on state must not rest on a fill change alone (1.4.1).
pub async fn assert_on_marker(page: &chromiumoxide::Page, on: &str, off: &str) {
    if let Err(e) = wait::for_js_true(page, &marked(on, off), "the on-state line").await {
        let lines: Vec<String> = page
            .evaluate(format!(
                "['{on}', '{off}'].map(q => {{ const s = getComputedStyle(document.querySelector(q)); \
                 return `${{s.color}} | ${{s.backgroundImage}} | ${{s.backgroundSize}}`; }})"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        panic!("{on} vs {off}: {e}; color | background-image | size {lines:?}");
    }
}

/// Forced colours drop author fills: the on state paints `Highlight` under its line.
pub async fn assert_on_in_forced_colours(page: &chromiumoxide::Page, on: &str, off: &str) {
    let [on_bg, off_bg, highlight]: [String; 3] = page
        .evaluate(format!(
            "(() => {{ const probe = document.createElement('div'); \
             probe.style.background = 'Highlight'; document.body.append(probe); \
             const highlight = getComputedStyle(probe).backgroundColor; probe.remove(); \
             const bg = q => getComputedStyle(document.querySelector(q)).backgroundColor; \
             return [bg('{on}'), bg('{off}'), highlight]; }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(on_bg, highlight, "{on} is not Highlight in forced colours");
    assert_ne!(off_bg, highlight, "{off} is Highlight too");

    // Unforced, or Chromium backs the label with a `Canvas` plate (todo 646);
    // then every descendant has to follow the label colour by hand.
    let [adjust, stray, line]: [String; 3] = page
        .evaluate(format!(
            "(() => {{ const probe = document.createElement('div'); \
             probe.style.color = 'HighlightText'; document.body.append(probe); \
             const text = getComputedStyle(probe).color; probe.remove(); \
             const el = document.querySelector('{on}'); const s = getComputedStyle(el); \
             const stray = [el, ...el.querySelectorAll('*')] \
                 .filter(e => getComputedStyle(e).color !== text).map(e => e.tagName).join(','); \
             return [s.forcedColorAdjust, stray, s.backgroundImage]; }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(
        adjust, "none",
        "{on} is forced, so its label sits on a plate"
    );
    assert_eq!(stray, "", "{on}: these do not paint HighlightText");
    assert!(
        line.contains("gradient"),
        "{on} lost its on-state line: {line}"
    );
}

/// Todo 517: under forced colours a disabled control's text is `GrayText`,
/// where the fade alone does not read as disabled.
pub async fn assert_gray_in_forced_colours(page: &chromiumoxide::Page, selector: &str) {
    assert_text_in_forced_colours(page, selector, "GrayText").await;
}

/// A disabled on state keeps `HighlightText`: `GrayText` does not read on `Highlight`.
pub async fn assert_text_in_forced_colours(
    page: &chromiumoxide::Page,
    selector: &str,
    system: &str,
) {
    let [color, expected]: [String; 2] = page
        .evaluate(format!(
            "(() => {{ const probe = document.createElement('div'); \
             probe.style.color = '{system}'; document.body.append(probe); \
             const expected = getComputedStyle(probe).color; probe.remove(); \
             return [getComputedStyle(document.querySelector('{selector}')).color, expected]; }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(
        color, expected,
        "{selector} is not {system} in forced colours"
    );
}

#[test]
fn a_pressed_button_shows_the_on_state_line_in_every_variant() {
    const VARIANTS: [&str; 5] = ["filled", "tonal", "elevated", "outlined", "standard"];
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        for variant in VARIANTS {
            assert_on_marker(page, &format!("#on-{variant}"), &format!("#off-{variant}")).await;
        }
        crate::calendar::force_colours(page).await;
        for variant in VARIANTS {
            assert_on_in_forced_colours(
                page,
                &format!("#on-{variant}"),
                &format!("#off-{variant}"),
            )
            .await;
        }
        assert_gray_in_forced_colours(page, "#disabled-link").await;
        fixture.close().await.unwrap();
    });
}

/// An `<a>` without `href` maps to `generic`, which drops the link role and
/// the name from content: a disabled link-mode button read as plain text.
#[test]
fn a_disabled_link_is_still_a_link() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let tree = e2e::ax::snapshot(&fixture.page, "#disabled-link")
            .await
            .unwrap();
        assert!(
            tree.starts_with("link \"Disabled link\" [disabled]"),
            "the disabled link is exposed as:\n{tree}"
        );
        fixture.close().await.unwrap();
    });
}

/// Enter in a field is implicit submission, a synthetic click on the busy
/// submit button. The scripted `requestSubmit` is the control: it lands after
/// the key presses, so a count of exactly 1 means none of them submitted.
#[test]
fn a_loading_submit_button_does_not_submit_its_form() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let submits = "document.querySelector('#form').dataset.submits";

        keyboard::tab_to(page, "#field", 20).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::tab_to(page, "#busy-submit", 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();

        page.evaluate("document.querySelector('#form').requestSubmit()")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{submits} !== '0'"), "the scripted submit")
            .await
            .unwrap();
        let count: String = page.evaluate(submits).await.unwrap().into_value().unwrap();
        assert_eq!(count, "1", "a loading submit button submitted its form");

        fixture
            .console
            .assert_clean("submitting while busy")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A marker on `window` survives only a client-side navigation, so it tells
/// the router's `Link` apart from a full page load of the same URL.
#[test]
fn link_mode_navigates_through_the_router() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        page.evaluate("window.__buttonMarker = 1").await.unwrap();
        keyboard::tab_to(page, LINK, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        wait::for_selector(page, "#landing").await.unwrap();
        let (path, kept): (String, bool) = page
            .evaluate("[location.pathname, window.__buttonMarker === 1]")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(path, "/button/landing");
        assert!(kept, "the link reloaded the page instead of routing");

        fixture
            .console
            .assert_clean("following a link button")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
