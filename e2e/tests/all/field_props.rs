//! Shared field props and the removable chip (todo 449 review).

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

use crate::rtl_keys::open_in;

/// A read-only field keeps its tab stop and says so, and none is announced as disabled.
#[test]
fn read_only_fields_stay_reachable_and_are_not_disabled() {
    block_on(async {
        let fixture = Fixture::open("/field-props/readonly", Viewport::Desktop)
            .await
            .unwrap();
        let bad: Vec<String> = fixture
            .page
            .evaluate(
                "[...document.querySelectorAll('#text, #area, #rating, #range-label-thumb-0, #range-label-thumb-1, #cascader, #date')] \
                 .filter(e => e.tabIndex !== 0 || e.disabled || e.getAttribute('aria-disabled') === 'true' \
                     || !(e.readOnly || e.getAttribute('aria-readonly') === 'true')).map(e => e.id)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(bad.is_empty(), "not read-only and reachable: {bad:?}");
        fixture.close().await.unwrap();
    });
}

/// The x sits at the end of its chip in both directions.
#[test]
fn the_remove_button_ends_its_chip_in_both_directions() {
    block_on(async {
        for dir in ["ltr", "rtl"] {
            let fixture = open_in("/field-props/chips", dir).await;
            let condition = format!(
                "[...document.querySelectorAll(\"[data-slot='remove'] button\")].every(b => {{ \
                 const l = b.closest('[data-slot=tag],[data-slot=chip]').querySelector(\"[data-slot='label']\").getBoundingClientRect(), \
                 r = b.getBoundingClientRect(); \
                 return {rtl} ? r.right <= l.left : r.left >= l.right; }})",
                rtl = dir == "rtl"
            );
            wait::for_js_true(
                &fixture.page,
                &condition,
                &format!("{dir}: x after the label"),
            )
            .await
            .unwrap();
            fixture.console.assert_clean(dir).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Removing a chip with its x says which one went.
#[test]
fn removing_a_chip_is_announced() {
    block_on(async {
        let fixture = Fixture::open("/field-props/chips", Viewport::Desktop)
            .await
            .unwrap();
        fixture
            .page
            .evaluate("document.querySelector(\"button[aria-label='Remove rust']\").click()")
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "[...document.querySelectorAll('[role=status]')].some(s => s.textContent === 'Removed rust')",
            "the removal said",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
