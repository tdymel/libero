//! A press on a field frame's padding is a press on its control (todo 462),
//! while the control's own presses and the slots' buttons keep theirs.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::page::{
    EventFileChooserOpened, SetInterceptFileChooserDialogParams,
};
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};
use futures::StreamExt;

#[test]
fn a_press_on_the_padding_is_a_press_on_the_control() {
    block_on(async {
        let fixture = Fixture::open("/field-frame", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // The frame's left padding, then its slot text, focus the input.
        let input = "[data-case=text] input";
        press(page, &left_padding("text")).await.unwrap();
        focused(page, input).await.unwrap();
        blur(page).await.unwrap();
        press(page, &centre("[data-case=text] [data-slot=leading]"))
            .await
            .unwrap();
        focused(page, input).await.unwrap();

        // A press on the padding leaves a selection alone; a drag in the input
        // still selects.
        page.evaluate(format!(
            "document.querySelector({input:?}).setSelectionRange(1, 3)"
        ))
        .await
        .unwrap();
        press(page, &bottom_padding("text")).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const el = document.querySelector({input:?}); \
                 return el.selectionStart === 1 && el.selectionEnd === 3; }})()"
            ),
            "the selection to survive a padding press",
        )
        .await
        .unwrap();
        drag_across(page, input).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const el = document.querySelector({input:?}); \
                 return el.selectionEnd - el.selectionStart >= 3; }})()"
            ),
            "a drag in the input to select",
        )
        .await
        .unwrap();

        // A stepper keeps its own press; the padding focuses the spinbutton.
        blur(page).await.unwrap();
        press(
            page,
            &centre("[data-case=number] button[aria-label=Increase]"),
        )
        .await
        .unwrap();
        truthy(
            page,
            "document.querySelector('[data-case=number] input').value === '4'",
            "one step up",
        )
        .await
        .unwrap();
        press(page, &bottom_padding("number")).await.unwrap();
        focused(page, "[data-case=number] input").await.unwrap();
        truthy(
            page,
            "document.querySelector('[data-case=number] input').value === '4'",
            "no step from the padding",
        )
        .await
        .unwrap();

        // The padding opens a select, as a press on its trigger does.
        let trigger = "[data-case=select] [role=combobox]";
        press(page, &bottom_padding("select")).await.unwrap();
        focused(page, trigger).await.unwrap();
        expanded(page, trigger, true).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        expanded(page, trigger, false).await.unwrap();

        // The multi-select's chevron is no button, and opens the list once.
        let multi = "[data-case=multi] [role=combobox]";
        blur(page).await.unwrap();
        press(page, &centre("[data-case=multi] [data-slot=trailing] > *"))
            .await
            .unwrap();
        expanded(page, multi, true).await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        expanded(page, multi, false).await.unwrap();
        blur(page).await.unwrap();
        press(page, &left_padding("multi")).await.unwrap();
        focused(page, multi).await.unwrap();
        expanded(page, multi, true).await.unwrap();

        fixture
            .console
            .assert_clean("pressing a field frame's padding")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The other framed fields: the padding focuses the control, not a slot's
/// button. `PinField` has a frame per cell, so its second cell is pressed.
#[test]
fn every_framed_field_takes_a_padding_press() {
    block_on(async {
        let fixture = Fixture::open("/field-frame", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        for case in [
            "textarea", "native", "phone", "color", "date", "time", "file",
        ] {
            blur(page).await.unwrap();
            press(page, &bottom_padding(case)).await.unwrap();
            let frame = format!("[data-case={case}] [data-frame]");
            in_control(page, &frame).await.unwrap();
            let state: String = page
                .evaluate(
                    "(() => { const el = document.activeElement; \
                     return el.tagName + ' expanded=' + el.getAttribute('aria-expanded'); })()"
                        .to_string(),
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            println!("{case}: {state}");
            keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        }

        blur(page).await.unwrap();
        let cell = "document.querySelectorAll('[data-case=pin] [data-frame]')[1]";
        press(
            page,
            &format!(
                "(() => {{ {cell}.scrollIntoView({{ block: 'center' }}); \
                 const r = {cell}.getBoundingClientRect(); \
                 return [r.x + 3, r.y + r.height / 2]; }})()"
            ),
        )
        .await
        .unwrap();
        truthy(
            page,
            &format!("document.activeElement === {cell}.querySelector('input')"),
            "the second pin cell to be focused",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("pressing every framed field's padding")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `FileField` opens its picker once per press: from the padding, which clicks
/// the surface, as from the surface itself and from Enter.
#[test]
fn a_file_field_opens_its_picker_once_per_press() {
    block_on(async {
        let fixture = Fixture::open("/field-frame", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(SetInterceptFileChooserDialogParams::new(true))
            .await
            .unwrap();
        let opened = Arc::new(AtomicUsize::new(0));
        let mut choosers = page
            .event_listener::<EventFileChooserOpened>()
            .await
            .unwrap();
        let count = opened.clone();
        tokio::spawn(async move {
            while choosers.next().await.is_some() {
                count.fetch_add(1, Ordering::SeqCst);
            }
        });
        let settle = |want: usize| {
            let opened = opened.clone();
            async move {
                tokio::time::sleep(Duration::from_millis(400)).await;
                assert_eq!(opened.load(Ordering::SeqCst), want, "file choosers opened");
            }
        };

        press(page, &bottom_padding("file")).await.unwrap();
        settle(1).await;
        press(page, &centre("[data-case=file] [role=button]"))
            .await
            .unwrap();
        settle(2).await;
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        settle(3).await;

        fixture
            .console
            .assert_clean("opening a file field's picker")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The focus is in the frame, outside its slots.
async fn in_control(page: &Page, frame: &str) -> Result<()> {
    truthy(
        page,
        &format!(
            "(() => {{ const frame = document.querySelector({frame:?}); \
             const el = document.activeElement; \
             return el !== frame && frame.contains(el) && !el.closest('[data-slot]'); }})()"
        ),
        &format!("the focus in {frame}'s control"),
    )
    .await
}

/// A JS point expression: 3px inside the frame's left edge, at its middle.
fn left_padding(case: &str) -> String {
    format!(
        "(() => {{ const f = document.querySelector('[data-case={case}] [data-frame]'); \
         f.scrollIntoView({{ block: 'center' }}); const r = f.getBoundingClientRect(); \
         return [r.x + 3, r.y + r.height / 2]; }})()"
    )
}

/// 3px above the frame's bottom edge, at its middle.
fn bottom_padding(case: &str) -> String {
    format!(
        "(() => {{ const f = document.querySelector('[data-case={case}] [data-frame]'); \
         f.scrollIntoView({{ block: 'center' }}); const r = f.getBoundingClientRect(); \
         return [r.x + r.width / 2, r.bottom - 3]; }})()"
    )
}

fn centre(selector: &str) -> String {
    format!(
        "(() => {{ const el = document.querySelector({selector:?}); \
         el.scrollIntoView({{ block: 'center' }}); const r = el.getBoundingClientRect(); \
         return [r.x + r.width / 2, r.y + r.height / 2]; }})()"
    )
}

/// A real pointer press and release at the point `at` evaluates to.
async fn press(page: &Page, at: &str) -> Result<()> {
    let (x, y): (f64, f64) = page.evaluate(at).await?.into_value()?;
    let at = pointer::Point { x, y };
    pointer::drag(page, at, at, 1).await
}

/// Drags across the input's text, from its left edge rightwards.
async fn drag_across(page: &Page, input: &str) -> Result<()> {
    let (x, y, width): (f64, f64, f64) = page
        .evaluate(format!(
            "(() => {{ const r = document.querySelector({input:?}).getBoundingClientRect(); \
             return [r.x, r.y + r.height / 2, r.width]; }})()"
        ))
        .await?
        .into_value()?;
    let from = pointer::Point { x: x + 1.0, y };
    let to = pointer::Point {
        x: x + width - 2.0,
        y,
    };
    pointer::drag(page, from, to, 5).await
}

async fn blur(page: &Page) -> Result<()> {
    page.evaluate("document.activeElement && document.activeElement.blur()")
        .await?;
    Ok(())
}

async fn focused(page: &Page, selector: &str) -> Result<()> {
    truthy(
        page,
        &format!("document.activeElement === document.querySelector({selector:?})"),
        &format!("{selector} to be focused"),
    )
    .await
}

async fn expanded(page: &Page, selector: &str, open: bool) -> Result<()> {
    truthy(
        page,
        &format!("document.querySelector({selector:?}).getAttribute('aria-expanded') === '{open}'"),
        &format!("{selector} aria-expanded={open}"),
    )
    .await
}

async fn truthy(page: &Page, expression: &str, what: &str) -> Result<()> {
    wait::for_js_true(page, expression, what).await
}
