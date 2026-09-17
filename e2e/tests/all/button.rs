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
    const [withIcon, glyph] = [document.getElementById('with-icon'), box('with-icon-glyph')];
    const text = document.createRange();
    text.selectNodeContents([...withIcon.childNodes].find(n => n.nodeType === 3 && n.textContent.trim()));
    const label = text.getBoundingClientRect();
    const gap = label.left - glyph.right;
    if (glyph.width < 15 || gap < 2) out.push(['icon squeezed or touching', glyph.width, gap]);
    const [mid, textMid] = [(glyph.top + glyph.bottom) / 2, (label.top + label.bottom) / 2];
    if (Math.abs(mid - textMid) > 2) out.push(['icon off the text centre', mid, textMid]);
    return out;
})()";

/// Todo 662: element children are the button's own flex items, so a caller's
/// gap, auto margin and percentage sizes reach them with no sx of its own.
/// 481's label span broke the docs search field and every docs colour swatch
/// (d1dc0f03). The `icon` prop keeps a gap and centres on the text.
#[test]
fn element_children_keep_the_callers_flex_layout() {
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

/// JS `ring(style)`: whether the style carries the house on-state ring, a
/// `currentColor` inset ring drawn as four offset shadows (todo 715): 1px deep,
/// 2px on a bordered control.
const RING: &str = "const ring = s => ['1px', '2px'].some(d => \
     [`${d} 0px`, `-${d} 0px`, `0px ${d}`, `0px -${d}`] \
     .every(o => s.boxShadow.includes(`${s.color} ${o} 0px 0px inset`)));";

/// JS `bar(style)`: whether the style carries the house start bar, a 2px
/// `currentColor` gradient (NavLink, a selected listbox row; todo 646).
const BAR: &str = "const bar = s => s.backgroundImage.replaceAll(s.color, 'currentcolor') \
     .toLowerCase() === 'linear-gradient(currentcolor, currentcolor)' \
     && s.backgroundSize.startsWith('2px');";

/// Todo 491: the on state must not rest on a fill change alone (1.4.1).
pub async fn assert_on_marker(page: &chromiumoxide::Page, on: &str, off: &str) {
    assert_marked(page, on, off, "ring").await;
}

/// [`assert_on_marker`] for the upright start bar.
pub async fn assert_on_bar(page: &chromiumoxide::Page, on: &str, off: &str) {
    assert_marked(page, on, off, "bar").await;
}

/// Whether `on` passes the JS predicate `mark` (`ring` or `bar`) and `off` does not.
async fn assert_marked(page: &chromiumoxide::Page, on: &str, off: &str, mark: &str) {
    let marked = format!(
        "(() => {{ {RING} {BAR} const s = q => getComputedStyle(document.querySelector(q)); \
         return {mark}(s('{on}')) && !{mark}(s('{off}')); }})()"
    );
    if let Err(e) = wait::for_js_true(page, &marked, "the on-state marker").await {
        let styles: Vec<String> = page
            .evaluate(format!(
                "['{on}', '{off}'].map(q => {{ const s = getComputedStyle(document.querySelector(q)); \
                 return `${{s.color}} | ${{s.boxShadow}} | ${{s.backgroundImage}} ${{s.backgroundSize}}`; }})"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        panic!("{on} vs {off}: no {mark}: {e}; color | box-shadow | background {styles:?}");
    }
}

/// Forced colours drop author fills: the on state paints `Highlight` under its
/// ring (a start bar for a NavLink or a listbox row).
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
    let [adjust, stray, line, label]: [String; 4] = page
        .evaluate(format!(
            "(() => {{ {RING} {BAR} const probe = document.createElement('div'); \
             probe.style.color = 'HighlightText'; document.body.append(probe); \
             const text = getComputedStyle(probe).color; probe.remove(); \
             const el = document.querySelector('{on}'); const s = getComputedStyle(el); \
             const stray = [el, ...el.querySelectorAll('*')] \
                 .filter(e => getComputedStyle(e).color !== text).map(e => e.tagName).join(','); \
             const line = ring(s) || bar(s) \
                 ? 'yes' : `${{s.boxShadow}} | ${{s.backgroundImage}}`; \
             return [s.forcedColorAdjust, stray, line, s.color]; }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(
        adjust, "none",
        "{on} is forced, so its label sits on a plate"
    );
    assert_ne!(label, on_bg, "{on}: the label is its own background");
    assert_eq!(stray, "", "{on}: these do not paint HighlightText");
    assert_eq!(line, "yes", "{on} lost its on-state marker");

    // Todo 686: `forced-color-adjust: none` is inherited, so a descendant's own
    // fill or border (a Badge, a filled x) kept its author colour under HighlightText.
    let painted: String = page
        .evaluate(format!(
            "(() => {{ const probe = document.createElement('div'); \
             probe.style.color = 'HighlightText'; document.body.append(probe); \
             const text = getComputedStyle(probe).color; probe.remove(); \
             const clear = 'rgba(0, 0, 0, 0)'; \
             const edge = s => ['Top', 'Right', 'Bottom', 'Left'].some(side => \
                 parseFloat(s[`border${{side}}Width`]) > 0 \
                 && ![clear, text].includes(s[`border${{side}}Color`])); \
             return [...document.querySelector('{on}').querySelectorAll('*')] \
                 .map(e => [e, getComputedStyle(e)]) \
                 .filter(([, s]) => s.backgroundColor !== clear || edge(s)) \
                 .map(([e, s]) => `${{e.tagName}} ${{s.backgroundColor}} ${{s.borderTopColor}}`).join(', '); }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(painted, "", "{on}: these keep an author fill or border");
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
fn a_pressed_button_shows_the_on_state_ring_in_every_variant() {
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
        assert_on_in_forced_colours(page, "#on-badge", "#off-badge").await;
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
