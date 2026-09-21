//! `Form`: a bound field's rule shows on blur and follows the form's value. The field skips
//! its parent's render (29), yet must redraw on a keystroke and a store write.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

const INPUT: &str = "input[name=email]";

#[test]
fn a_bound_rule_follows_the_value_through_a_parent_render() {
    block_on(async {
        let fixture = Fixture::open("/form", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 3).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        settle(page, true).await.unwrap();

        pointer::click(page, INPUT).await.unwrap();
        keyboard::type_text(page, "a").await.unwrap();
        settle(page, false).await.unwrap();

        click_button(page, "Rerender").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[data-renders]').dataset.renders === '1'",
            "the page to re-render",
        )
        .await
        .unwrap();
        settle(page, false).await.unwrap();

        click_button(page, "Clear").await.unwrap();
        settle(page, true).await.unwrap();

        fixture
            .console
            .assert_clean("validating a bound field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The web's own submit and reset, which Blitz emulates: both triggers run
/// `onsubmit` once, and a reset empties the bound field.
#[test]
fn a_submit_button_and_the_handle_submit_once_each_and_reset_clears() {
    block_on(async {
        let fixture = Fixture::open("/form", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        pointer::click(page, INPUT).await.unwrap();
        keyboard::type_text(page, "a").await.unwrap();
        click_button(page, "Send").await.unwrap();
        submits(page, 1).await.unwrap();
        click_button(page, "Submit").await.unwrap();
        submits(page, 2).await.unwrap();

        click_button(page, "Reset").await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({INPUT:?}).value === ''"),
            "the reset to empty the field",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("submitting a form").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn submits(page: &Page, count: u32) -> Result<()> {
    wait::for_js_true(
        page,
        &format!("document.querySelector('[data-submits]').dataset.submits === '{count}'"),
        &format!("{count} submits"),
    )
    .await
}

async fn click_button(page: &Page, text: &str) -> Result<()> {
    page.evaluate(format!(
        "[...document.querySelectorAll('button')].find(b => b.textContent.trim() === {text:?}).setAttribute('data-e2e', {text:?})"
    ))
    .await?;
    pointer::click(page, &format!("button[data-e2e={text}]")).await
}

/// The field is invalid and shows its message, or neither.
async fn settle(page: &Page, invalid: bool) -> Result<()> {
    wait::for_js_true(
        page,
        &format!(
            "(() => {{ const el = document.querySelector({INPUT:?}); \
             const shown = document.body.textContent.includes('Email needed'); \
             return (el.getAttribute('aria-invalid') === 'true') === {invalid} && shown === {invalid}; }})()"
        ),
        &format!("the field to be invalid: {invalid}"),
    )
    .await
}
