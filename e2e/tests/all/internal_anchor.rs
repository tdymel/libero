//! `InternalAnchor` (through `Anchor`): a link the router does not handle keeps its base path.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

/// Todo 4855: a `_self` internal link under a prefixed router points into the app, an
/// external one stays as written.
#[test]
fn an_internal_self_link_keeps_the_router_prefix() {
    block_on(async {
        let fixture = Fixture::open("/internal-anchor", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#self").await.unwrap();
        let js = "(() => { const a = document.getElementById('self'), b = document.getElementById('external'); \
                  return a.getAttribute('href') === '/base/elsewhere' && a.target === '_self' \
                    && b.getAttribute('href') === 'https://example.com'; })()";
        wait::for_js_true(page, js, "the internal link under /base")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}
