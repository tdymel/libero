//! `Button`: a busy button swallows keyboard activation, every mode shows a
//! ring on Tab, and link mode navigates through the router (todo 406).

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::Driver;
use e2e::passes::contrast::COLOUR_JS;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const PLAIN: &str = "#plain";
const BUSY: &str = "#busy";
const LINK: &str = "#to-landing";

/// Todo 2338 (1.4.4): at 200% text every size's line fits the button and the
/// segment, which clip their overflow; at 100% the steps keep their heights.
#[test]
fn every_size_fits_its_label_at_200_percent_text() {
    block_on(async {
        let fixture = Fixture::open("/button/sizes", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#sizes > button").await.unwrap();
        let xs: f64 = page
            .evaluate("document.querySelector('#sizes > button').getBoundingClientRect().height")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(xs, 24.0, "the xs step changed height at 100%");
        let clipped: Vec<String> = page
            .evaluate(
                "(() => { document.documentElement.style.fontSize = '200%'; \
                 return [...document.querySelectorAll('#sizes > button, #sizes label')] \
                 .filter(b => parseFloat(getComputedStyle(b).fontSize) * 1.2 > b.clientHeight) \
                 .map(b => b.textContent); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(clipped.is_empty(), "clipped at 200%: {clipped:?}");
        fixture.console.assert_clean("the button sizes").unwrap();
        fixture.close().await.unwrap();
    });
}

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

/// Todo 877: a literal fill publishes no palette contrast, so the label takes
/// black or white from the fill; a named colour asks `contrast-color()`.
#[test]
fn a_literal_fill_gets_a_readable_label() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let colors: Vec<String> = fixture
            .page
            .evaluate(
                "['#literal-filled', '#literal-tonal', '#literal-named']\
                 .map(s => getComputedStyle(document.querySelector(s)).color)\
                 .concat([String(CSS.supports('color', 'contrast-color(red)'))])",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            colors[..2],
            ["rgb(0, 0, 0)", "rgb(255, 255, 255)"],
            "{colors:?}"
        );
        if colors[3] == "true" {
            assert_eq!(colors[2], "rgb(0, 0, 0)", "{colors:?}");
        }
        fixture.close().await.unwrap();
    });
}

/// Todo 2339: a `<button>` takes the theme's typeface and letter spacing, as link mode does.
#[test]
fn a_button_and_a_link_share_the_theme_font() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let fonts: Vec<String> = fixture
            .page
            .evaluate(format!(
                "['body', '{PLAIN}', '{LINK}'].map(q => {{ const s = getComputedStyle(document.querySelector(q)); \
                 return `${{s.fontFamily}} | ${{s.letterSpacing}}`; }})"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(fonts[1], fonts[0], "{fonts:?}");
        assert_eq!(fonts[2], fonts[0], "{fonts:?}");
        fixture.close().await.unwrap();
    });
}

/// Todo 2410: the other `<button>`-rooted parts take the page's typeface and letter spacing.
#[test]
fn button_rooted_parts_take_the_page_font() {
    const MENUBAR_TRIGGER: &str = "[role=menubar] [role=menuitem]";
    block_on(async {
        e2e::browser::at_once(
            [
                "/accordion",
                "/tabs",
                "/stepper",
                "/bottom-navigation",
                "/image",
                "/table",
                "/calendar",
                "/menubar",
            ],
            async |route| {
                let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
                let page = &fixture.page;
                let parts: &[&str] = match route {
                    "/accordion" => &["[data-accordion-heading] > button"],
                    "/tabs" => &["[role=tab]"],
                    "/stepper" => &["[data-slot=step] > button"],
                    "/bottom-navigation" => &["[aria-label=Main] button"],
                    "/image" => &["button[aria-haspopup=dialog]"],
                    "/table" => &["[data-sort-button]"],
                    "/calendar" => &["[data-slot=day]", "[data-slot=title]"],
                    _ => &[MENUBAR_TRIGGER],
                };
                assert_takes_the_page_font(page, parts).await;
                if route == "/menubar" {
                    pointer::click(page, MENUBAR_TRIGGER).await.unwrap();
                    wait::for_visible(page, "[role=menu] [role=menuitem]")
                        .await
                        .unwrap();
                    assert_takes_the_page_font(page, &["[role=menu] [role=menuitem]"]).await;
                }
                fixture.close().await.unwrap();
            },
        )
        .await;
    });
}

/// Each of `selectors` takes the body's typeface and letter spacing, set here to values
/// no UA sheet or theme has, so a `<button>` that resets either shows (todo 2376).
pub(crate) async fn assert_takes_the_page_font(page: &chromiumoxide::Page, selectors: &[&str]) {
    let fonts: Vec<String> = page
        .evaluate(format!(
            "(() => {{ document.body.style.fontFamily = 'Georgia, serif'; \
             document.body.style.letterSpacing = '0.1em'; \
             const font = q => {{ const s = getComputedStyle(document.querySelector(q)); \
               return `${{q}}: ${{s.fontFamily}} | ${{s.letterSpacing}}`; }}; \
             return {selectors:?}.map(font); }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    let body: Vec<String> = page
        .evaluate(format!(
            "{selectors:?}.map(q => {{ const s = getComputedStyle(document.body); \
             return `${{q}}: ${{s.fontFamily}} | ${{s.letterSpacing}}`; }})"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(fonts, body, "[selector: font-family | letter-spacing]");
}

fn presses(which: &str) -> String {
    format!("document.querySelector('#presses').dataset.{which}")
}

/// The plain button is the control: its presses land after the busy ones, so a busy count
/// still at 0 then means they were swallowed, not merely not yet handled.
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

/// Todo 662: element children are the button's own flex items, so a caller's gap, margins
/// and percentages reach them (481's label span broke that). `icon` keeps a gap.
async fn element_children_layout<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let search = d.rect("#search-like").await?;
    let pad: f64 = d
        .style("#search-like", "padding-right")
        .await?
        .trim_end_matches("px")
        .parse()?;
    let kbd = d.rect("#search-kbd").await?;
    let icon = d.rect("#search-icon").await?;
    let search_end = search.x + search.width - pad - 1.0;
    ensure!(
        (search_end - (kbd.x + kbd.width)).abs() <= 1.0,
        "kbd not at the end: {kbd:?} in {search:?}"
    );
    ensure!(
        icon.x - search.x <= 24.0,
        "icon not at the start: {icon:?} in {search:?}"
    );

    let swatch = d.rect("#swatch").await?;
    let fill = d.rect("#swatch-fill").await?;
    ensure!(
        fill.width >= swatch.width - 2.0 && fill.height >= swatch.height - 2.0,
        "swatch fill collapsed: {fill:?} in {swatch:?}"
    );

    let glyph = d.rect("#with-icon-glyph").await?;
    let label = d.rect("#with-icon-text").await?;
    let gap = label.x - (glyph.x + glyph.width);
    ensure!(
        glyph.width >= 15.0 && gap >= 2.0,
        "icon squeezed or touching: {glyph:?}, gap {gap}"
    );
    let (mid, text_mid) = (glyph.y + glyph.height / 2.0, label.y + label.height / 2.0);
    ensure!(
        (mid - text_mid).abs() <= 2.0,
        "icon off the text centre: {mid} vs {text_mid}"
    );
    Ok(())
}

e2e::scenario!(
    element_children_keep_the_callers_flex_layout,
    "/button",
    element_children_layout
);

/// The label against the background as painted, `null` while either is
/// translucent (the resting unfilled variants, or a transition under way).
fn pair(selector: &str) -> String {
    format!(
        "(() => {{ {COLOUR_JS} \
         const style = getComputedStyle(document.querySelector('{selector}')); \
         const [fg, bg] = [RGBA(style.color), RGBA(style.backgroundColor)]; \
         return fg[3] < 1 || bg[3] < 1 ? null : CONTRAST(fg, bg); }})()"
    )
}

/// Todo 1715: `COLOUR_JS` reads a mixed colour's `color(srgb ..)` as painted, not as its digits.
#[test]
fn the_shared_colour_reader_takes_color_srgb() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let ratio: f64 = e2e::js(
            &fixture.page,
            format!("(() => {{ {COLOUR_JS} return CONTRAST(RGBA('color(srgb 1 1 1)'), RGBA('rgb(0, 0, 0)')); }})()"),
        )
        .await;
        assert!(
            (ratio - 21.0).abs() < 0.01,
            "white on black read {ratio:.2}:1"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 452: an outlined `primary` label read 3.52:1 on its hover tint. Resting unfilled
/// buttons paint no background, so any ratio means the hover rule applied; for all four the
/// fill must change from rest under `:hover`, in both schemes (todo 1828).
#[test]
fn a_hovered_button_s_label_reads_on_its_hover_fill() {
    use e2e::Scheme;
    block_on(async {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open_in("/button", Viewport::Desktop, scheme)
                .await
                .unwrap();
            let page = &fixture.page;

            for selector in ["#outlined", "#standard-error", "#elevated", "#filled-muted"] {
                let fill = format!(
                    "getComputedStyle(document.querySelector({selector:?})).backgroundColor"
                );
                let _: bool = e2e::js(
                    page,
                    format!("(window.__rest = {fill}, window.__last = undefined, true)"),
                )
                .await;
                pointer::hover(page, selector).await.unwrap();
                let settled = format!(
                    "(() => {{ if (!document.querySelector({selector:?}).matches(':hover') \
                     || {fill} === window.__rest) return false; const r = {}; \
                     if (r === null || r !== window.__last) {{ window.__last = r; return false; }} \
                     return true; }})()",
                    pair(selector)
                );
                wait::for_js_true(
                    page,
                    &settled,
                    &format!("{selector}'s hover fill to settle"),
                )
                .await
                .unwrap_or_else(|e| panic!("{scheme:?}: {e}"));
                let ratio: f64 = e2e::js(page, pair(selector)).await;
                assert!(
                    ratio >= 4.5,
                    "{scheme:?}: {selector} hovered reads {ratio:.2}:1"
                );
            }

            fixture
                .console
                .assert_clean(&format!("hovering buttons, {scheme:?}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// JS `ring(style)`: whether the style carries the house on-state ring, four `currentColor`
/// inset shadows (715): 1px deep, 2px on a bordered control.
const RING: &str = "const ring = s => ['1px', '2px'].some(d => \
     [`${d} 0px`, `-${d} 0px`, `0px ${d}`, `0px -${d}`] \
     .every(o => s.boxShadow.includes(`${s.color} ${o} 0px 0px inset`)));";

/// JS `bar(style)`: whether the style carries the house start bar, a 2px gradient at 3:1 on
/// the row's tint (646). Not the label colour: the bar is the one indicator (764).
/// Needs [`COLOUR_JS`] in scope.
pub const BAR: &str = "const bar = s => { \
     const m = s.backgroundImage.match(/^linear-gradient\\((rgb\\([^)]*\\)), (rgb\\([^)]*\\))\\)$/); \
     if (!m || m[1] !== m[2] || m[1] === s.color || !s.backgroundSize.startsWith('2px')) return false; \
     return CONTRAST(RGBA(m[1]), RGBA(s.backgroundColor)) >= 3; };";

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
        "(() => {{ {COLOUR_JS} {RING} {BAR} const s = q => getComputedStyle(document.querySelector(q)); \
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
    assert_forced_colours(page, on, off, true).await;
}

/// [`assert_on_in_forced_colours`] for a row marked by its tint alone: no ring or bar to find.
pub async fn assert_unmarked_in_forced_colours(page: &chromiumoxide::Page, on: &str, off: &str) {
    assert_forced_colours(page, on, off, false).await;
}

async fn assert_forced_colours(page: &chromiumoxide::Page, on: &str, off: &str, marker: bool) {
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

    // Unforced, or Chromium backs the label with a `Canvas` plate (646). The start bar
    // turns `HighlightText` too (764): its author colour vanishes on `Highlight`.
    let [adjust, stray, line, label]: [String; 4] = page
        .evaluate(format!(
            "(() => {{ {COLOUR_JS} {RING} {BAR} const probe = document.createElement('div'); \
             probe.style.color = 'HighlightText'; document.body.append(probe); \
             const text = getComputedStyle(probe).color; probe.remove(); \
             const el = document.querySelector('{on}'); const s = getComputedStyle(el); \
             const stray = [el, ...el.querySelectorAll('*')] \
                 .filter(e => getComputedStyle(e).color !== text).map(e => e.tagName).join(','); \
             const inked = s.backgroundImage === `linear-gradient(${{s.color}}, ${{s.color}})`; \
             const line = ring(s) || inked \
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
    if marker {
        assert_eq!(line, "yes", "{on} lost its on-state marker");
    }

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
        e2e::browser::force_colours(page).await.unwrap();
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

/// Enter in a field clicks the busy submit button. The scripted `requestSubmit` lands after
/// the keys, so a count of exactly 1 means none of them submitted.
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
