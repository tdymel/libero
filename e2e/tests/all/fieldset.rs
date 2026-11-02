//! A Fieldset keeps its fields mounted when its description or legend toggles (4926), and
//! leaving it reveals its named rules (5246).

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Todo 5246: leaving the group shows a named rule on the field it names, before that field
/// was ever focused, and only there; a reset hides it until the group is left again.
#[test]
fn leaving_the_group_reveals_a_named_rule_until_a_reset() {
    block_on(async {
        let fixture = Fixture::open("/fieldset/rules", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#city").await.unwrap();
        let focus = async |id: &str| {
            page.evaluate_bounded(format!("document.getElementById('{id}').focus()"))
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!("document.activeElement?.id === '{id}'"),
                &format!("focus on {id}"),
            )
            .await
            .unwrap();
        };
        let invalid = |id: &str| format!("document.getElementById('{id}').ariaInvalid === 'true'");

        focus("city").await;
        keyboard::type_text(page, "Bonn").await.unwrap();
        focus("street").await;
        let hidden = page
            .evaluate_bounded(format!("!({})", invalid("zip")))
            .await
            .unwrap()
            .into_value::<bool>()
            .unwrap();
        assert!(hidden, "the rule showed while focus was in the group");

        focus("outside").await;
        wait::for_js_true(
            page,
            &format!(
                "{} && document.body.textContent.includes('A city needs its zip code.')",
                invalid("zip")
            ),
            "the rule on Zip once focus left the group",
        )
        .await
        .unwrap();
        let others = page
            .evaluate_bounded(format!("{} || {}", invalid("street"), invalid("city")))
            .await
            .unwrap()
            .into_value::<bool>()
            .unwrap();
        assert!(!others, "a field the rule does not name turned invalid");

        page.evaluate_bounded("document.getElementById('form').reset()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "!({}) && !document.body.textContent.includes('A city needs its zip code.')",
                invalid("zip")
            ),
            "the reset to hide the rule",
        )
        .await
        .unwrap();

        // Failing again after the reset: it waits for another leave.
        focus("city").await;
        keyboard::type_text(page, "Bonn").await.unwrap();
        focus("street").await;
        let hidden = page
            .evaluate_bounded(format!("!({})", invalid("zip")))
            .await
            .unwrap()
            .into_value::<bool>()
            .unwrap();
        assert!(hidden, "the reset kept the rule revealed");
        focus("outside").await;
        wait::for_js_true(page, &invalid("zip"), "the rule on a second leave")
            .await
            .unwrap();
        fixture.console.assert_clean("a named rule").unwrap();
        fixture.close().await.unwrap();
    });
}

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
