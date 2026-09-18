//! `use_focus_return`: the panel from its docs page hands focus back.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, ENTER, ESCAPE, TAB};
use e2e::{Fixture, Viewport, wait};

async fn focused(page: &chromiumoxide::Page) -> String {
    page.evaluate("document.activeElement?.id ?? ''")
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Closing from inside, by Apply and then by Escape, lands on the trigger each
/// time: the second open re-arms what the first `restore()` consumed.
#[test]
fn closing_the_panel_returns_focus_to_its_trigger() {
    block_on(async {
        let fixture = Fixture::open("/focus-return", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#filters").await.unwrap();
        page.evaluate("document.querySelector('#filters').focus()")
            .await
            .unwrap();

        keyboard::press(page, ENTER).await.unwrap();
        wait::for_visible(page, "#apply").await.unwrap();
        keyboard::press(page, TAB).await.unwrap();
        keyboard::press(page, TAB).await.unwrap();
        assert_eq!(focused(page).await, "apply", "Tab reaches Apply");
        keyboard::press(page, ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.id === 'filters'",
            "focus back on the trigger after Apply",
        )
        .await
        .unwrap();

        keyboard::press(page, ENTER).await.unwrap();
        wait::for_visible(page, "#apply").await.unwrap();
        keyboard::press(page, TAB).await.unwrap();
        keyboard::press(page, ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#apply') && document.activeElement?.id === 'filters'",
            "focus back on the trigger after Escape",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("the focus-return panel")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A trigger named once from `onmounted` stays armed: the second close lands on
/// it too, with no `remember` call in between.
#[test]
fn a_trigger_remembered_on_mount_takes_focus_on_every_close() {
    block_on(async {
        let fixture = Fixture::open("/focus-return/mounted", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#trigger").await.unwrap();

        for round in ["first", "second"] {
            page.evaluate("document.querySelector('#trigger').focus()")
                .await
                .unwrap();
            keyboard::press(page, ENTER).await.unwrap();
            wait::for_visible(page, "#close").await.unwrap();
            keyboard::press(page, TAB).await.unwrap();
            assert_eq!(focused(page).await, "close", "Tab reaches Close ({round})");
            keyboard::press(page, ENTER).await.unwrap();
            wait::for_js_true(
                page,
                "!document.querySelector('#close') && document.activeElement?.id === 'trigger'",
                &format!("focus back on the trigger after the {round} close"),
            )
            .await
            .unwrap();
        }

        fixture
            .console
            .assert_clean("the remember-on-mount panel")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
