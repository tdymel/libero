//! Non-text contrast (WCAG 1.4.11) of a control's boundary, in both schemes.
//! A helper, not a unit: the units that own the boundaries call it.

use e2e::browser::{Scheme, block_on, emulate_media};
use e2e::{Fixture, Viewport, wait};

/// In scope of every `body`: `RATIO(a, b)` of two opaque computed colours, and
/// `PAGE(el)`, the first opaque background above `el`.
const HELPERS: &str = "
    const RATIO = (a, b) => {
        const lum = v => {
            const [r, g, b] = v.match(/[\\d.]+/g).slice(0, 3).map(Number).map(c => c / 255)
                .map(c => c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
            return 0.2126 * r + 0.7152 * g + 0.0722 * b;
        };
        const [x, y] = [lum(a), lum(b)];
        return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
    };
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
    block_on(async {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open(path, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            emulate_media(page, scheme, None).await.unwrap();
            let pairs = format!("(() => {{ {HELPERS} {body} }})()");
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
                    ratio >= 3.0,
                    "{path} {}: {name} reads {ratio:.2}:1",
                    scheme.name()
                );
            }
            fixture.close().await.unwrap();
        }
    });
}
