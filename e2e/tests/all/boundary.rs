//! Non-text contrast (WCAG 1.4.11) of a control's boundary, in both schemes.
//! A helper, not a unit: the units that own the boundaries call it.

use e2e::browser::{Scheme, block_on, emulate_media};
use e2e::passes::contrast::COLOUR_JS;
use e2e::{Fixture, Viewport, wait};

/// In scope of every `body`, beside [`COLOUR_JS`]: `RATIO(a, b)` of two opaque computed
/// colours, and `PAGE(el)`, the first opaque background above `el`.
const HELPERS: &str = "
    const RATIO = (a, b) => CONTRAST(RGBA(a), RGBA(b));
    const PAGE = el => {
        let at = el.parentElement;
        while (getComputedStyle(at).backgroundColor === 'rgba(0, 0, 0, 0)') at = at.parentElement;
        return getComputedStyle(at).backgroundColor;
    };
    const CSS = (el, prop, pseudo) => getComputedStyle(el, pseudo)[prop];
";

/// Opens `path` light and dark and asserts every `[name, ratio]` pair `body`
/// returns is at least 3:1. `body` is a JS function body.
pub(crate) fn assert_boundaries(path: &str, body: &str) {
    assert_ratios(path, body, 3.0);
}

/// [`assert_boundaries`] at 4.5:1, for visible text axe skips, such as an `aria-hidden` badge (1.4.3).
pub(crate) fn assert_text_contrast(path: &str, body: &str) {
    assert_ratios(path, body, 4.5);
}

fn assert_ratios(path: &str, body: &str, floor: f64) {
    block_on(async {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open(path, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            emulate_media(page, scheme, None).await.unwrap();
            let pairs = format!("(() => {{ {COLOUR_JS} {HELPERS} {body} }})()");
            // The scheme flips through a media query; wait for the page to follow.
            wait::for_js_true(
                page,
                &format!("(() => {{ const r = {pairs}; if (JSON.stringify(r) !== window.__last) {{ window.__last = JSON.stringify(r); return false; }} return true; }})()"),
                "settled colours",
            )
            .await
            .unwrap();
            let pairs: Vec<(String, f64)> =
                page.evaluate(pairs).await.unwrap().into_value().unwrap();
            for (name, ratio) in pairs {
                eprintln!("{path} {}: {name} {ratio:.2}:1", scheme.name());
                assert!(
                    ratio >= floor,
                    "{path} {}: {name} reads {ratio:.2}:1",
                    scheme.name()
                );
            }
            fixture.close().await.unwrap();
        }
    });
}
