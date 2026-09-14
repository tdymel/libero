//! `Dialog` outside a modal, and the plain surfaces beside it: `Paper` and
//! `VisuallyHidden`.

use e2e::browser::block_on;
use e2e::passes::focus;
use e2e::{Fixture, Viewport, wait};

/// Outside a modal: a named `role=dialog` that claims no modality and takes
/// no tab stop of its own.
#[test]
fn an_inline_dialog_is_named_by_its_title_and_not_modal() {
    block_on(async {
        let fixture = Fixture::open("/dialog", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#inline").await.unwrap();

        let facts: Vec<Option<String>> = page
            .evaluate(
                "(() => { const d = document.querySelector('#inline'); \
                 const h = document.getElementById(d.getAttribute('aria-labelledby')); \
                 return [d.getAttribute('role'), d.getAttribute('aria-modal'), \
                 d.getAttribute('tabindex'), h && h.tagName + ':' + h.textContent]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            facts,
            [Some("dialog".into()), None, None, Some("H2:Filters".into())]
        );
        fixture.console.assert_clean("the inline dialog").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `component: "a"` makes a card link; it has to show the keyboard ring.
#[test]
fn a_paper_link_shows_a_focus_ring() {
    block_on(async {
        let fixture = Fixture::open("/dialog", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let ring = focus::assert_focus_ring(page, "#card", 6).await.unwrap();
        focus::assert_ring_contrast(&ring).unwrap();
        fixture.close().await.unwrap();
    });
}
