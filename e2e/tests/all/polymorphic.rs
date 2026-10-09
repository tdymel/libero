//! A component's inline `style` must not wipe the caller's style attributes.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

const BOX: &str = "(() => { const s = getComputedStyle(document.querySelector('#sized')); return s.position + '|' + s.left + '|' + s.height; })()";

#[test]
fn a_coloured_paper_keeps_its_style_attributes() {
    block_on(async {
        let fixture = Fixture::open("/polymorphic", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#sized").await.unwrap();

        let style: String = page.evaluate(BOX).await.unwrap().into_value().unwrap();
        assert_eq!(style, "relative|5px|40px");
        fixture.close().await.unwrap();
    });
}
