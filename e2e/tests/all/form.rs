//! `Form`: a bound field's rule shows on blur and follows the form's value. The field skips
//! its parent's render (29), yet must redraw on a keystroke and a store write.

use anyhow::{Result, ensure};
use chromiumoxide::Page;
use e2e::browser::Scheme;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::{contrast, focus, keyboard, pointer};
use e2e::wait::PageBounded;
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
        page.evaluate_bounded("document.querySelector('button[type=submit]').scrollIntoView()")
            .await
            .unwrap();
        pointer::click(page, "button[type=submit]").await.unwrap();
        focus::wait_for_focus(page, "[data-slot=summary]", "a blocked submit")
            .await
            .unwrap();
        let lines: usize = page
            .evaluate_bounded("document.querySelectorAll('[data-slot=summary] a').length")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(lines, 21);
        // A field named by `aria_label` only still names its line (1526).
        let texts: Vec<String> = page
            .evaluate_bounded(
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
        page.evaluate_bounded("window.__landed = new Set()")
            .await
            .unwrap();
        for line in 0..lines {
            page.evaluate_bounded(format!(
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
            page.evaluate_bounded("document.querySelector('[data-slot=summary]').focus()")
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
    page.evaluate_bounded(
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
            .evaluate_bounded(
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
        page.evaluate_bounded("document.querySelector('button[type=reset]').click()")
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
            "the reset to clear the text and the refusal",
        )
        .await;
        if let Err(error) = cleared {
            let state: String = page
                .evaluate_bounded(format!(
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

/// Todo 2328: a native reset takes the form back to pristine, as `FormHandle::reset`.
#[test]
fn a_native_reset_clears_the_summary_and_the_shown_errors() {
    block_on(async {
        let fixture = Fixture::open("/form/targets", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded("document.querySelector('button[type=submit]').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "!!document.querySelector('[data-slot=summary]') \
             && !!document.querySelector('[aria-invalid=true]')",
            "a blocked submit to show its summary and errors",
        )
        .await
        .unwrap();
        page.evaluate_bounded("document.querySelector('button[type=reset]').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[data-slot=summary]') \
             && !document.querySelector('[aria-invalid=true]')",
            "the reset to clear the summary and the errors",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a native form reset").unwrap();
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
    page.evaluate_bounded(format!(
        "[...document.querySelectorAll('button')].find(b => b.textContent.trim() === {text:?}).setAttribute('data-e2e', {text:?})"
    ))
    .await?;
    pointer::click(page, &format!("button[data-e2e={text:?}]")).await
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

const SUMMARY: &str = "[data-slot=summary]";

/// Todo 2572: `required` alone cancels an empty submit before `onsubmit`, with the summary as a
/// rule's error has it; the button stays enabled, and a filled form submits.
async fn required_blocks_an_empty_submit<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        d.attr("#send", "disabled").await?.is_none(),
        "the submit button is disabled"
    );
    ensure!(
        !d.text("body").await?.contains("Fill in this field."),
        "the required error showed before the submit"
    );
    d.click("#send").await?;
    eventually(d, "the error summary", async |d| d.exists(SUMMARY).await).await?;
    let summary = d.text(SUMMARY).await?;
    for label in ["Name", "Plan", "Tier"] {
        ensure!(
            summary.contains(label),
            "no {label} line in the summary: {summary}"
        );
    }
    // Todo 2647: a select asks in its own words.
    ensure!(
        summary.matches("Fill in this field.").count() == 1
            && summary.matches("Select an item in the list.").count() == 2,
        "not one fill and two select lines: {summary}"
    );
    ensure!(
        d.text("#submits").await? == "0",
        "onsubmit ran on an empty submit"
    );
    ensure!(
        !summary.contains("Note"),
        "the read-only field is in the summary: {summary}"
    );
    for field in ["#name", "#plan"] {
        ensure!(
            d.attr(field, "aria-invalid").await?.as_deref() == Some("true"),
            "{field} is not invalid"
        );
    }
    d.click("#fill").await?;
    d.click("#send").await?;
    eventually_text(d, "#submits", "1", "a filled submit").await
}

e2e::scenario!(
    required_fields_block_an_empty_submit,
    "/form/required",
    required_blocks_an_empty_submit
);

/// Todo 4091: a `ColorField` always holds a color, so `required` fails only on text cleared and
/// kept (`fix_on_blur: false`); the cleared field blocks the submit and names itself.
async fn a_cleared_color_blocks_the_submit<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#fill").await?;
    d.click("#colour").await?;
    d.press(keyboard::END).await?;
    for _ in 0..10 {
        d.press(keyboard::BACKSPACE).await?;
    }
    eventually(d, "the colour text cleared", async |d| {
        Ok(d.value("#colour").await?.is_empty())
    })
    .await?;
    // The click opened the dropdown over `#send`.
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the colour dropdown to close", async |d| {
        Ok(d.attr("#colour", "aria-expanded").await?.as_deref() == Some("false"))
    })
    .await?;
    // Blank text is empty, not invalid (4242): like any required field, it says so at the submit.
    d.press(keyboard::TAB).await?;
    ensure!(
        d.attr("#colour", "aria-invalid").await?.as_deref() != Some("true"),
        "the cleared colour is invalid before the submit"
    );
    d.click("#send").await?;
    eventually(d, "the submit to land", async |d| {
        Ok(d.exists(SUMMARY).await? || d.text("#submits").await? != "0")
    })
    .await?;
    ensure!(
        d.text("#submits").await? == "0",
        "onsubmit ran with the colour cleared to {:?}, aria-invalid {:?}",
        d.value("#colour").await?,
        d.attr("#colour", "aria-invalid").await?
    );
    let summary = d.text(SUMMARY).await?;
    ensure!(
        summary.contains("Colour: Fill in this field."),
        "no required Colour line in the summary: {summary}"
    );
    ensure!(
        d.attr("#colour", "aria-invalid").await?.as_deref() == Some("true"),
        "the cleared colour is not invalid after the submit"
    );
    Ok(())
}

e2e::scenario!(
    a_cleared_required_color_blocks_the_submit,
    "/form/required",
    a_cleared_color_blocks_the_submit
);

/// Todo 2573: a status set in `onsubmit` after a valid submit shows the summary and focuses it.
async fn a_status_in_onsubmit_shows_the_summary<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#send").await?;
    eventually_text(d, "#submits", "1", "a valid submit").await?;
    eventually_focused(d, SUMMARY, "the server's status").await?;
    let summary = d.text(SUMMARY).await?;
    ensure!(
        summary.contains("Username: Taken, submit 1."),
        "no Username line in the summary: {summary}"
    );
    Ok(())
}

e2e::scenario!(
    a_status_set_in_onsubmit_shows_and_focuses_the_summary,
    "/form/server-sync",
    a_status_in_onsubmit_shows_the_summary
);

/// Todo 2573: a status that arrives later shows one summary with the latest text, after two
/// submits too; a rule broken after the submit shows none. A reset stops the wait.
async fn a_late_status_shows_the_summary<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#send").await?;
    eventually_text(d, "#submits", "1", "the first submit").await?;
    d.click("#send").await?;
    eventually_text(d, "#submits", "2", "the second submit").await?;
    d.click("#break").await?;
    eventually(d, "the broken code's own error", async |d| {
        Ok(d.attr("#code", "aria-invalid").await?.as_deref() == Some("true"))
    })
    .await?;
    d.flush().await?;
    ensure!(
        !d.exists(SUMMARY).await?,
        "a rule broken after a valid submit showed the summary"
    );
    d.click("#answer").await?;
    eventually_focused(d, SUMMARY, "the late status").await?;
    let summary = d.text(SUMMARY).await?;
    ensure!(
        summary.matches("Username: Taken, submit 2.").count() == 1 && !summary.contains("submit 1"),
        "not one line with the latest status: {summary}"
    );
    Ok(())
}

e2e::scenario!(
    a_late_status_after_a_valid_submit_shows_and_focuses_the_summary,
    "/form/server",
    a_late_status_shows_the_summary
);

async fn a_reset_stops_the_wait<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#send").await?;
    eventually_text(d, "#submits", "1", "a valid submit").await?;
    d.click("#reset").await?;
    d.click("#answer").await?;
    eventually(d, "the status under the field", async |d| {
        Ok(d.attr("#name", "aria-invalid").await?.as_deref() == Some("true"))
    })
    .await?;
    d.flush().await?;
    ensure!(
        !d.exists(SUMMARY).await?,
        "a status after a reset showed the summary"
    );
    Ok(())
}

e2e::scenario!(
    a_status_after_a_reset_shows_no_summary,
    "/form/server",
    a_reset_stops_the_wait
);

/// Todo 2608: a disabled `Fieldset` dims its legend, description and helper to `muted.6`.
async fn a_disabled_fieldset_dims_its_text<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let muted = d.style("#probe", "color").await?;
    for part in [
        "#group > legend",
        "#group > [data-slot=description]",
        "#group > [data-slot=helper]",
    ] {
        let color = d.style(part, "color").await?;
        ensure!(color == muted, "{part} is {color}, not muted.6's {muted}");
    }
    Ok(())
}

e2e::scenario!(
    a_disabled_fieldset_dims_its_legend_and_captions,
    "/form/fieldset-disabled",
    a_disabled_fieldset_dims_its_text
);

/// Todo 3873: the group renders no native `disabled`, so a real submit must still
/// send nothing from inside it; only the field outside it is sent.
#[test]
fn a_disabled_fieldset_sends_no_field_value() {
    block_on(async {
        let fixture = Fixture::open("/form/fieldset-submit", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#submit").await.unwrap();
        page.evaluate_bounded(
            "document.querySelector('#sent').addEventListener('submit', (event) => { \
             event.preventDefault(); \
             window.sent = [...new FormData(event.target, event.submitter).keys()]; })",
        )
        .await
        .unwrap();
        pointer::click(page, "#submit").await.unwrap();
        wait::for_js_true(page, "Array.isArray(window.sent)", "the submit")
            .await
            .unwrap();
        let sent: Vec<String> = page
            .evaluate_bounded("window.sent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            sent,
            ["outside"],
            "fields of the disabled fieldset were sent"
        );
        fixture.close().await.unwrap();
    });
}

/// Concept domain-values, fix 1: a mapped field keeps the text as typed, the store holding the
/// parsed value, until a blur reformats it; invalid text stays, empties the value and blocks.
#[test]
fn a_mapped_field_keeps_its_typed_text_until_a_blur() {
    block_on(async {
        let fixture = Fixture::open("/form/domain", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let iban = "input[name=iban]";
        let value_is = |text: &str| format!("document.querySelector({iban:?}).value === {text:?}");
        let stored_is = |text: &str| {
            format!("document.querySelector('[data-stored]').dataset.stored === {text:?}")
        };

        pointer::click(page, iban).await.unwrap();
        keyboard::type_text(page, "de89370400440532013000")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &stored_is("DE89370400440532013000"),
            "the parsed value",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &value_is("de89370400440532013000"),
            "the text as typed",
        )
        .await
        .unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &value_is("DE89 3704 0044 0532 0130 00"),
            "the blur to reformat",
        )
        .await
        .unwrap();

        pointer::click(page, iban).await.unwrap();
        keyboard::press(page, keyboard::END).await.unwrap();
        for _ in 0..5 {
            keyboard::press(page, keyboard::BACKSPACE).await.unwrap();
        }
        wait::for_js_true(page, &stored_is("none"), "invalid text to empty the value")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &value_is("DE89 3704 0044 0532 01"),
            "the invalid text to stay",
        )
        .await
        .unwrap();

        click_button(page, "Send").await.unwrap();
        wait::for_js_true(
            page,
            "document.body.textContent.includes('Not an IBAN.')",
            "the parse error",
        )
        .await
        .unwrap();
        submits(page, 0).await.unwrap();
        wait::for_js_true(
            page,
            &value_is("DE89 3704 0044 0532 01"),
            "the text to outlive the submit",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("typing into a mapped field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Concept domain-values, fix 1, on a field with its own text buffer: a `NumberField` mapped to a
/// share keeps "150" as typed while the share it cannot be resets the value.
#[test]
fn a_mapped_number_field_keeps_invalid_text() {
    block_on(async {
        let fixture = Fixture::open("/form/domain", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let share = "input[name=share]";
        let value_is = |text: &str| format!("document.querySelector({share:?}).value === {text:?}");
        let stored_is = |text: &str| {
            format!("document.querySelector('[data-share]').dataset.share === {text:?}")
        };

        pointer::click(page, share).await.unwrap();
        keyboard::type_text(page, "15").await.unwrap();
        wait::for_js_true(page, &stored_is("15"), "the parsed share")
            .await
            .unwrap();
        keyboard::type_text(page, "0").await.unwrap();
        wait::for_js_true(page, &stored_is("none"), "150 to empty the value")
            .await
            .unwrap();
        wait::for_js_true(page, &value_is("150"), "the text as typed")
            .await
            .unwrap();

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(page, &value_is("150"), "the invalid text to outlive a blur")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.body.textContent.includes('Not an IBAN.')",
            "the parse error",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("typing into a mapped number field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4090: a `PhoneField` mapped to a phone keeps the digits as typed, short of a number or
/// not; only the E.164 reaches the value, reset while the number is too short.
#[test]
fn a_mapped_phone_field_keeps_typed_text() {
    block_on(async {
        let fixture = Fixture::open("/form/domain", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let tel = "input[type=tel]";
        let value_is = |text: &str| format!("document.querySelector({tel:?}).value === {text:?}");
        let stored_is = |text: &str| {
            format!("document.querySelector('[data-phone]').dataset.phone === {text:?}")
        };

        pointer::click(page, tel).await.unwrap();
        keyboard::type_text(page, "301234567").await.unwrap();
        wait::for_js_true(page, &stored_is("+49301234567"), "the parsed number")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &value_is("301234567"),
            "the text as typed, not regrouped",
        )
        .await
        .unwrap();

        for _ in 0..3 {
            keyboard::press(page, keyboard::BACKSPACE).await.unwrap();
        }
        wait::for_js_true(
            page,
            &stored_is("none"),
            "a short number to empty the value",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &value_is("301234"), "the short text to stay")
            .await
            .unwrap();

        keyboard::type_text(page, "567").await.unwrap();
        wait::for_js_true(page, &stored_is("+49301234567"), "the number again")
            .await
            .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            &stored_is("+49301234567"),
            "the number to outlive a blur",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("typing into a mapped phone field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4084: `Submit::parse` runs after the field's own parse; its error lands on the field and
/// in the summary, nothing is sent, and the next edit clears the error.
#[test]
fn a_submit_blocked_by_parse_shows_the_error_on_its_field() {
    block_on(async {
        let fixture = Fixture::open("/form/command", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let iban = "input[name=iban]";
        let sent_is =
            |text: &str| format!("document.querySelector('[data-sent]').dataset.sent === {text:?}");
        let shows = |text: &str| format!("document.body.textContent.includes({text:?})");

        pointer::click(page, iban).await.unwrap();
        keyboard::type_text(page, "GB29NWBK60161331926819")
            .await
            .unwrap();
        click_button(page, "Send").await.unwrap();
        wait::for_js_true(
            page,
            &shows("Use a German account."),
            "the parse step's error",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[data-slot=summary]')?.textContent.includes('IBAN: Use a German account.')",
            "the summary line",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector({iban:?}).getAttribute('aria-invalid') === 'true'"),
            "the field marked invalid",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &sent_is(""), "nothing sent")
            .await
            .unwrap();

        pointer::click(page, iban).await.unwrap();
        keyboard::press(page, keyboard::END).await.unwrap();
        keyboard::press(page, keyboard::BACKSPACE).await.unwrap();
        wait::for_js_true(
            page,
            "!document.body.textContent.includes('Use a German account.')",
            "the edit to clear the error",
        )
        .await
        .unwrap();

        for _ in 0..30 {
            keyboard::press(page, keyboard::BACKSPACE).await.unwrap();
        }
        keyboard::type_text(page, "DE89370400440532013000")
            .await
            .unwrap();
        click_button(page, "Send").await.unwrap();
        wait::for_js_true(page, &sent_is("DE89370400440532013000"), "the command")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("a submit blocked by parse")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4099: a hidden summary shows no list, focuses the first invalid field and counts the
/// errors in a status.
#[test]
fn a_hidden_summary_focuses_the_first_invalid_field() {
    block_on(async {
        let fixture = Fixture::open("/form/async", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        click_button(page, "Sign in").await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.id === 'email'",
            "focus on the first invalid field",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('[role=status]')].some(s => s.textContent.includes('2 errors in the form.'))",
            "the error count",
        )
        .await
        .unwrap();
        let summary = page
            .evaluate("document.querySelector('[data-slot=summary]') === null")
            .await
            .unwrap()
            .into_value::<bool>()
            .unwrap();
        assert!(summary, "a hidden summary rendered its list");
        wait::for_js_true(
            page,
            "document.querySelector('[data-calls]').dataset.calls === '0'",
            "nothing sent",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a hidden summary").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 4097, 4100: an async submit marks the form busy, ignores a second submit, and its
/// form-wide error shows above the fields in a hidden-summary form and takes focus.
#[test]
fn an_async_submit_is_busy_until_it_answers() {
    block_on(async {
        let fixture = Fixture::open("/form/async", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let data = |key: &str, value: &str| {
            format!("document.querySelector('[data-{key}]').dataset.{key} === {value:?}")
        };

        pointer::click(page, "#email").await.unwrap();
        keyboard::type_text(page, "me@libero.dev").await.unwrap();
        pointer::click(page, "#code").await.unwrap();
        keyboard::type_text(page, "1234").await.unwrap();
        click_button(page, "Sign in").await.unwrap();
        wait::for_js_true(page, &data("submitting", "true"), "the form submitting")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('form').getAttribute('aria-busy') === 'true'",
            "the form busy",
        )
        .await
        .unwrap();

        // Enter in a field submits again: ignored while the first runs.
        pointer::click(page, "#code").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        click_button(page, "Refuse").await.unwrap();
        wait::for_js_true(page, &data("submitting", "false"), "the request answered")
            .await
            .unwrap();
        wait::for_js_true(page, &data("calls", "1"), "one request only")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[data-slot=summary]')?.textContent.includes('Wrong code.')",
            "the form-wide error above the fields",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.dataset.slot === 'summary'",
            "focus on the alert, as no field is at fault",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('form').getAttribute('aria-busy') === 'false'",
            "the form idle",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("an async submit").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4094: a saved-login autofill, as Firefox fills it after load (value, then bubbling
/// `beforeinput`, `input` and `change`), reaches the bound store and the required check.
#[test]
fn a_saved_login_autofill_fills_the_bound_fields() {
    block_on(async {
        let fixture = Fixture::open("/form/login", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded(
            "(() => {
                for (const [name, text] of [['username', 'alice'], ['password', 's3cret']]) {
                    const input = document.querySelector(`input[name=${name}]`);
                    input.dispatchEvent(new InputEvent('beforeinput', { bubbles: true }));
                    input.value = text;
                    input.dispatchEvent(new InputEvent('input', { bubbles: true }));
                    input.dispatchEvent(new Event('change', { bubbles: true }));
                }
            })()"
                .to_string(),
        )
        .await
        .unwrap();
        let stored = "document.querySelector('#stored').dataset";
        wait::for_js_true(
            page,
            &format!("{stored}.user === 'alice' && {stored}.pass === 's3cret'"),
            "the autofilled login in the store",
        )
        .await
        .unwrap();
        pointer::click(page, "#send").await.unwrap();
        wait::for_js_true(page, &format!("{stored}.submits === '1'"), "the submit")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("a saved-login autofill")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4254: a native reset button resets the store too, so the next submit sends no old value.
#[test]
fn a_native_reset_button_resets_the_store() {
    block_on(async {
        let fixture = Fixture::open("/form/login", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "input[name=username]").await.unwrap();
        keyboard::type_text(page, "alice").await.unwrap();
        let stored = "document.querySelector('#stored').dataset";
        wait::for_js_true(
            page,
            &format!("{stored}.user === 'alice'"),
            "the typed name",
        )
        .await
        .unwrap();
        pointer::click(page, "#wipe").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{stored}.user === '' && document.querySelector('input[name=username]').value === ''"
            ),
            "the store and the input reset",
        )
        .await
        .unwrap();
        // Blocked as empty: the old name is not sent.
        pointer::click(page, "#send").await.unwrap();
        wait::for_js_true(
            page,
            "!!document.querySelector('[data-slot=summary]')",
            "the summary",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &format!("{stored}.submits === '0'"), "no submit")
            .await
            .unwrap();

        fixture.console.assert_clean("a native reset").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4106: a password mapped to `Option<ByAddress<T>>` stores what is typed and submits it.
#[test]
fn a_password_mapped_to_a_secret_submits_it() {
    block_on(async {
        let fixture = Fixture::open("/form/secret", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, "input[name=password]").await.unwrap();
        keyboard::type_text(page, "hunter2").await.unwrap();
        pointer::click(page, "#send").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#sent').dataset.sent === 'hunter2'",
            "the secret the submit parsed",
        )
        .await
        .unwrap();
        // Todo 4114: `try_unwrap` gives the holder back while the form's store keeps a clone.
        wait::for_js_true(
            page,
            "document.querySelector('#sent').dataset.shared === 'shared'",
            "try_unwrap refusing a shared secret",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("a mapped secret password")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4094: text the browser wrote before a field mounted - an autofill into server-rendered
/// HTML ahead of hydration, no event heard - is taken on mount by every text-like field.
#[test]
fn text_written_before_mount_reaches_the_form() {
    block_on(async {
        let fixture = Fixture::open("/form/prefill", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        // Filled in the insertion call itself: a `MutationObserver` runs after the mount handlers.
        page.evaluate_bounded(
            "(() => {
                const fills = { user: 'alice', pass: 's3cret', note: 'hello', count: '7', phone: '30 1234567' };
                const fill = () => {
                    for (const [name, text] of Object.entries(fills)) {
                        const control = name === 'phone'
                            ? document.querySelector('input[type=tel]')
                            : document.querySelector(`[name=${name}]`);
                        if (control && !control.dataset.filled) {
                            control.dataset.filled = '1';
                            control.value = text;
                        }
                    }
                };
                const inserts = [
                    [Node.prototype, ['appendChild', 'insertBefore', 'replaceChild']],
                    [Element.prototype, ['after', 'before', 'replaceWith', 'append', 'prepend']],
                ];
                for (const [proto, names] of inserts) {
                    for (const name of names) {
                        const original = proto[name];
                        proto[name] = function (...args) {
                            const out = original.apply(this, args);
                            fill();
                            return out;
                        };
                    }
                }
            })()"
                .to_string(),
        )
        .await
        .unwrap();
        pointer::click(page, "#show").await.unwrap();
        let stored = "document.querySelector('#stored').dataset";
        let fills = [
            ("user", "alice"),
            ("pass", "s3cret"),
            ("note", "hello"),
            ("count", "7"),
            ("phone", "+49301234567"),
        ];
        for (key, text) in fills {
            wait::for_js_true(
                page,
                &format!("{stored}.{key} === {text:?}"),
                &format!("the prefilled {key} in the store"),
            )
            .await
            .unwrap();
        }
        click_button(page, "Send").await.unwrap();
        wait::for_js_true(page, &format!("{stored}.submits === '1'"), "the submit")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("text written before mount")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4096: `text` written into `control` in the insertion call that mounts it, as above,
/// reaches the store, read off `#stored`'s `key`.
async fn prefill_reaches_the_form(control: &str, text: &str, key: &str, stored: &str) {
    let fixture = Fixture::open("/form/prefill-more", Viewport::Desktop)
        .await
        .unwrap();
    let page = &fixture.page;
    page.evaluate_bounded(format!(
        "(() => {{
            const fill = () => {{
                const control = document.querySelector({control:?});
                if (control && !control.dataset.filled) {{
                    control.dataset.filled = '1';
                    control.value = {text:?};
                }}
            }};
            const inserts = [
                [Node.prototype, ['appendChild', 'insertBefore', 'replaceChild']],
                [Element.prototype, ['after', 'before', 'replaceWith', 'append', 'prepend']],
            ];
            for (const [proto, names] of inserts) {{
                for (const name of names) {{
                    const original = proto[name];
                    proto[name] = function (...args) {{
                        const out = original.apply(this, args);
                        fill();
                        return out;
                    }};
                }}
            }}
        }})()"
    ))
    .await
    .unwrap();
    pointer::click(page, "#show").await.unwrap();
    wait::for_js_true(
        page,
        &format!("document.querySelector('#stored').dataset.{key} === {stored:?}"),
        &format!("the prefilled {key} in the store"),
    )
    .await
    .unwrap();

    fixture
        .console
        .assert_clean(&format!("a prefilled {key}"))
        .unwrap();
    fixture.close().await.unwrap();
}

#[test]
fn an_autocomplete_takes_text_written_before_mount() {
    block_on(prefill_reaches_the_form(
        "#city", "Lisbon", "city", "Lisbon",
    ));
}

/// The draft commits as a blur would, split on the field's comma.
#[test]
fn a_tags_field_commits_text_written_before_mount() {
    block_on(prefill_reaches_the_form(
        "#tags",
        "rust, web",
        "tags",
        "rust|web",
    ));
}

/// A one-time code lands whole in the first cell and spreads over the rest.
#[test]
fn a_pin_field_spreads_a_code_written_before_mount() {
    block_on(prefill_reaches_the_form("#pin-1", "4821", "pin", "4821"));
}

#[test]
fn a_date_field_commits_text_written_before_mount() {
    block_on(prefill_reaches_the_form(
        "#due",
        "2026-03-14",
        "due",
        "2026-03-14",
    ));
}

#[test]
fn a_color_field_takes_text_written_before_mount() {
    block_on(prefill_reaches_the_form(
        "#color", "#fa5252", "color", "#fa5252",
    ));
}

/// Todo 4327: a reset while a submit waits on an async rule drops that submit: nothing is
/// sent and the fresh form shows no summary.
#[test]
fn a_reset_drops_a_submit_waiting_on_an_async_rule() {
    block_on(async {
        let fixture = Fixture::open("/form/async-rule", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let data = |key: &str, value: &str| {
            format!("document.querySelector('[data-{key}]').dataset.{key} === {value:?}")
        };

        pointer::click(page, "#email").await.unwrap();
        keyboard::type_text(page, "me@libero.dev").await.unwrap();
        click_button(page, "Sign up").await.unwrap();
        wait::for_js_true(page, &data("submitting", "true"), "the submit waiting")
            .await
            .unwrap();
        click_button(page, "Reset").await.unwrap();
        wait::for_js_true(page, &data("submitting", "false"), "the wait over")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#email').value === ''",
            "the field reset",
        )
        .await
        .unwrap();
        let resubmitted: bool = page
            .evaluate_bounded(
                "document.querySelector('[data-sent]').dataset.sent !== '0' \
                 || !!document.querySelector('[data-slot=summary]')",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!resubmitted, "the reset form was submitted again");

        fixture
            .console
            .assert_clean("a reset during an async rule")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4087: an async rule shows its field busy after a blur, then its answer; a submit
/// ends the debounce, waits for the answer and sends once the address is free.
#[test]
fn an_async_rule_answers_on_the_field_and_holds_the_submit() {
    block_on(async {
        let fixture = Fixture::open("/form/async-rule", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let data = |key: &str, value: &str| {
            format!("document.querySelector('[data-{key}]').dataset.{key} === {value:?}")
        };
        let busy = |value: &str| {
            format!("document.querySelector('#email').getAttribute('aria-busy') === {value}")
        };

        pointer::click(page, "#email").await.unwrap();
        keyboard::type_text(page, "me@libero.dev").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(page, &busy("'true'"), "the field busy after the blur")
            .await
            .unwrap();
        wait::for_js_true(page, &data("asked", "1"), "one request after the pause")
            .await
            .unwrap();
        click_button(page, "Taken").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[data-slot=status]')?.textContent.includes('This address is taken.')",
            "the answer on the field",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &busy("null"), "the field no longer busy")
            .await
            .unwrap();

        pointer::click(page, "#email").await.unwrap();
        keyboard::type_text(page, "m").await.unwrap();
        click_button(page, "Sign up").await.unwrap();
        wait::for_js_true(page, &data("submitting", "true"), "the submit waiting")
            .await
            .unwrap();
        wait::for_js_true(page, &data("asked", "2"), "the submit asking at once")
            .await
            .unwrap();
        wait::for_js_true(page, &data("sent", "0"), "nothing sent while it asks")
            .await
            .unwrap();
        click_button(page, "Free").await.unwrap();
        wait::for_js_true(page, &data("sent", "1"), "sent once the address is free")
            .await
            .unwrap();
        wait::for_js_true(page, &data("submitting", "false"), "the form idle")
            .await
            .unwrap();

        fixture.console.assert_clean("an async rule").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4116: with `pending_loader` a pending field draws a loader in its frame, at the frame's
/// own height, and drops it with the answer.
#[test]
fn a_pending_field_draws_its_loader_inside_the_frame() {
    block_on(async {
        let fixture = Fixture::open("/form/async-rule-loader", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded(
            "window.frameHeight = document.querySelector('[data-slot=frame]').getBoundingClientRect().height",
        )
        .await
        .unwrap();

        pointer::click(page, "#email").await.unwrap();
        keyboard::type_text(page, "me@libero.dev").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const loader = document.querySelector('[data-slot=frame] > [data-pending] > [aria-hidden]'); \
             return !!loader && loader.getBoundingClientRect().width > 0; })()",
            "the loader drawn while the rule asks",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[data-slot=frame]').getBoundingClientRect().height === window.frameHeight",
            "the frame at its own height beside the loader",
        )
        .await
        .unwrap();
        // An answer before the request would be dropped by it.
        wait::for_js_true(
            page,
            "document.querySelector('[data-asked]').dataset.asked === '1'",
            "the request after the pause",
        )
        .await
        .unwrap();

        click_button(page, "Free").await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('[data-pending]')",
            "the loader gone with the answer",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a pending loader").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4207: summary lines follow the page, not registration: a field mounted late and a
/// Fieldset rule sit in their place. Hidden, focus goes to the first invalid field on the page.
#[test]
fn summary_lines_and_hidden_focus_follow_the_page() {
    const LABELS: &str = "[...document.querySelectorAll('[data-slot=summary] a')].map(a => a.textContent.split(':')[0])";
    block_on(async {
        let fixture = Fixture::open("/form/order", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        add_company(page).await.unwrap();
        click_button(page, "Order").await.unwrap();
        focus::wait_for_focus(page, "[data-slot=summary]", "a blocked submit")
            .await
            .unwrap();
        let labels: Vec<String> = page
            .evaluate_bounded(LABELS)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(labels, ["Name", "Company name", "Repeat password", "Age"]);
        fixture.close().await.unwrap();

        let fixture = Fixture::open("/form/order-hidden", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        add_company(page).await.unwrap();
        pointer::click(page, "#name").await.unwrap();
        keyboard::type_text(page, "Ada").await.unwrap();
        click_button(page, "Order").await.unwrap();
        focus::wait_for_focus(page, "#company", "the first invalid field on the page")
            .await
            .unwrap();
        fixture.console.assert_clean("the summary order").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn add_company(page: &Page) -> Result<()> {
    click_button(page, "Add company").await?;
    wait::for_js_true(
        page,
        "!!document.getElementById('company')",
        "the company field",
    )
    .await
}

/// Todo 4208: a disabled or readonly field is barred from validation, as natively: its own
/// failing rule neither blocks the submit nor shows.
#[test]
fn a_disabled_or_readonly_field_does_not_block_the_submit() {
    const LINES: &str =
        "[...document.querySelectorAll('[data-slot=summary] a')].map(a => a.textContent)";
    block_on(async {
        let fixture = Fixture::open("/form/order", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        click_button(page, "Order").await.unwrap();
        focus::wait_for_focus(page, "[data-slot=summary]", "a blocked submit")
            .await
            .unwrap();
        let lines: Vec<String> = page
            .evaluate_bounded(LINES)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            lines
                .iter()
                .all(|line| !line.starts_with("Locked") && !line.starts_with("Frozen")),
            "a barred field has a line: {lines:?}"
        );
        let invalid: bool = page
            .evaluate_bounded(
                "['locked', 'frozen'].some(id => document.getElementById(id).getAttribute('aria-invalid') === 'true')",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!invalid, "a barred field shows an error");

        for (field, text) in [("#name", "Ada"), ("#repeat", "secret"), ("#age", "36")] {
            pointer::click(page, field).await.unwrap();
            keyboard::type_text(page, text).await.unwrap();
        }
        click_button(page, "Order").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[data-calls]').dataset.calls === '1'",
            "the submit past the barred fields",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("barred fields").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4209: a second blocked submit with the same count is announced again: the hidden
/// summary's status changes in the DOM, so a screen reader hears it.
#[test]
fn every_blocked_submit_announces_the_error_count() {
    const STATUS: &str = "[...document.querySelectorAll('[role=status]')].find(s => s.textContent.includes('errors in the form'))";
    block_on(async {
        let fixture = Fixture::open("/form/order-hidden", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        click_button(page, "Order").await.unwrap();
        focus::wait_for_focus(page, "#name", "the first invalid field")
            .await
            .unwrap();
        wait::for_js_true(page, &format!("!!{STATUS}"), "the error count")
            .await
            .unwrap();
        page.evaluate_bounded(format!(
            "window.__changes = 0; new MutationObserver(r => window.__changes += r.length)\
             .observe({STATUS}, {{ childList: true, subtree: true, characterData: true }})"
        ))
        .await
        .unwrap();
        click_button(page, "Order").await.unwrap();
        wait::for_js_true(
            page,
            &format!("window.__changes > 0 && {STATUS}?.textContent === '3 errors in the form.'"),
            "the count announced again",
        )
        .await
        .unwrap();
        focus::wait_for_focus(page, "#name", "the first invalid field again")
            .await
            .unwrap();
        fixture.console.assert_clean("the repeated count").unwrap();
        fixture.close().await.unwrap();
    });
}
