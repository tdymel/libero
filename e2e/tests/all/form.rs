//! `Form`: a bound field's rule shows on blur and follows the form's value.
//!
//! Rules that capture nothing compare equal, so the field skips its parent's
//! render (todo 29); it must still redraw on a keystroke and on a store write.

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
