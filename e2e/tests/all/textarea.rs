//! `Textarea { counter }` (todo 584): the visible count follows the text, and
//! the polite status speaks only in the last tenth of `maxlength`.
//!
//! `/textarea/counter`: a controlled textarea, then one owning its own text,
//! both `maxlength="20"`.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

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
