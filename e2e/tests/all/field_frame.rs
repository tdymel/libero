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
use e2e::driver::{Driver, eventually_text};
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
        press(
            page,
            &centre("[data-case=multi] [data-frame] > [data-slot=trailing] > *"),
        )
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

/// Typing fills one pin cell and moves on, Backspace clears and steps back. A cell redraws
/// only on its own change, so each keystroke checks every cell.
#[test]
fn typing_a_pin_fills_and_clears_one_cell_at_a_time() {
    const BACKSPACE: keyboard::Key = keyboard::Key {
        key: "Backspace",
        code: "Backspace",
        vk: 8,
        text: None,
    };
    block_on(async {
        let fixture = Fixture::open("/field-frame", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let cells = "[...document.querySelectorAll('[data-case=pin] input')]";
        let pin_is = |pin: &str, focus: usize| {
            format!(
                "(() => {{ const cells = {cells}; \
                 return cells.map(c => c.value).join('') === {pin:?} \
                 && document.activeElement === cells[{focus}]; }})()"
            )
        };

        page.evaluate(format!("{cells}[0].focus()")).await.unwrap();
        let mut typed = String::new();
        for (index, digit) in ["1", "2", "3", "4"].into_iter().enumerate() {
            keyboard::type_text(page, digit).await.unwrap();
            typed.push_str(digit);
            wait::for_js_true(page, &pin_is(&typed, (index + 1).min(3)), "a digit")
                .await
                .unwrap();
        }

        keyboard::press(page, BACKSPACE).await.unwrap();
        wait::for_js_true(page, &pin_is("123", 3), "the last cell cleared")
            .await
            .unwrap();
        keyboard::press(page, BACKSPACE).await.unwrap();
        wait::for_js_true(page, &pin_is("123", 2), "a step back from an empty cell")
            .await
            .unwrap();

        fixture.console.assert_clean("typing a pin").unwrap();
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
            "textarea",
            "native",
            "phone",
            "color",
            "date",
            "time",
            "file",
            "autocomplete",
            "cascader",
            "tags",
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

/// `FileField` opens its picker once per press: from the padding, as from the
/// Browse button itself and from Enter.
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
        press(page, &centre("[data-case=file] [data-slot=browse]"))
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

/// Todo 532: with no value and no placeholder a trigger shrank to its content,
/// a 0px `MultiSelect` trigger; it fills the frame and takes a press at its centre.
#[test]
fn an_empty_trigger_fills_its_frame() {
    block_on(async {
        let fixture = Fixture::open("/field-frame", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        for case in ["select-bare", "multi-bare", "cascader"] {
            let (trigger, frame, hit): (f64, f64, bool) = page
                .evaluate(format!(
                    "(() => {{ const f = document.querySelector('[data-case={case}] [data-frame]'); \
                     f.scrollIntoView({{ block: 'center' }}); \
                     const t = f.querySelector('[role=combobox]'); const s = getComputedStyle(f); \
                     const inner = f.clientHeight - parseFloat(s.paddingTop) - parseFloat(s.paddingBottom); \
                     const r = f.getBoundingClientRect(); \
                     const at = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2); \
                     return [t.getBoundingClientRect().height, inner, t.contains(at)]; }})()"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                (trigger - frame).abs() < 0.5 && hit,
                "{case}: the trigger is {trigger}px tall in a {frame}px frame, \
                 the frame's centre on it: {hit}"
            );
        }

        fixture.console.assert_clean("bare triggers").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The focus is in the frame, outside its slots. `FileField`'s Browse button
/// is its control, in a slot of its own.
async fn in_control(page: &Page, frame: &str) -> Result<()> {
    truthy(
        page,
        &format!(
            "(() => {{ const frame = document.querySelector({frame:?}); \
             const el = document.activeElement; \
             return el !== frame && frame.contains(el) \
             && !el.closest('[data-slot]:not([data-slot=browse], [data-slot=control], [data-slot=frame])'); }})()"
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

/// WCAG 1.4.11: a field frame's border parts from the page outside and from
/// its own surface inside at 3:1 (todo 490).
#[test]
fn a_frame_border_parts_at_3_to_1() {
    crate::boundary::assert_boundaries(
        "/field-frame",
        "const frame = [...document.querySelectorAll('[data-frame]')]
             .find(f => !/error|warning|disabled/.test(f.dataset.state || ''));
         const border = CSS(frame, 'borderTopColor');
         return [
             ['frame border on the page', RATIO(border, PAGE(frame))],
             ['frame border on its surface', RATIO(border, CSS(frame, 'backgroundColor'))],
         ];",
    );
}

/// WCAG 1.4.10: a field in a row flex box narrower than its input's intrinsic
/// width shrinks to it, and its slot buttons keep a 24px target.
async fn a_field_shrinks_into_a_narrow_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for stage in ["#stage", "#stage-text"] {
        let outer = d.rect(stage).await?;
        let frame = d.rect(&format!("{stage} [data-frame]")).await?;
        assert!(
            frame.x + frame.width <= outer.x + outer.width + 0.5,
            "{:?}: {stage}'s frame ends at {}, its box at {}",
            d.platform(),
            frame.x + frame.width,
            outer.x + outer.width
        );
    }
    for button in ["button[aria-label=Increase]", "button[aria-label=Decrease]"] {
        let rect = d.rect(&format!("#stage {button}")).await?;
        assert!(
            rect.width >= 24.0 || rect.height >= 24.0,
            "{:?}: {button} squeezed to {}x{}",
            d.platform(),
            rect.width,
            rect.height
        );
    }
    Ok(())
}

e2e::scenario!(
    a_field_shrinks_into_a_narrow_flex_row,
    "/field-frame/narrow",
    a_field_shrinks_into_a_narrow_row
);

/// WCAG 1.4.11: the warning state recolours the border, so it must still reach 3:1.
#[test]
fn a_warning_frame_border_at_3_to_1() {
    crate::boundary::assert_boundaries(
        "/field-frame",
        "const frame = document.querySelector('[data-case=warning] [data-frame]');
         const border = CSS(frame, 'borderTopColor');
         return [
             ['warning border on the page', RATIO(border, PAGE(frame))],
             ['warning border on its surface', RATIO(border, CSS(frame, 'backgroundColor'))],
         ];",
    );
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

/// A warning says it is one before its message, not by colour only (1527); an
/// empty label and status draw nothing and mark nothing invalid (1525).
async fn a_warning_says_so_and_a_blank_status_is_none<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    eventually_text(
        d,
        "[data-case=warning] [data-slot=status]",
        "Warning: Already taken",
        "a warning status",
    )
    .await?;
    for part in ["label", "[data-slot=status]"] {
        anyhow::ensure!(
            !d.exists(&format!("[data-case=blank] {part}")).await?,
            "a blank caption drew a {part}"
        );
    }
    let invalid = d.attr("[data-case=blank] input", "aria-invalid").await?;
    anyhow::ensure!(
        invalid.is_none(),
        "a blank status is aria-invalid={invalid:?}"
    );
    Ok(())
}

e2e::scenario!(
    a_warning_says_so_and_a_blank_status_is_none,
    "/field-frame",
    a_warning_says_so_and_a_blank_status_is_none
);
