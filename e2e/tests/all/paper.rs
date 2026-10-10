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

/// Todo 3146: a coloured paper steps to a darker shade under the pointer instead of laying
/// a layer of its label over the fill, which took the label under 4.5:1.
#[test]
fn a_coloured_paper_steps_its_shade_under_the_pointer() {
    const FILL: &str = "(sel) => { const s = getComputedStyle(document.querySelector(sel)); \
                        return s.backgroundColor + '|' + s.backgroundImage + '|' + s.color; }";
    block_on(async {
        let fixture = Fixture::open("/paper", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#press-primary").await.unwrap();
        let fill = async |selector: &str| -> String {
            page.evaluate(format!("({FILL})({selector:?})"))
                .await
                .unwrap()
                .into_value()
                .unwrap()
        };

        let rest = fill("#press-primary").await;
        pointer::hover(page, "#press-primary").await.unwrap();
        let hovered = fill("#press-primary").await;

        let rest_fill = rest.split('|').next().unwrap();
        assert!(
            hovered.contains("|none|"),
            "no layer over the fill: {hovered}"
        );
        assert!(
            !hovered.starts_with(rest_fill),
            "the fill steps to another shade: {rest} then {hovered}"
        );
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_button_paper_is_no_submit_and_drops_the_user_agent_look() {
    block_on(async {
        let fixture = Fixture::open("/paper", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#press").await.unwrap();

        let look: (Option<String>, String, String, String) = page
            .evaluate(
                "(() => { const e = document.querySelector('#press'); const s = getComputedStyle(e); \
                 return [e.getAttribute('type'), s.borderTopWidth, s.paddingLeft, s.textAlign]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(look.0.as_deref(), Some("button"));
        assert_eq!(
            (look.1.as_str(), look.2.as_str()),
            ("0px", "0px"),
            "{look:?}"
        );
        assert_ne!(look.3, "center", "{look:?}");

        let bordered: String = page
            .evaluate("getComputedStyle(document.querySelector('#press-bordered')).borderTopWidth")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(bordered, "1px", "bordered still wins the reset");
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
        let (style, width, color): (String, f64, String) = page
            .evaluate(
                "(() => { const s = getComputedStyle(document.querySelector('#link')); \
                 return [s.outlineStyle, parseFloat(s.outlineWidth), s.outlineColor]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(style, "solid");
        assert!(width > 0.0, "the ring is {width}px wide");
        assert_ne!(color, "rgba(0, 0, 0, 0)", "the ring is transparent");
        fixture.close().await.unwrap();
    });
}
