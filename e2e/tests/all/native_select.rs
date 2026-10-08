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

/// Todos 581 and 582: an untouched required select is required, not invalid,
/// and its placeholder is dimmed until a pick while the options keep ink.
#[test]
fn an_empty_required_select_is_quiet_and_dimmed() {
    block_on(async {
        let fixture = Fixture::open("/native-select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // Chromium reports no `required` for a `<select>` either way, so the
        // markup says it.
        let snapshot = ax::snapshot(page, "[data-case=wired]").await.unwrap();
        assert!(!snapshot.contains("[invalid]"), "{snapshot}");
        let markup = format!(
            "(() => {{ const s = document.querySelector({WIRED:?}); \
             return s.getAttribute('aria-required') === 'true' && !s.required; }})()"
        );
        wait::for_js_true(page, &markup, "aria-required without native required")
            .await
            .unwrap();

        let color = |selector: &str| {
            format!("getComputedStyle(document.querySelector({selector:?})).color")
        };
        let ink = color("[data-case=refused] select");
        let dimmed = format!(
            "{} !== {ink} && {} === {ink}",
            color(WIRED),
            color("[data-case=wired] option[value=Apple]")
        );
        wait::for_js_true(page, &dimmed, "a dimmed placeholder over ink options")
            .await
            .unwrap();

        keyboard::tab_to(page, WIRED, 8).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        let picked = format!("{} === {ink}", color(WIRED));
        wait::for_js_true(page, &picked, "ink once Apple is picked")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("an empty required select")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2574: a disabled select shows the not-allowed cursor, an enabled one the pointer.
#[test]
fn a_disabled_select_shows_the_not_allowed_cursor() {
    block_on(async {
        let fixture = Fixture::open("/native-select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let cursor = |case: &str| {
            format!("getComputedStyle(document.querySelector('[data-case={case}] select')).cursor")
        };
        let check = format!(
            "{} === 'not-allowed' && {} === 'pointer'",
            cursor("disabled"),
            cursor("refused")
        );
        wait::for_js_true(page, &check, "not-allowed on the disabled select only")
            .await
            .unwrap();

        fixture.close().await.unwrap();
    });
}

/// Todo 2575: a value the pending options do not hold yet draws the dimmed placeholder,
/// and the select takes ink once the option arrives.
#[test]
fn a_value_outside_the_options_shows_the_placeholder_until_it_arrives() {
    block_on(async {
        let fixture = Fixture::open("/native-select/pending", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let select = |case: &str| format!("document.querySelector('[data-case={case}] select')");
        let pending = select("pending");
        let color = |element: &str| format!("getComputedStyle({element}).color");
        let dimmed = format!(
            "{pending}.hasAttribute('data-placeholder') && {} !== {}",
            color(&pending),
            color(&select("plain"))
        );
        wait::for_js_true(page, &dimmed, "the dimmed placeholder")
            .await
            .unwrap();

        pointer::click(page, "#load").await.unwrap();
        let arrived = format!(
            "!{pending}.hasAttribute('data-placeholder') && {pending}.value === 'Banana' \
             && {} === {}",
            color(&pending),
            color(&select("plain"))
        );
        wait::for_js_true(page, &arrived, "ink once Banana is offered")
            .await
            .unwrap();

        fixture.console.assert_clean("a pending select").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 583: named groups are `<optgroup>`s, a disabled option is announced
/// disabled and the arrows step over it.
#[test]
fn groups_and_a_disabled_option_reach_assistive_technology() {
    block_on(async {
        let fixture = Fixture::open("/native-select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        const GROUPED: &str = "[data-case=grouped] select";

        let snapshot = ax::snapshot(page, "[data-case=grouped]").await.unwrap();
        assert!(snapshot.contains(r#"group "Stone""#), "{snapshot}");
        assert!(
            snapshot.contains(r#"option "Cherry" [disabled]"#),
            "{snapshot}"
        );
        assert!(!snapshot.contains(r#"option "Banana""#), "{snapshot}");

        keyboard::tab_to(page, GROUPED, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({GROUPED:?}).value === 'Damson'"),
            "ArrowDown to skip Cherry",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("grouped options").unwrap();
        fixture.close().await.unwrap();
    });
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
