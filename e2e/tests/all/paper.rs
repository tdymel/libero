//! `Paper`: the interactive look is for a surface that takes a press.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

const LAYER: &str = "(sel) => { const s = getComputedStyle(document.querySelector(sel)); return s.backgroundImage + '|' + s.cursor; }";

async fn look(page: &chromiumoxide::Page, selector: &str) -> String {
    page.evaluate(format!("({LAYER})({selector:?})"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

#[test]
fn only_a_pressable_paper_tints_under_the_pointer() {
    block_on(async {
        let fixture = Fixture::open("/paper", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#link").await.unwrap();

        let plain = look(page, "#plain").await;
        let rest = look(page, "#link").await;
        assert!(rest.starts_with("none|pointer"), "{rest}");

        pointer::hover(page, "#plain").await.unwrap();
        assert_eq!(look(page, "#plain").await, plain);

        pointer::hover(page, "#link").await.unwrap();
        assert!(
            look(page, "#link").await.starts_with("linear-gradient("),
            "a hovered link paper paints a layer"
        );

        pointer::hover(page, "#gradient-link").await.unwrap();
        let gradient = look(page, "#gradient-link").await;
        assert!(
            gradient.matches("linear-gradient(").count() == 2,
            "a hover layer sits over the gradient: {gradient}"
        );
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_tinted_glass_paper_keeps_its_cues_under_the_pointer() {
    block_on(async {
        let fixture = Fixture::open("/paper", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#glass-link").await.unwrap();

        for (selector, layers) in [("#glass-link", 3), ("#glass-gradient-link", 4)] {
            let rest = look(page, selector)
                .await
                .matches("linear-gradient(")
                .count();
            pointer::hover(page, selector).await.unwrap();
            let hovered = look(page, selector).await;
            assert_eq!(rest, layers - 1, "{selector} at rest");
            assert_eq!(
                hovered.matches("linear-gradient(").count(),
                layers,
                "{selector} hovered: {hovered}"
            );
        }
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_link_paper_shows_the_focus_ring() {
    block_on(async {
        let fixture = Fixture::open("/paper", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#link").await.unwrap();

        keyboard::tab_to(page, "#link", 3).await.unwrap();
        let ring: String = page
            .evaluate("getComputedStyle(document.querySelector('#link')).outlineStyle")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(ring, "solid");
        fixture.close().await.unwrap();
    });
}
