//! Shared field props and the removable chip (todo 449 review).

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
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
            .evaluate_bounded(
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

/// A description under the control keeps side-by-side controls level; above, it pushes one down.
#[test]
fn a_description_below_keeps_side_by_side_controls_level() {
    block_on(async {
        let fixture = Fixture::open("/field-props/description", Viewport::Desktop)
            .await
            .unwrap();
        let tops: Vec<f64> = fixture
            .page
            .evaluate_bounded(
                "['below-a', 'below-b', 'above-a', 'above-b'].map(id => document.getElementById(id).getBoundingClientRect().top)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(tops[0], tops[1], "below: {tops:?}");
        assert!(tops[2] > tops[3], "above: {tops:?}");
        fixture.close().await.unwrap();
    });
}

/// Select, MultiSelect, Cascader and PasswordField draw a `below` description under the control (todo 5399).
#[test]
fn a_description_below_follows_the_control_on_select_cascader_and_password() {
    block_on(async {
        let fixture = Fixture::open("/field-props/description-below", Viewport::Desktop)
            .await
            .unwrap();
        let bad: Vec<String> = fixture
            .page
            .evaluate_bounded(
                "['select', 'multi', 'cascader', 'password'].filter(id => { \
                 const root = document.getElementById(id), \
                 control = root.querySelector('input, button, [role=combobox]'), \
                 description = root.querySelector(\"[data-slot='description']\"); \
                 return !description || description.getBoundingClientRect().top < control.getBoundingClientRect().bottom - 1; })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(bad.is_empty(), "description not under the control: {bad:?}");
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
            .evaluate_bounded(
                "document.querySelector(\"button[aria-label='Remove rust']\").click()",
            )
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

/// A `label_trailing` button is the label's sibling, not its child, and ends the label row at 375px.
#[test]
fn a_label_trailing_button_sits_beside_the_label_at_the_row_end() {
    block_on(async {
        let fixture = Fixture::open("/field-props/label-trailing", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetDeviceMetricsOverrideParams::new(375, 780, 1.0, true))
            .await
            .unwrap();
        wait::for_visible(page, "#text-add").await.unwrap();
        let bad: Vec<String> = page
            .evaluate_bounded(
                "(() => { const bad = []; if (innerWidth !== 375) bad.push('viewport ' + innerWidth); \
                 for (const id of ['text', 'area', 'select', 'check', 'bare']) { \
                   const root = document.getElementById(id).firstElementChild, \
                     button = document.getElementById(id + '-add'), \
                     row = button.closest(\"[data-slot='label-row']\"), \
                     label = row && row.querySelector(':scope > label'), \
                     end = button.getBoundingClientRect(); \
                   if (button.closest('label')) bad.push(id + ': in the label'); \
                   if (!row || row.parentElement !== root) { bad.push(id + ': no label row in the field'); continue; } \
                   if (id !== 'bare' && (!label || label.contains(button))) bad.push(id + ': no sibling label'); \
                   if (Math.abs(end.right - root.getBoundingClientRect().right) > 1) bad.push(id + ': not at the end'); \
                   if (label) { const l = label.getBoundingClientRect(); \
                     if (end.left < l.right - 1) bad.push(id + ': overlaps the label'); \
                     if (Math.abs((end.top + end.bottom) / 2 - (l.top + l.bottom) / 2) > 2) bad.push(id + ': off the label line'); } \
                 } return bad; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(bad.is_empty(), "{bad:?}");
        fixture.close().await.unwrap();
    });
}
