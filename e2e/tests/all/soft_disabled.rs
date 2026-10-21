//! Todo 3838: a focused button-like control that turns disabled keeps its focus and
//! swallows activation; a native `disabled` dropped the focus to `<body>`.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

const CONTROLS: [&str; 7] = [
    "#button",
    "#icon",
    "#burger",
    "#chip",
    "#nav [data-slot='row'] > button",
    "#password button",
    "#item",
];

#[test]
fn a_control_that_becomes_disabled_keeps_its_focus() {
    block_on(async {
        let fixture = Fixture::open("/soft-disabled", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#item").await.unwrap();
        // `click()` by script: the flip moves no focus.
        let flip = "document.querySelector('#flip').click()";

        for control in CONTROLS {
            page.evaluate_bounded(format!("document.querySelector(\"{control}\").focus()"))
                .await
                .unwrap();
            page.evaluate_bounded(flip).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const c = document.querySelector(\"{control}\"); \
                     return document.activeElement === c && !c.disabled \
                         && c.getAttribute('aria-disabled') === 'true' && c.tabIndex === -1; }})()"
                ),
                &format!("focus kept on the soft-disabled {control}"),
            )
            .await
            .unwrap();

            keyboard::press(page, keyboard::ENTER).await.unwrap();
            keyboard::press(page, keyboard::SPACE).await.unwrap();
            page.evaluate_bounded(flip).await.unwrap();
            wait::for_js_true(
                page,
                &format!("!document.querySelector(\"{control}\").hasAttribute('aria-disabled')"),
                &format!("{control} enabled again"),
            )
            .await
            .unwrap();
            let swallowed: bool = page
                .evaluate_bounded(
                    "document.querySelector('#clicks').dataset.clicks === '0' \
                     && document.querySelector('#password input').type === 'password'",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(swallowed, "the disabled {control} still acted on a key");
        }

        fixture
            .console
            .assert_clean("soft-disabled controls")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
