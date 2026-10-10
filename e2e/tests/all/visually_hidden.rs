//! `VisuallyHidden { focusable }`: hidden until focus lands inside, then shown
//! (todo 613, 2.4.7).

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("visually_hidden", "/visually-hidden")
        .focusable("#skip-link")
        .run();
}

const WIDTH: &str = "document.querySelector('#skip').getBoundingClientRect().width";

#[test]
fn a_focusable_skip_link_shows_on_focus() {
    block_on(async {
        let fixture = Fixture::open("/visually-hidden", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let hidden: f64 = page.evaluate(WIDTH).await.unwrap().into_value().unwrap();
        assert!(hidden <= 1.0, "hidden until focused, got {hidden}px");

        keyboard::tab_to(page, "#skip-link", 3).await.unwrap();
        wait::for_js_true(page, &format!("{WIDTH} > 20"), "the link to show on focus")
            .await
            .unwrap();

        page.evaluate("document.activeElement.blur()")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("{WIDTH} <= 1"), "the link to hide again")
            .await
            .unwrap();

        fixture.console.assert_clean("the skip link").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_focused_skip_link_sits_above_a_sticky_header() {
    block_on(async {
        let fixture = Fixture::open("/visually-hidden", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#skip-link", 3).await.unwrap();
        wait::for_js_true(page, &format!("{WIDTH} > 20"), "the link to show on focus")
            .await
            .unwrap();

        let on_top: bool = page
            .evaluate(
                "(() => { const r = document.querySelector('#skip-link').getBoundingClientRect();
                    return document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2)
                        === document.querySelector('#skip-link'); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(on_top, "the Header covers the focused skip link");

        fixture.close().await.unwrap();
    });
}

#[test]
fn enter_on_the_skip_link_moves_focus_to_the_page() {
    block_on(async {
        let fixture = Fixture::open("/visually-hidden", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#skip-link", 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement === document.querySelector('#page')",
            "focus to reach the page",
        )
        .await
        .unwrap();

        fixture.close().await.unwrap();
    });
}
