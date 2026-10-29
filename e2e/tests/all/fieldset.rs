//! A Fieldset keeps its fields mounted when its description or legend toggles (4926).

use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

#[test]
fn toggling_a_slot_keeps_the_fields_mounted() {
    block_on(async {
        let fixture = Fixture::open("/fieldset", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#fieldset input").await.unwrap();
        let street = page
            .evaluate_bounded(
                "(() => { const input = document.querySelector('#fieldset input'); \
                 input.focus(); window.street = input; return input.id; })()",
            )
            .await
            .unwrap()
            .into_value::<String>()
            .unwrap();

        // A scripted click moves no focus, so the Street field keeps it across the toggle.
        for (toggle, slot, states) in [
            ("toggle-description", "description", [true, false]),
            ("toggle-legend", "legend", [false, true]),
        ] {
            for shown in states {
                let present = match shown {
                    true => "!== null",
                    false => "=== null",
                };
                page.evaluate_bounded(format!("document.getElementById('{toggle}').click()"))
                    .await
                    .unwrap();
                wait::for_js_true(
                    page,
                    &format!("document.querySelector('#fieldset > [data-slot={slot}]') {present}"),
                    &format!("{slot} toggled"),
                )
                .await
                .unwrap();
                let kept = page
                    .evaluate_bounded(format!(
                        "document.activeElement === window.street && window.street.isConnected \
                         && window.street.id === '{street}'"
                    ))
                    .await
                    .unwrap()
                    .into_value::<bool>()
                    .unwrap();
                assert!(kept, "toggling the {slot} remounted the Street field");
            }
        }
    });
}
