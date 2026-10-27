//! The page's next Tab stop past a picker sheet: a radio group whose checked radio is hidden
//! (todo 4601) and one tied to a form by its `form` attribute (4600).

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::passes::pointer::tap;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

const TYPED: &str = "[role=dialog] [data-slot=text-input]";

/// Tabs out of field `field`'s sheet from its text box; the id focus lands on must be `lands`.
async fn tab_out(field: &str, lands: &str, what: &str) {
    let fixture = Fixture::open("/tab-stops", Viewport::Mobile).await.unwrap();
    let page = &fixture.page;
    page.execute_bounded(SetDeviceMetricsOverrideParams::new(360, 780, 1.0, true))
        .await
        .unwrap();
    tap(page, &format!("#{field} input[data-controlled]"))
        .await
        .unwrap();
    let shown = format!("!!document.querySelector({TYPED:?})");
    wait::for_js_true(page, &shown, "the sheet's text box")
        .await
        .unwrap();
    tap(page, TYPED).await.unwrap();
    let focused = format!("document.querySelector({TYPED:?}) === document.activeElement");
    wait::for_js_true(page, &focused, "the text box focused by the tap")
        .await
        .unwrap();
    keyboard::press(page, keyboard::TAB).await.unwrap();
    let landed = format!("document.activeElement?.id === {lands:?}");
    wait::for_js_true(page, &landed, what).await.unwrap();
    fixture.console.assert_clean(what).unwrap();
    fixture.close().await.unwrap();
}

/// Todo 4601: the group's checked radio is `display: none`, so its next radio takes the stop.
#[test]
fn a_group_whose_checked_radio_is_hidden_is_still_a_stop() {
    block_on(tab_out("hidden", "medium", "the group's next radio"));
}

/// Todo 4600: the radio outside the form names it, so the form's checked radio is the stop.
#[test]
fn a_radio_joins_the_form_its_form_attribute_names() {
    block_on(tab_out(
        "owned",
        "held",
        "the checked radio of the named form's group",
    ));
}
