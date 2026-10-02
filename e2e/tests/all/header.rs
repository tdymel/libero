//! `Header`: the banner landmark, and the page-side fix that keeps focus from
//! hiding under it (WCAG 2.4.11).

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, TAB};
use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("header", "/header").focusable("#home").run();
}

/// Scrolls `#item-5` just under the stuck banner, Shift+Tabs from `#item-6`, and returns
/// how far the focused button's bottom sits below the banner's (negative: covered).
/// A length as laid out, so a published `calc()` with the safe-area inset compares as pixels.
const RESOLVED_HEIGHT: &str = "((height) => { const probe = document.createElement('div'); \
     probe.style.height = height; document.body.append(probe); \
     const px = probe.getBoundingClientRect().height; probe.remove(); return px; })";

async fn clearance_after_shift_tab(padded: bool) -> f64 {
    let fixture = Fixture::open("/header", Viewport::Desktop).await.unwrap();
    let page = &fixture.page;
    wait::for_visible(page, "#banner").await.unwrap();
    page.evaluate(
        "(() => { document.querySelector('#item-6').focus(); \
         const covered = document.querySelector('#item-5').getBoundingClientRect().bottom; \
         const banner = document.querySelector('#banner').getBoundingClientRect().height; \
         window.scrollBy(0, covered - banner + 2); })()",
    )
    .await
    .unwrap();
    if !padded {
        page.evaluate("document.documentElement.style.scrollPaddingTop = '0px'")
            .await
            .unwrap();
    }
    // The banner publishes both on `:root` itself; the height with a top inset of 0px added.
    let published: String = page
        .evaluate(
            "(() => { const root = getComputedStyle(document.documentElement); \
             return resolved('var(--lsx-header-height)') + 'px|' + root.scrollPaddingTop; })()"
                .replace("resolved", RESOLVED_HEIGHT),
        )
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(
        published.starts_with("64px|"),
        "the banner published no height: {published}"
    );
    keyboard::press_shift(page, TAB).await.unwrap();

    let clearance: f64 = page
        .evaluate(
            "(() => { if (document.activeElement.id !== 'item-5') return NaN; \
             return document.activeElement.getBoundingClientRect().bottom \
             - document.querySelector('#banner').getBoundingClientRect().bottom; })()",
        )
        .await
        .unwrap()
        .into_value()
        .unwrap();
    fixture.console.assert_clean("the header fixture").unwrap();
    fixture.close().await.unwrap();
    clearance
}

/// The opted-in banner's size is what `:root` gets; a static header and a
/// sticky one without `publish_height` never publish.
#[test]
fn only_an_opted_in_header_publishes_its_height() {
    block_on(async {
        let fixture = Fixture::open("/header-static", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#nested").await.unwrap();
        let heights: String = page
            .evaluate(
                "(() => [resolved('var(--lsx-header-height)'), resolved('var(--lsx-header-height-lg)'), \
                 document.querySelector('#nested').getBoundingClientRect().height].join('|'))()"
                    .replace("resolved", RESOLVED_HEIGHT),
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let parts: Vec<&str> = heights.split('|').collect();
        assert_eq!(
            parts[0], parts[1],
            "root is not the banner's lg height: {heights}"
        );
        assert_eq!(
            parts[2], "48",
            "the nested header lost its own xs size: {heights}"
        );
        fixture
            .console
            .assert_clean("the static header fixture")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `#banner` and `#card` as `backdrop|background alpha|text colour`, each.
const GLASS: &str = "(() => ['#banner', '#card'].map(id => { \
    const s = getComputedStyle(document.querySelector(id)); \
    const alpha = (s.backgroundColor.match(/[\\d.]+/g) || [])[3] ?? '1'; \
    return [s.backdropFilter, alpha, s.color].join('|'); }).join(' / '))()";

/// Todo 812, 1085. Glass is translucent with a blur on the web, the banner tinted by its
/// `color` (a raised share, the tint's label, the glass cues), and turns opaque under
/// reduced transparency and forced colours.
#[test]
fn glass_blurs_and_turns_opaque_where_asked() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/header-glass", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#card").await.unwrap();
        let glass: String = page.evaluate(GLASS).await.unwrap().into_value().unwrap();
        let body: String = page
            .evaluate("getComputedStyle(document.body).color")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let surfaces: Vec<Vec<&str>> = glass
            .split(" / ")
            .map(|surface| surface.split('|').collect())
            .collect();
        let (banner, card) = (&surfaces[0], &surfaces[1]);
        assert_eq!(banner[0], "blur(12px) saturate(1.6)", "{glass}");
        let share: f64 = banner[1].parse().unwrap();
        assert!(
            share >= 0.8,
            "the tint share never drops below 80%: {glass}"
        );
        let label: String = page
            .evaluate("getComputedStyle(document.querySelector('#banner')).getPropertyValue('--lsx-header-color')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            !label.is_empty(),
            "the banner takes the tint's label: {glass}"
        );
        assert_eq!(card[0], "blur(12px)", "{glass}");
        assert_eq!(card[1], "0.8", "{glass}");
        assert_eq!(card[2], body, "{glass}");

        for feature in [
            MediaFeature::new("prefers-reduced-transparency", "reduce"),
            MediaFeature::new("forced-colors", "active"),
        ] {
            let name = feature.name.clone();
            page.execute(
                SetEmulatedMediaParams::builder()
                    .features(vec![feature])
                    .build(),
            )
            .await
            .unwrap();
            let opaque: String = page.evaluate(GLASS).await.unwrap().into_value().unwrap();
            for surface in opaque.split(" / ") {
                let parts: Vec<&str> = surface.split('|').collect();
                assert_eq!(parts[0], "none", "{name}: {opaque}");
                assert_eq!(parts[1], "1", "{name}: {opaque}");
            }
        }
        fixture.console.assert_clean("the glass fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Without `scroll-padding-top` the browser leaves a button it thinks is on
/// screen where it is, under a sticky header; the banner sets it on `:root`.
#[test]
fn scroll_padding_keeps_focus_clear_of_a_sticky_header() {
    block_on(async {
        let bare = clearance_after_shift_tab(false).await;
        assert!(bare < 0.0, "the fixture no longer hides the button: {bare}");
        let padded = clearance_after_shift_tab(true).await;
        assert!(padded > 0.0, "focus still under the header: {padded}");
    });
}

/// Todo 1647. On a coloured or gradient fill a link is the text colour, so it
/// keeps its underline at rest.
#[test]
fn a_link_on_a_fill_is_underlined_at_rest() {
    block_on(async {
        let fixture = Fixture::open("/header", Viewport::Desktop).await.unwrap();
        wait::for_js_true(
            &fixture.page,
            "['#colored a', '#gradient a'].every(s => { const a = document.querySelector(s); \
             return a && getComputedStyle(a).textDecorationLine === 'underline'; })",
            "links on the coloured and gradient headers underlined",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
