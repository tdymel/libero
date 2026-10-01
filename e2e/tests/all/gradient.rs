//! Gradient fills (937): the label reads on every point in both schemes; overrides, hover
//! layer, glass and forced colours.

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{contrast, pointer};
use e2e::suite::Suite;
use e2e::{Fixture, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("gradient", "/gradient").run();
}

/// Todo 1663: an uncoloured `standard` Button on a gradient Paper or Header takes its label
/// (1.03:1 in `primary` before), at rest and on hover; a plain Paper inside gives the
/// button its own colour back.
#[test]
fn a_standard_button_takes_the_gradient_label() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#in-paper").await.unwrap();
        for name in ["light", "dark"] {
            scheme(page, name).await;
            for (button, surface) in [("#in-paper", "#paper"), ("#in-header", "#header")] {
                assert_eq!(
                    style(page, button, "color").await,
                    style(page, surface, "color").await,
                    "{button} in {name}"
                );
            }
            assert_eq!(
                style(page, "#in-nested", "color").await,
                style(page, "#plain-standard", "color").await,
                "in {name}"
            );
        }
        scheme(page, "light").await;
        pointer::hover(page, "#in-paper").await.unwrap();
        wait::for_js_true(
            page,
            "getComputedStyle(document.querySelector('#in-paper')).backgroundColor !== 'rgba(0, 0, 0, 0)'",
            "the hover fill",
        )
        .await
        .unwrap();
        let hovered = ratio(page, FILL_RATIO, "#in-paper").await;
        assert!(hovered >= 4.5, "#in-paper hovered: {hovered:.2}:1");
        fixture.close().await.unwrap();
    });
}

/// Todo 1649. axe leaves text on a gradient undecided, which read as green; the
/// contrast pass now measures it, so a grey label on a white-to-silver fill fails.
#[test]
fn the_contrast_pass_measures_text_on_a_gradient() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        page.evaluate(
            "(() => { const p = document.createElement('p'); p.id = 'faint'; p.textContent = 'Faint label'; \
             p.style.cssText = 'background-image: linear-gradient(90deg, #fff, #ddd); color: #999; padding: 8px; margin: 0; position: fixed; top: 0; left: 600px; z-index: 9999'; \
             document.body.append(p); return true; })()",
        )
        .await
        .unwrap();
        let error = contrast::assert_clean(page, "#faint")
            .await
            .expect_err("a 2.8:1 label on a gradient passed");
        assert!(
            format!("{error:#}").contains("measured past axe"),
            "{error:#}"
        );
        fixture.close().await.unwrap();
    });
}

/// The worst WCAG ratio of the label over `from`, `to` and their midpoint.
const WORST_RATIO: &str = "(sel => { const el = document.querySelector(sel); \
    const probe = document.createElement('div'); document.body.append(probe); \
    const rgb = c => { probe.style.color = c; return getComputedStyle(probe).color.match(/[\\d.]+/g).slice(0, 3).map(Number); }; \
    const s = getComputedStyle(el); \
    const from = rgb(s.getPropertyValue('--lsx-gradient-from')); \
    const to = rgb(s.getPropertyValue('--lsx-gradient-to')); \
    const label = rgb(s.color); probe.remove(); \
    const mid = from.map((v, i) => Math.ceil((v + to[i]) / 2)); \
    const lum = c => c.map(v => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }) \
        .reduce((a, v, i) => a + v * [0.2126, 0.7152, 0.0722][i], 0); \
    const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); }; \
    return Math.min(...[from, to, mid].map(p => ratio(p, label))); })";

/// The WCAG ratio of the label over a flat fill, a translucent one composited over the page.
const FILL_RATIO: &str = "(sel => { const el = document.querySelector(sel); \
    const rgba = c => { const v = c.match(/[\\d.]+/g).map(Number); const k = c.startsWith('color(') ? 255 : 1; \
        return [v[0] * k, v[1] * k, v[2] * k, v.length > 3 ? v[3] : 1]; }; \
    const probe = document.createElement('div'); probe.style.color = 'var(--lsx-surface)'; \
    document.body.append(probe); const surface = rgba(getComputedStyle(probe).color); probe.remove(); \
    const pages = [document.body, document.documentElement].map(e => rgba(getComputedStyle(e).backgroundColor)); \
    const page = pages.find(p => p[3] > 0) || surface; \
    const [r, g, b, a] = rgba(getComputedStyle(el).backgroundColor); \
    const fill = [r, g, b].map((v, i) => v * a + page[i] * (1 - a)); \
    const label = rgba(getComputedStyle(el).color).slice(0, 3); \
    const lum = c => c.map(v => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }) \
        .reduce((a, v, i) => a + v * [0.2126, 0.7152, 0.0722][i], 0); \
    const [x, y] = [lum(fill), lum(label)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); })";

/// The worst WCAG ratio of the label on a glass surface's fill, the tint over the page
/// and the sheen on it: the points `glass_tint` measures, read back from the browser.
const GLASS_RATIO: &str = "(sel => { const el = document.querySelector(sel); \
    const probe = document.createElement('div'); document.body.append(probe); \
    const rgb = c => { probe.style.color = c; return getComputedStyle(probe).color.match(/[\\d.]+/g).slice(0, 3).map(Number); }; \
    const s = getComputedStyle(el); \
    const bg = getComputedStyle(document.body).backgroundColor; \
    const page = rgb(bg === 'rgba(0, 0, 0, 0)' ? 'var(--lsx-surface)' : bg); \
    const share = parseFloat(s.getPropertyValue('--lsx-glass-share') || s.getPropertyValue('--lsx-paper-tint-share')) / 100; \
    const tintShare = s.getPropertyValue('--lsx-paper-tint-share'); \
    const k = tintShare ? parseFloat(tintShare) / 100 : share; \
    const sheen = parseFloat(s.getPropertyValue('--lsx-glass-sheen') || '18') / 100; \
    const fills = (el.getAttribute('data-state') || '').includes('gradient') \
        ? (() => { const a = rgb(s.getPropertyValue('--lsx-gradient-from')); const b = rgb(s.getPropertyValue('--lsx-gradient-to')); \
            return [a, b, a.map((v, i) => Math.ceil((v + b[i]) / 2))]; })() \
        : [rgb(s.getPropertyValue('--lsx-paper-fill') || s.getPropertyValue('--lsx-header-background'))]; \
    const label = rgb(s.color); probe.remove(); \
    const lum = c => c.map(v => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }) \
        .reduce((a, v, i) => a + v * [0.2126, 0.7152, 0.0722][i], 0); \
    const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); }; \
    const points = fills.flatMap(f => { const tint = f.map((v, i) => v * k + page[i] * (1 - k)); \
        return [f, tint, tint.map(v => 255 * sheen + v * (1 - sheen))]; }); \
    return Math.min(...points.map(p => ratio(p, label))); })";

async fn ratio(page: &Page, script: &str, selector: &str) -> f64 {
    page.evaluate(format!("{script}('{selector}')"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn scheme(page: &Page, scheme: &str) {
    page.evaluate(format!(
        "document.documentElement.setAttribute('data-lsx-theme', '{scheme}')"
    ))
    .await
    .unwrap();
}

async fn style(page: &Page, selector: &str, property: &str) -> String {
    page.evaluate(format!(
        "getComputedStyle(document.querySelector('{selector}')).getPropertyValue('{property}')"
    ))
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

#[test]
fn every_label_reads_on_its_whole_gradient_in_both_schemes() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#button").await.unwrap();
        for name in ["light", "dark"] {
            scheme(page, name).await;
            for selector in [
                "#button",
                "#override",
                "#action-icon",
                "#badge",
                "#icon",
                "#paper",
                "#header",
                "#color-only",
                "#paper-gradient",
                "#in-paper",
            ] {
                let ratio = ratio(page, WORST_RATIO, selector).await;
                assert!(ratio >= 4.5, "{selector} in {name}: {ratio:.2}:1");
            }
            // Paper `color` (1017): a palette fill and a glass tint of it.
            for selector in ["#paper-color", "#paper-glass"] {
                let ratio = ratio(page, FILL_RATIO, selector).await;
                assert!(ratio >= 4.5, "{selector} in {name}: {ratio:.2}:1");
            }
            // 1085: glass over a colour or a gradient, sheen included.
            for selector in ["#paper-glass", "#header-tint", "#glass", "#header"] {
                let ratio = ratio(page, GLASS_RATIO, selector).await;
                assert!(ratio >= 4.5, "{selector} in {name}: {ratio:.2}:1");
            }
        }
        fixture.close().await.unwrap();
    });
}

#[test]
fn an_override_replaces_the_theme_stops_and_other_variants_ignore_it() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#button").await.unwrap();
        let theme = style(page, "#button", "background-image").await;
        let own = style(page, "#override", "background-image").await;
        assert!(theme.starts_with("linear-gradient(45deg"), "{theme}");
        assert!(own.starts_with("linear-gradient(90deg"), "{own}");
        assert_ne!(
            style(page, "#button", "--lsx-gradient-to").await,
            style(page, "#override", "--lsx-gradient-to").await
        );
        assert_eq!(style(page, "#ignored", "background-image").await, "none");
        fixture.close().await.unwrap();
    });
}

/// `color` is the first stop (1016); unset, the theme's `from` stays.
#[test]
fn color_is_the_first_stop() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#button").await.unwrap();
        let from = |selector| style(page, selector, "--lsx-gradient-from");
        let fill = |var| style(page, "#button", var);
        assert_eq!(from("#button").await, fill("--lsx-primary-fill-6").await);
        assert_eq!(from("#override").await, fill("--lsx-success-fill-6").await);
        let error = fill("--lsx-error-fill-6").await;
        assert_eq!(from("#color-only").await, error);
        assert_eq!(from("#paper-gradient").await, error);
        assert_eq!(from("#header").await, error);
        assert_eq!(
            style(page, "#color-only", "--lsx-gradient-to").await,
            style(page, "#button", "--lsx-gradient-to").await
        );
        fixture.close().await.unwrap();
    });
}

/// Paper `color` (1017): a palette fill hands its label to rings inside; a literal is as given;
/// glass tints it; `Text` takes a plain `color`.
#[test]
fn paper_color_paints_the_fill_and_its_label() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#paper-color").await.unwrap();
        assert_eq!(
            style(page, "#paper-color", "background-image").await,
            "none"
        );
        assert_eq!(
            style(page, "#in-paper-color", "--lsx-focus-contrast").await,
            style(page, "#paper-color", "--lsx-paper-fill-contrast").await
        );
        assert_eq!(
            style(page, "#paper-literal", "background-color").await,
            "rgb(18, 52, 86)"
        );
        let glass = style(page, "#paper-glass", "background-color").await;
        assert!(
            glass.starts_with("rgba") || glass.contains("color(srgb"),
            "{glass}"
        );
        assert_ne!(style(page, "#paper-glass", "backdrop-filter").await, "none");
        assert_ne!(
            style(page, "#text-color", "color").await,
            style(page, "#text", "color").await
        );
        fixture.close().await.unwrap();
    });
}

/// 1085: a coloured or gradient glass saturates its backdrop and carries a highlight and a
/// sheen over the tint; an uncoloured one stays plain blur.
#[test]
fn a_coloured_glass_carries_the_glass_cues() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#paper-glass").await.unwrap();
        for selector in ["#paper-glass", "#header-tint", "#glass", "#header"] {
            let filter = style(page, selector, "backdrop-filter").await;
            assert!(
                filter.contains("blur(12px)") && filter.contains("saturate"),
                "{selector}: {filter}"
            );
            let image = style(page, selector, "background-image").await;
            let layers = image.matches("linear-gradient").count();
            assert!(layers >= 2, "{selector}: {image}");
        }
        let share = style(page, "#paper-glass", "--lsx-paper-tint-share").await;
        assert!(share.ends_with('%'), "{share}");
        // The tint of `#header-tint` is measured for its colour, like the Paper's.
        assert!(
            !style(page, "#header-tint", "--lsx-paper-tint-share")
                .await
                .is_empty()
        );
        assert!(!style(page, "#glass", "--lsx-glass-share").await.is_empty());
        fixture.close().await.unwrap();
    });
}

#[test]
fn hover_lays_a_state_layer_over_the_gradient() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#button").await.unwrap();
        let layers = |image: &str| image.matches("linear-gradient").count();
        assert_eq!(layers(&style(page, "#button", "background-image").await), 1);
        assert_eq!(
            layers(&style(page, "#selected", "background-image").await),
            2
        );
        pointer::hover(page, "#button").await.unwrap();
        wait::for_js_true(
            page,
            "getComputedStyle(document.querySelector('#button')).backgroundImage.split('linear-gradient').length === 3",
            "the hover layer",
        )
        .await
        .unwrap();
        // A static badge grows no hover.
        pointer::hover(page, "#badge").await.unwrap();
        assert_eq!(layers(&style(page, "#badge", "background-image").await), 1);
        fixture.close().await.unwrap();
    });
}

#[test]
fn glass_mixes_the_stops_down_and_text_clips_to_the_glyphs() {
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#glass").await.unwrap();
        let glass = style(page, "#glass", "background-image").await;
        assert!(
            glass.contains("color-mix") || glass.contains("rgba") || glass.contains("/ 0.8"),
            "{glass}"
        );
        assert_ne!(style(page, "#glass", "backdrop-filter").await, "none");
        // Header combines them as Paper does, and a ring inside takes the label.
        let header = style(page, "#header", "background-image").await;
        assert!(header.starts_with("linear-gradient"), "{header}");
        assert!(
            header.contains("color-mix") || header.contains("rgba") || header.contains("/ 0.8"),
            "{header}"
        );
        assert_ne!(style(page, "#header", "backdrop-filter").await, "none");
        assert_eq!(
            style(page, "#in-header", "--lsx-focus-contrast").await,
            style(page, "#header", "--lsx-gradient-contrast").await
        );
        // So does one inside a gradient Paper (971).
        assert_ne!(style(page, "#paper", "--lsx-gradient-from").await, "");
        assert_eq!(
            style(page, "#in-paper", "--lsx-focus-contrast").await,
            style(page, "#paper", "--lsx-gradient-contrast").await
        );
        assert_eq!(
            style(page, "#in-paper", "--lsx-focus-ring-halo").await,
            style(page, "#paper", "--lsx-gradient-from").await
        );
        assert_eq!(style(page, "#text", "background-clip").await, "text");
        assert_eq!(style(page, "#text", "color").await, "rgba(0, 0, 0, 0)");
        fixture.close().await.unwrap();
    });
}

#[test]
fn forced_colours_drop_the_image() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/gradient", Viewport::Desktop).await.unwrap();
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
        for selector in [
            "#button",
            "#paper",
            "#glass",
            "#text",
            "#header",
            "#paper-glass",
        ] {
            // One `none` per layer: a glass carries its cue layers.
            let image = style(page, selector, "background-image").await;
            assert!(
                image.split(", ").all(|layer| layer == "none"),
                "{selector}: {image}"
            );
        }
        assert_eq!(style(page, "#glass", "backdrop-filter").await, "none");
        assert_eq!(style(page, "#header", "backdrop-filter").await, "none");
        for selector in ["#paper-glass", "#header-tint"] {
            assert_eq!(
                style(page, selector, "backdrop-filter").await,
                "none",
                "{selector}"
            );
        }
        assert_ne!(style(page, "#text", "color").await, "rgba(0, 0, 0, 0)");
        fixture.close().await.unwrap();
    });
}
