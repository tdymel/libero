//! Controlled fields: the control and slot redraw without the shell (29), and must still
//! show what the caller holds, keystroke by keystroke.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const TEXT: &str = "[data-case=text] input";
const NUMBER: &str = "[data-case=number] input";
const NATIVE: &str = "[data-case=native] select";
const PASSWORD: &str = "[data-case=password] input";
const REVEAL: &str = "[data-case=password] button";

#[test]
fn every_keystroke_and_pick_reaches_the_control_and_its_slot() {
    block_on(async {
        let fixture = Fixture::open("/field-value", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, TEXT).await.unwrap();

        focus(page, TEXT).await.unwrap();
        keyboard::type_text(page, "abc").await.unwrap();
        settle(
            page,
            &format!("{} && {}", value_is(TEXT, "abc"), echo_is("text", "3")),
            "the text and its slot to follow the keystrokes",
        )
        .await
        .unwrap();

        focus(page, NUMBER).await.unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        settle(
            page,
            &format!("{} && {}", value_is(NUMBER, "4"), echo_is("number", "4")),
            "the arrow to step the number",
        )
        .await
        .unwrap();
        keyboard::type_text(page, "2").await.unwrap();
        settle(
            page,
            &format!(
                "{} && {} && document.querySelector({NUMBER:?}).getAttribute('aria-valuenow') === '42'",
                value_is(NUMBER, "42"),
                echo_is("number", "42")
            ),
            "the typed digit to land on the number",
        )
        .await
        .unwrap();

        page.evaluate(format!(
            "(() => {{ const s = document.querySelector({NATIVE:?}); s.value = 'Cherry'; \
             s.dispatchEvent(new Event('change', {{ bubbles: true }})); }})()"
        ))
        .await
        .unwrap();
        settle(
            page,
            &format!(
                "{} && document.querySelector('{NATIVE} option[value=Cherry]').selected \
                 && !document.querySelector('{NATIVE} option[value=Banana]').selected",
                echo_is("native", "Cherry")
            ),
            "the pick to move the selected option",
        )
        .await
        .unwrap();

        focus(page, PASSWORD).await.unwrap();
        keyboard::type_text(page, "xy").await.unwrap();
        settle(
            page,
            &value_is(PASSWORD, "xy"),
            "the password to take the keystrokes",
        )
        .await
        .unwrap();
        page.evaluate(format!("document.querySelector({REVEAL:?}).click()"))
            .await
            .unwrap();
        settle(
            page,
            &format!(
                "{} && document.querySelector({PASSWORD:?}).type === 'text'",
                value_is(PASSWORD, "xy")
            ),
            "the reveal button to show the text it kept",
        )
        .await
        .unwrap();
        focus(page, PASSWORD).await.unwrap();
        keyboard::type_text(page, "z").await.unwrap();
        settle(
            page,
            &value_is(PASSWORD, "xyz"),
            "a revealed password to keep typing",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("typing and picking in controlled fields")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Focus with the caret at the end, as a click after the text would leave it.
async fn focus(page: &Page, selector: &str) -> Result<()> {
    page.evaluate(format!(
        "(() => {{ const el = document.querySelector({selector:?}); el.focus(); \
         const end = el.value.length; el.setSelectionRange(end, end); }})()"
    ))
    .await?;
    Ok(())
}

fn value_is(selector: &str, value: &str) -> String {
    format!("document.querySelector({selector:?}).value === {value:?}")
}

fn echo_is(case: &str, value: &str) -> String {
    format!("document.querySelector('[data-echo={case}]').textContent === {value:?}")
}

async fn settle(page: &Page, condition: &str, what: &str) -> Result<()> {
    wait::for_js_true(page, &format!("(() => {condition})()"), what).await
}
