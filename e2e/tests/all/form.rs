//! `Form`: a bound field's rule shows on blur and follows the form's value. The field skips
//! its parent's render (29), yet must redraw on a keystroke and a store write.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::Scheme;
use e2e::browser::block_on;
use e2e::passes::{contrast, focus, keyboard, pointer};
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

/// A blocked submit hands focus to the summary, axe finds nothing on it in either
/// scheme, and Tab then Enter on a line focuses its field.
#[test]
fn a_blocked_submit_focuses_the_summary_and_its_lines_reach_the_fields() {
    block_on(async {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let fixture = Fixture::open_in("/form/summary", Viewport::Mobile, scheme)
                .await
                .unwrap();
            let page = &fixture.page;
            pointer::click(page, "input[type=email]").await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            focus::wait_for_focus(page, "[data-slot=summary]", "a blocked submit")
                .await
                .unwrap();
            contrast::assert_clean(page, "form").await.unwrap();

            keyboard::tab_to(page, "[data-slot=summary] a", 2)
                .await
                .unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            focus::wait_for_focus(page, "input[type=email]", "Enter on its line")
                .await
                .unwrap();

            fixture.console.assert_clean("the error summary").unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 1272: a summary line reads as a link by its underline, not by colour alone, also in
/// forced colours (GOV.UK's summary).
#[test]
fn the_summary_lines_are_underlined_also_in_forced_colours() {
    const UNDERLINED: &str = "(() => { const links = [...document.querySelectorAll('[data-slot=summary] a')]; \
        return links.length > 0 && links.every(a => getComputedStyle(a).textDecorationLine.includes('underline')); })()";
    block_on(async {
        let fixture = Fixture::open("/form/summary", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "input[type=email]").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        focus::wait_for_focus(page, "[data-slot=summary]", "a blocked submit")
            .await
            .unwrap();
        wait::for_js_true(page, UNDERLINED, "underlined summary lines")
            .await
            .unwrap();
        e2e::browser::force_colours(page).await.unwrap();
        wait::for_js_true(page, UNDERLINED, "underlined lines in forced colours")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A group field has no single control to carry the id its summary line links
/// to: the line must still focus the group's tab stop. Every validated field type (1274).
/// A field named by `aria_label` only is named in its line (1526). A fieldset in error has a
/// line, which focuses a control inside it (1273).
#[test]
fn every_summary_line_focuses_something_in_its_field() {
    block_on(async {
        let fixture = Fixture::open("/form/targets", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        // Below the fold with every field type: a click off screen hits nothing.
        page.evaluate("document.querySelector('button[type=submit]').scrollIntoView()")
            .await
            .unwrap();
        pointer::click(page, "button[type=submit]").await.unwrap();
        focus::wait_for_focus(page, "[data-slot=summary]", "a blocked submit")
            .await
            .unwrap();
        let lines: usize = page
            .evaluate("document.querySelectorAll('[data-slot=summary] a').length")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(lines, 21);
        // A field named by `aria_label` only still names its line (1526).
        let texts: Vec<String> = page
            .evaluate(
                "[...document.querySelectorAll('[data-slot=summary] a')].map((a) => a.textContent)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        for line in [
            "Newsletter: Tick the box.",
            "Nickname: Enter a nickname.",
            "Address: Address not found.",
        ] {
            assert!(
                texts.iter().any(|text| text == line),
                "no {line:?} in {texts:?}"
            );
        }
        page.evaluate("window.__landed = new Set()").await.unwrap();
        for line in 0..lines {
            page.evaluate(format!(
                "document.querySelectorAll('[data-slot=summary] a')[{line}].click()"
            ))
            .await
            .unwrap();
            // A control no earlier line reached: each line finds its own field.
            wait::for_js_true(
                page,
                "(() => { const a = document.activeElement; \
                 if (a === document.body || a.closest('[data-slot=summary]') \
                     || window.__landed.has(a)) return false; \
                 window.__landed.add(a); return true; })()",
                &format!("summary line {line} to focus its field"),
            )
            .await
            .unwrap();
            // Back on the summary, so a line that focuses nothing shows.
            page.evaluate("document.querySelector('[data-slot=summary]').focus()")
                .await
                .unwrap();
        }
        fixture.console.assert_clean("the summary lines").unwrap();
        fixture.close().await.unwrap();
    });
}

const DUE: &str = "input[data-e2e=due]";

/// Tags the date field's text input as `DUE`.
async fn tag_due(page: &Page) -> Result<()> {
    page.evaluate(
        "[...document.querySelectorAll('label')].find(l => l.textContent.trim() === 'Due') \
         .control.setAttribute('data-e2e', 'due')",
    )
    .await?;
    Ok(())
}

/// Todo 2305: Enter both commits the typed date and submits; the submit sees the date.
#[test]
fn enter_in_a_date_field_submits_the_committed_date() {
    block_on(async {
        let fixture = Fixture::open("/form/targets", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        tag_due(page).await.unwrap();
        pointer::click(page, DUE).await.unwrap();
        keyboard::type_text(page, "March 5, 2026").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        focus::wait_for_focus(page, "[data-slot=summary]", "a blocked submit")
            .await
            .unwrap();
        let texts: Vec<String> = page
            .evaluate(
                "[...document.querySelectorAll('[data-slot=summary] a')].map((a) => a.textContent)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(texts.len(), 20, "{texts:?}");
        assert!(
            !texts.iter().any(|text| text.starts_with("Due")),
            "{texts:?}"
        );
        fixture
            .console
            .assert_clean("Enter in a date field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2304: a form reset drops a date field's refused text and its error.
#[test]
fn a_reset_drops_a_refused_date() {
    block_on(async {
        let fixture = Fixture::open("/form/targets", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        tag_due(page).await.unwrap();
        pointer::click(page, DUE).await.unwrap();
        keyboard::type_text(page, "nope").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({DUE:?}).getAttribute('aria-invalid') === 'true'"),
            "the refused date",
        )
        .await
        .unwrap();
        page.evaluate("document.querySelector('button[type=reset]').click()")
            .await
            .unwrap();
        let cleared = wait::for_js_true(
            page,
            &format!(
                "(() => {{ const input = document.querySelector({DUE:?}); \
                 const ids = (input.getAttribute('aria-describedby') || '').split(' '); \
                 return input.value === '' \
                   && !ids.some(id => document.getElementById(id)?.textContent.includes('Not a valid date')); }})()"
            ),
            // A native reset keeps the touched flag, so the field's own rule may show.
            "the reset to clear the text and the refusal",
        )
        .await;
        if let Err(error) = cleared {
            let state: String = page
                .evaluate(format!(
                    "(() => {{ const input = document.querySelector({DUE:?}); \
                     const ids = (input.getAttribute('aria-describedby') || '').split(' '); \
                     return JSON.stringify([input.value, \
                       ids.map(id => document.getElementById(id)?.textContent)]); }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            panic!("{error}: [value, descriptions] {state}");
        }
        fixture.console.assert_clean("a reset date field").unwrap();
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
