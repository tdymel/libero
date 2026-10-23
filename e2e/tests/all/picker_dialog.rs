//! `ColorField`, `ChronoField`: an input with a non-modal dialog dropdown (APG Date Picker
//! Combobox). Opening keeps focus in the input; ArrowDown enters; Escape returns.

use anyhow::{Result, bail, ensure};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::archetypes::back_held;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, linger};
use e2e::passes::{focus, keyboard};
use e2e::suite::Step;
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, wait};

const DIALOG: &str = "[role=dialog]";
// The "open" state is the one Arrow Down entered.
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
    let phone = android || d.platform() == Platform::Ios;
    let inputmode = d.attr(FIELD, "inputmode").await?;
    ensure!(
        inputmode.as_deref() == phone.then_some("none"),
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

/// Todo 3905: a tap on the picker's own text box focuses it natively, so the WebView raises
/// its keyboard; lifting `inputmode` on the focused field, or a `focus()` from Rust, does not (3902).
async fn the_picker_text_box_raises_the_keyboard<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    const TYPED: &str = "[role=dialog] [data-slot=text-input]";
    let field = match route {
        "/color-field" => "#color-field",
        _ => "#date-range-field",
    };
    d.click(field).await?;
    eventually(d, "the text box in the picker", async |d| {
        d.exists(TYPED).await
    })
    .await?;
    linger(d, 40).await;
    ensure!(
        !d.soft_keyboard_shown().await?,
        "the field's tap raised the keyboard"
    );
    d.click(TYPED).await?;
    eventually(
        d,
        "the text box focused, the picker still open",
        async |d| Ok(d.is_focused(TYPED).await? && d.exists(DIALOG).await?),
    )
    .await?;
    eventually(d, "the soft keyboard", async |d| {
        d.soft_keyboard_shown().await
    })
    .await?;
    // Back hides the keyboard first, so the next scenario starts without it.
    d.press_back().await?;
    eventually(d, "Back to hide the keyboard", async |d| {
        Ok(!d.soft_keyboard_shown().await?)
    })
    .await
}

e2e::scenario!(
    android_a_date_pickers_text_box_raises_the_keyboard,
    "/date-range-field",
    the_picker_text_box_raises_the_keyboard,
    android_only("only a phone app keeps the field's keyboard down under a mouse")
);

e2e::scenario!(
    android_a_color_pickers_text_box_raises_the_keyboard,
    "/color-field",
    the_picker_text_box_raises_the_keyboard,
    android_only("only a phone app keeps the field's keyboard down under a mouse")
);

/// Todo 3952: a hardware Arrow Down enters the picker in a WebView too, and the input's blur,
/// which no `:focus` query can place there, leaves it open.
async fn arrow_down_enters_and_stays<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    let field = FIELDS
        .iter()
        .find(|(path, _)| *path == route)
        .map(|(_, field)| *field)
        .unwrap_or("#chrono-field");
    d.focus(field).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    let entered = format!("document.querySelector({DIALOG:?})?.contains(document.activeElement)");
    eventually(d, "Arrow Down to move focus into the picker", async |d| {
        Ok(d.evaluate(&entered).await? == serde_json::Value::Bool(true))
    })
    .await?;
    d.flush().await?;
    ensure!(
        d.exists(DIALOG).await? && d.attr(field, "aria-expanded").await?.as_deref() == Some("true"),
        "the picker closed once focus entered it"
    );
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "Escape to close and return to the field", async |d| {
        Ok(!d.exists(DIALOG).await? && d.is_focused(field).await?)
    })
    .await
}

e2e::scenario!(
    android_arrow_down_enters_a_date_picker,
    "/chrono-field",
    arrow_down_enters_and_stays,
    android_only("the web's `:focus` answer is covered by the dialog contract")
);
e2e::scenario!(
    android_arrow_down_enters_a_color_picker,
    "/color-field",
    arrow_down_enters_and_stays,
    android_only("the web's `:focus` answer is covered by the dialog contract")
);

/// Todo 4042: Tab walks the dropdown and leaves it past its last stop for a page control,
/// Shift+Tab past its first for the field; a WebView queries no stops for the edges.
async fn tab_leaves_at_the_edges<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    // A fresh mark per focused element, so each press waits for its own move.
    const MARK: &str = "(() => { const el = document.activeElement; \
        el.dataset.e2eTab ??= String(Math.random()); \
        return [el.dataset.e2eTab, !!el.closest('[role=dialog]'), el.id, \
        el === document.body]; })()";
    let field = match route {
        "/color-field/tab-out" => "#color-field",
        _ => "#chrono-field",
    };
    for backwards in [false, true] {
        d.focus(field).await?;
        d.press(keyboard::ARROW_DOWN).await?;
        let entered =
            format!("document.querySelector({DIALOG:?})?.contains(document.activeElement)");
        eventually(d, "Arrow Down to move focus into the picker", async |d| {
            Ok(d.evaluate(&entered).await? == serde_json::Value::Bool(true))
        })
        .await?;
        let mut left = None;
        for _ in 0..40 {
            let before = d.evaluate(MARK).await?;
            match backwards {
                true => d.press_shift(keyboard::TAB).await?,
                false => d.press(keyboard::TAB).await?,
            }
            let mut now = before.clone();
            eventually(d, "Tab to move focus", async |d| {
                now = d.evaluate(MARK).await?;
                Ok(now[0] != before[0])
            })
            .await?;
            if now[1] == serde_json::Value::Bool(false) {
                left = Some(now);
                break;
            }
        }
        let Some(left) = left else {
            bail!("40 Tabs (backwards: {backwards}) never left the picker");
        };
        // Forward lands on the stop after the field: the colour's own button, else `#after`.
        let on_field = format!("#{}", left[2].as_str().unwrap_or_default()) == field;
        ensure!(
            on_field == backwards && left[3] == serde_json::Value::Bool(false),
            "Tab (backwards: {backwards}) past the edge landed on {left:?}"
        );
    }
    Ok(())
}

e2e::scenario!(
    tab_leaves_a_date_picker_at_its_edges,
    "/chrono-field/tab-out",
    tab_leaves_at_the_edges,
    native: skip("Blitz reads no script; the WebView walk this guards is 4042's")
);
e2e::scenario!(
    tab_leaves_a_color_picker_at_its_edges,
    "/color-field/tab-out",
    tab_leaves_at_the_edges,
    native: skip("Blitz reads no script; the WebView walk this guards is 4042's")
);

/// Android's Back closes the dropdown, keeps the value, and the app stays (1300).
async fn back_closes<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    let field = FIELDS
        .iter()
        .find(|(path, _)| *path == route)
        .map(|(_, field)| *field)
        .unwrap_or("#chrono-field");
    let before = d.value(field).await?;
    d.click(field).await?;
    eventually(d, "the dropdown", async |d| d.exists(DIALOG).await).await?;
    back_held(d).await?;
    // The colour's text input raises the keyboard; Android's first Back only hides it.
    if d.attr(field, "inputmode").await?.as_deref() != Some("none") {
        eventually(d, "the soft keyboard", async |d| {
            d.soft_keyboard_shown().await
        })
        .await?;
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
    android_only("1275: no Back key off Android")
);
e2e::scenario!(
    android_back_closes_a_date_field,
    "/chrono-field",
    back_closes,
    android_only("1275: no Back key off Android")
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
    keyboard::tab_to(page, input, 5).await?;
    // WCAG 3.2.1: focus alone opens nothing.
    wait::for_js_true(
        page,
        &format!(
            "document.querySelector({input:?})?.getAttribute('aria-expanded') === 'false' \
             && !document.querySelector({DIALOG:?})"
        ),
        "the focused field to stay closed",
    )
    .await?;
    focus::assert_focused(page, input, "tabbing into the field").await?;

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
        .evaluate_bounded(format!(
            "document.querySelector({:?})?.getAttribute('aria-label') ?? ''",
            DIALOG
        ))
        .await?
        .into_value()?;
    if name.trim().is_empty() {
        bail!("the dialog has no accessible name");
    }
    let modal: bool = page
        .evaluate_bounded(format!(
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

/// Todo 1552: under the `sm` breakpoint the range dropdown fits a 320px page, its months in
/// one scrolling column (3903); above it, two months side by side (1.4.10).
#[test]
fn a_date_range_field_opens_one_month_on_a_narrow_page() {
    block_on(async {
        let route = "/date-range-field";
        for (narrow, months) in [(true, 12), (false, 2)] {
            let viewport = if narrow {
                Viewport::Mobile
            } else {
                Viewport::Desktop
            };
            let fixture = Fixture::open(route, viewport).await.unwrap();
            let page = &fixture.page;
            if narrow {
                page.execute_bounded(SetDeviceMetricsOverrideParams::new(320, 800, 1.0, true))
                    .await
                    .unwrap();
            }
            keyboard::tab_to(page, "#date-range-field", 5)
                .await
                .unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            wait::for_js_true(
                page,
                &format!("document.querySelectorAll('{DIALOG} [role=grid]').length === {months}"),
                &format!("{months} month grids, narrow {narrow}"),
            )
            .await
            .unwrap();
            // Not `innerWidth`: a mobile layout viewport widens to its content.
            let width = if narrow {
                320
            } else {
                Viewport::Desktop.size().0
            };
            let overflow: f64 = page
                .evaluate_bounded(format!("document.documentElement.scrollWidth - {width}"))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                overflow <= 0.0,
                "narrow {narrow}: the page scrolls sideways by {overflow}px"
            );
            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }
    });
}
