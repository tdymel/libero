//! `Textarea { counter }` (584): the count follows the text; the polite status speaks only
//! in the last tenth of `maxlength`. Controlled and uncontrolled, both 20.

use anyhow::Result;
use anyhow::ensure;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually_text, linger};
use e2e::passes::keyboard::{self, ARROW_DOWN, ARROW_UP, BACKSPACE, END, ENTER, HOME};
use e2e::{Fixture, Suite, Viewport, wait};

async fn typing_reaches_the_value<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("textarea").await?;
    d.type_text("hi").await?;
    eventually_text(d, "#echo", "hi", "typing hi").await
}

e2e::scenario!(
    typing_into_a_textarea_reaches_its_value,
    "/textarea/echo",
    typing_reaches_the_value
);

/// Todo 1201: WebKitGTK scrolled the nearest scroller on an arrow the caret cannot
/// follow past a field's edge (1178); an ArrowUp on the first line, an ArrowDown on the last.
async fn an_arrow_at_an_edge_line_stays<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let area = "#scroller textarea";
    d.focus(area).await?;
    for (to, arrow) in [(HOME, ARROW_UP), (END, ARROW_DOWN)] {
        d.press_ctrl(to).await?;
        if !d.frame().await? {
            linger(d, 5).await;
        }
        let before = d.rect(area).await?.y;
        d.press(arrow).await?;
        if !d.frame().await? {
            linger(d, 10).await;
        }
        let after = d.rect(area).await?.y;
        ensure!(
            (after - before).abs() < 1.0 && d.is_focused(area).await?,
            "{:?}: {} on an edge line scrolled the textarea from {before} to {after}",
            d.platform(),
            arrow.key
        );
    }
    Ok(())
}

e2e::scenario!(
    an_arrow_on_a_textareas_edge_line_leaves_the_scroller,
    "/textarea/scroller",
    an_arrow_at_an_edge_line_stays
);

/// Todo 942: Blitz's editor ignored `maxlength`, so natively the count ran to
/// 23/20. A textarea's Enter is a character too; a freed one can be typed again.
async fn typing_stops_at_maxlength<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("textarea").await?;
    d.type_text("0123456789012345678xyz").await?;
    eventually_text(d, "[data-slot=counter]", "20/20", "typing past the limit").await?;
    d.press(ENTER).await?;
    d.settle().await?;
    eventually_text(d, "[data-slot=counter]", "20/20", "Enter at the limit").await?;
    d.press(BACKSPACE).await?;
    eventually_text(d, "[data-slot=counter]", "19/20", "a Backspace").await?;
    d.type_text("!?").await?;
    eventually_text(d, "[data-slot=counter]", "20/20", "a freed character").await?;

    d.click("input").await?;
    d.type_text("abcdef").await?;
    eventually_text(d, "#code", "abcd", "typing past a text field's limit").await
}

e2e::scenario!(
    typing_stops_at_maxlength_on_every_platform,
    "/textarea/counter",
    typing_stops_at_maxlength
);

/// The counter and the status text of the field around textarea `index`.
fn counter_reads(index: usize, count: &str, spoken: &str) -> String {
    format!(
        "(() => {{ const field = document.querySelectorAll('textarea')[{index}] \
           .closest('[data-frame]').parentElement; \
         const counter = field.querySelector('[data-slot=counter]'); \
         const status = field.querySelector('[role=status]'); \
         return counter.getAttribute('aria-hidden') === 'true' \
           && counter.textContent.trim() === {count:?} \
           && status.textContent.trim() === {spoken:?}; }})()"
    )
}

#[test]
fn the_counter_speaks_only_near_the_limit() {
    block_on(async {
        let fixture = Fixture::open("/textarea/counter", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        for (index, which) in ["controlled", "uncontrolled"].into_iter().enumerate() {
            let reads = |count: &str, spoken: &str| counter_reads(index, count, spoken);
            wait::for_js_true(page, &reads("0/20", ""), "an empty count")
                .await
                .unwrap();
            page.evaluate(format!(
                "document.querySelectorAll('textarea')[{index}].focus()"
            ))
            .await
            .unwrap();
            keyboard::type_text(page, "0123456789").await.unwrap();
            wait::for_js_true(page, &reads("10/20", ""), &format!("{which}: nothing said"))
                .await
                .unwrap();
            keyboard::type_text(page, "01234567").await.unwrap();
            wait::for_js_true(
                page,
                &reads("18/20", "2 characters left"),
                &format!("{which}: two left said"),
            )
            .await
            .unwrap();
            keyboard::type_text(page, "8").await.unwrap();
            wait::for_js_true(
                page,
                &reads("19/20", "1 character left"),
                &format!("{which}: the singular"),
            )
            .await
            .unwrap();
            // `maxlength` stops the text at 20.
            keyboard::type_text(page, "9xyz").await.unwrap();
            wait::for_js_true(
                page,
                &reads("20/20", "0 characters left"),
                &format!("{which}: the limit"),
            )
            .await
            .unwrap();
        }

        fixture.console.assert_clean("typing to the limit").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Axe, contrast and focus rings, light and dark, desktop and mobile. Axe
/// skips the `aria-hidden` counter.
#[test]
fn it_meets_the_baseline() {
    Suite::new("textarea", "/textarea/counter")
        .focusable("textarea")
        .run();
}

/// Todo 685: a raw `<form>`'s own reset reaches the count too, with no libero
/// `Form` to say so.
#[test]
fn an_uncontrolled_count_follows_a_raw_form_reset() {
    block_on(async {
        let fixture = Fixture::open("/textarea/raw-reset", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let reads = |count: &str| counter_reads(0, count, "");

        wait::for_js_true(page, &reads("2/20"), "the initial text")
            .await
            .unwrap();
        for round in 0..2 {
            page.evaluate("document.querySelector('textarea').focus()")
                .await
                .unwrap();
            keyboard::type_text(page, "hello").await.unwrap();
            wait::for_js_true(page, &reads("7/20"), &format!("typed, round {round}"))
                .await
                .unwrap();
            page.evaluate("document.querySelector('button[type=reset]').click()")
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelector('textarea').value === 'hi' && {}",
                    reads("2/20")
                ),
                &format!("the initial count, round {round}"),
            )
            .await
            .unwrap();
        }

        fixture.console.assert_clean("a raw reset").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 679: a reset (native button or `FormHandle`) restores an uncontrolled textarea
/// without an input event, and the count follows.
#[test]
fn an_uncontrolled_count_follows_a_form_reset() {
    block_on(async {
        let fixture = Fixture::open("/textarea/reset", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let reads = |count: &str| counter_reads(0, count, "");

        wait::for_js_true(page, &reads("2/20"), "the initial text")
            .await
            .unwrap();
        for button in ["Clear", "Reset"] {
            page.evaluate("document.querySelector('textarea').focus()")
                .await
                .unwrap();
            keyboard::type_text(page, "hello").await.unwrap();
            wait::for_js_true(page, &reads("7/20"), "typed")
                .await
                .unwrap();
            page.evaluate(format!(
                "[...document.querySelectorAll('button')].find(b => b.textContent.trim() === {button:?}).click()"
            ))
            .await
            .unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelector('textarea').value === 'hi' && {}",
                    reads("2/20")
                ),
                &format!("{button}: the initial count"),
            )
            .await
            .unwrap();
        }

        fixture.console.assert_clean("resetting").unwrap();
        fixture.close().await.unwrap();
    });
}
