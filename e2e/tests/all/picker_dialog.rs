//! `ColorField`, `ChronoField`: an input with a non-modal dialog dropdown (APG Date Picker
//! Combobox). Opening keeps focus in the input; ArrowDown enters; Escape returns.

use anyhow::{Result, bail, ensure};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, linger};
use e2e::passes::{focus, keyboard};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const DIALOG: &str = "[role=dialog]";
// Focus alone opens the dialog, so the "open" state is the one Arrow Down entered.
const ENTERED: &str = "[role=dialog]:focus-within";

const FIELDS: [(&str, &str); 2] = [
    ("/color-field", "#color-field"),
    ("/chrono-field", "#chrono-field"),
];

/// Todo 1012: a tap on a date field opens the calendar and, on a phone, no
/// soft keyboard over it. The web keeps its keyboard.
async fn a_tap_opens_only_the_calendar<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const FIELD: &str = "#date-range-field";
    d.click(FIELD).await?;
    eventually(d, "the calendar after a tap", async |d| {
        d.exists(DIALOG).await
    })
    .await?;
    let android = d.platform() == Platform::Android;
    let inputmode = d.attr(FIELD, "inputmode").await?;
    ensure!(
        inputmode.as_deref() == android.then_some("none"),
        "{:?}: inputmode {inputmode:?}",
        d.platform()
    );
    // Only Android reports a soft keyboard; it rises a while after the tap.
    if android {
        linger(d, 40).await;
        ensure!(!d.soft_keyboard_shown().await?, "the soft keyboard came up");
    }
    Ok(())
}

e2e::scenario!(
    a_tap_on_a_date_range_field_opens_only_the_calendar,
    "/date-range-field",
    a_tap_opens_only_the_calendar
);

/// Android's Back closes the dropdown, keeps the value, and the app stays (1300).
async fn back_closes<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    if d.platform() != Platform::Android {
        return Ok(());
    }
    let field = FIELDS
        .iter()
        .find(|(path, _)| *path == route)
        .map(|(_, field)| *field)
        .unwrap_or("#chrono-field");
    let before = d.value(field).await?;
    d.click(field).await?;
    eventually(d, "the dropdown", async |d| d.exists(DIALOG).await).await?;
    linger(d, 40).await;
    // The colour's text input raises the keyboard; Android's first Back only hides it.
    if d.soft_keyboard_shown().await? {
        d.press_back().await?;
        eventually(d, "Back to hide the keyboard", async |d| {
            Ok(!d.soft_keyboard_shown().await?)
        })
        .await?;
        ensure!(
            d.exists(DIALOG).await?,
            "Back hid the keyboard and closed the dropdown"
        );
    }
    d.press_back().await?;
    eventually(d, "Back to close the dropdown", async |d| {
        Ok(!d.exists(DIALOG).await?)
    })
    .await?;
    let after = d.value(field).await?;
    ensure!(
        after == before,
        "Back changed the value: {before:?} to {after:?}"
    );
    // The app stayed and still opens the dropdown.
    d.click(field).await?;
    eventually(d, "the dropdown again", async |d| d.exists(DIALOG).await).await
}

e2e::scenario!(
    android_back_closes_a_color_field,
    "/color-field",
    back_closes,
    native: skip("1275: no Back key off Android"),
    desktop: skip("1275: no Back key off Android")
);
e2e::scenario!(
    android_back_closes_a_date_field,
    "/chrono-field",
    back_closes,
    native: skip("1275: no Back key off Android"),
    desktop: skip("1275: no Back key off Android")
);

#[test]
fn the_color_field_meets_the_baseline() {
    Suite::new("color_field", "/color-field")
        .focusable("#color-field")
        .state(
            "open",
            &[
                Step::TabTo("#color-field"),
                Step::Press(keyboard::ARROW_DOWN),
            ],
            ENTERED,
        )
        .run();
}

#[test]
fn the_date_field_meets_the_baseline() {
    Suite::new("chrono_field", "/chrono-field")
        .focusable("#chrono-field")
        .state(
            "open",
            &[
                Step::TabTo("#chrono-field"),
                Step::Press(keyboard::ARROW_DOWN),
            ],
            ENTERED,
        )
        .run();
}

#[test]
fn arrow_down_enters_the_dialog_and_escape_returns_to_the_input() {
    block_on(async {
        for (route, input) in FIELDS {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            dialog_contract(&fixture.page, input)
                .await
                .unwrap_or_else(|e| panic!("{route}: {e}"));
            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 509: Ctrl/Meta+ArrowDown is the caret's and leaves focus in the input;
/// Alt+ArrowDown enters the dialog like the plain key (APG).
#[test]
fn only_alt_arrow_down_enters_the_dialog() {
    block_on(async {
        for (route, input) in FIELDS {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, input, 5).await.unwrap();
            wait::for_visible(page, DIALOG).await.unwrap();
            keyboard::assert_chords_ignored_with(
                page,
                &[("Ctrl", keyboard::CTRL), ("Meta", keyboard::META)],
                &[keyboard::ARROW_DOWN],
                &format!("document.activeElement === document.querySelector({input:?})"),
            )
            .await
            .unwrap_or_else(|e| panic!("{route}: {e}"));
            keyboard::press_with(page, keyboard::ARROW_DOWN, keyboard::ALT)
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!("document.querySelector({DIALOG:?})?.contains(document.activeElement)"),
                "Alt+ArrowDown to move focus into the dialog",
            )
            .await
            .unwrap_or_else(|e| panic!("{route}: {e}"));
            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// A theme's own range separator reads back, typed with or without its spaces.
#[test]
fn a_typed_range_reads_at_the_theme_separator() {
    block_on(async {
        let route = "/date-range-field";
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#date-range-field", 5)
            .await
            .unwrap();
        keyboard::type_text(page, "Sep 1, 2026～Sep 5, 2026")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('input[name=stay]')?.value === '2026-09-01/2026-09-05'",
            "the typed range to post",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#date-range-field')?.value === 'September 1, 2026 ～ September 5, 2026'",
            "the field to show the range with the theme separator",
        )
        .await
        .unwrap();
        fixture.console.assert_clean(route).unwrap();
        fixture.close().await.unwrap();
    });
}

async fn dialog_contract(page: &Page, input: &str) -> Result<()> {
    // Focus alone opens it, and focus stays in the input.
    keyboard::tab_to(page, input, 5).await?;
    wait::for_visible(page, DIALOG).await?;
    focus::assert_focused(page, input, "focus opening the dropdown").await?;

    keyboard::press(page, keyboard::ARROW_DOWN).await?;
    wait::for_js_true(
        page,
        &format!("document.querySelector({DIALOG:?})?.contains(document.activeElement)"),
        "Arrow Down to move focus into the dialog",
    )
    .await?;

    keyboard::press(page, keyboard::ESCAPE).await?;
    wait::for_js_true(
        page,
        &format!("document.querySelector({input:?})?.getAttribute('aria-expanded') === 'false'"),
        "Escape to close the dialog",
    )
    .await?;
    focus::assert_focused(page, input, "Escape closing the dialog").await?;

    // Named, and honest about not trapping focus.
    keyboard::press(page, keyboard::ARROW_DOWN).await?;
    wait::for_visible(page, DIALOG).await?;
    let name: String = page
        .evaluate(format!(
            "document.querySelector({:?})?.getAttribute('aria-label') ?? ''",
            DIALOG
        ))
        .await?
        .into_value()?;
    if name.trim().is_empty() {
        bail!("the dialog has no accessible name");
    }
    let modal: bool = page
        .evaluate(format!(
            "document.querySelector({:?})?.hasAttribute('aria-modal')",
            DIALOG
        ))
        .await?
        .into_value()?;
    if modal {
        bail!("the dialog does not trap focus, so it must not claim aria-modal");
    }
    Ok(())
}
