//! `NativeSelect` and `Textarea`: what assistive technology hears of the label,
//! the captions and the status, and the placeholder coming back.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, ax, wait};

const WIRED: &str = "[data-case=wired] select";

#[test]
fn it_meets_the_baseline() {
    Suite::new("native_select", "/native-select")
        .focusable(WIRED)
        .focusable("[data-case=textarea] textarea")
        .focusable("[data-case=textarea-readonly] textarea")
        // 21px tall, but the frame's padding forwards a click to it.
        .targets_spaced("select")
        .run();
}

/// Description and helper describe the select, the rule shows on blur and
/// goes with a pick, and `None` brings the placeholder back.
#[test]
fn the_captions_the_rule_and_the_placeholder_reach_assistive_technology() {
    block_on(async {
        let fixture = Fixture::open("/native-select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let state = |value: &'static str, invalid: bool, what: &'static str| async move {
            let check = format!(
                "(() => {{ const s = document.querySelector({WIRED:?}); \
                 return s.value === {value:?} \
                 && (s.getAttribute('aria-invalid') === 'true') === {invalid}; }})()"
            );
            wait::for_js_true(page, &check, what).await.unwrap();
        };

        assert_eq!(
            ax::description(page, WIRED).await.unwrap(),
            "For the smoothie. One per order."
        );
        assert_eq!(
            ax::description(page, "[data-case=textarea] textarea")
                .await
                .unwrap(),
            "Anything the team should know. Markdown is not rendered."
        );

        keyboard::tab_to(page, WIRED, 8).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        state("", true, "the rule on blur").await;
        assert_eq!(
            ax::description(page, WIRED).await.unwrap(),
            "For the smoothie. One per order. Fruit needed"
        );

        keyboard::tab_to(page, WIRED, 8).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        state("Apple", false, "ArrowDown to pick Apple").await;
        wait::for_js_true(
            page,
            "document.querySelector('[data-echo=wired]').textContent === 'Apple'",
            "the caller to hold Apple",
        )
        .await
        .unwrap();

        pointer::click(page, "#clear").await.unwrap();
        // Touched already, so the rule fails again at once.
        state("", true, "the placeholder back on None").await;
        let snapshot = ax::snapshot(page, "[data-case=wired]").await.unwrap();
        assert!(
            snapshot.contains(r#"combobox "Fruit" = Pick one"#),
            "{snapshot}"
        );

        fixture
            .console
            .assert_clean("describing a native select")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
