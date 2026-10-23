//! `CheckboxGroup` and `SwitchGroup` (todo 3984), and a required `Fieldset` (todo 3992).

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, eventually, eventually_text};
use e2e::passes::keyboard;

const BOXES: &str = "[role=group] input[type=checkbox]";
/// Option `n`'s box, 1-based in option order.
fn option_box(n: usize) -> String {
    format!("[role=group] > :nth-child({n}) [data-slot=box]")
}

/// Picks land in option order; at two the rest disable and refuse a click, until one is unchecked.
async fn a_capped_group_holds_its_picks_in_option_order<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    d.click(&option_box(3)).await?;
    eventually_text(d, "#value", "Basil", "Basil checked").await?;
    d.click(&option_box(1)).await?;
    eventually_text(d, "#value", "Cheese,Basil", "option order, not click order").await?;

    eventually(
        d,
        "the two unchecked boxes to disable at the cap",
        async |d| Ok(d.attrs(&format!("{BOXES}:disabled"), "type").await?.len() == 2),
    )
    .await?;
    d.click(&option_box(2)).await?;
    d.flush().await?;
    eventually_text(d, "#value", "Cheese,Basil", "a click past the cap").await?;

    d.click(&option_box(1)).await?;
    eventually_text(d, "#value", "Basil", "Cheese unchecked").await?;
    d.click(&option_box(2)).await?;
    eventually_text(d, "#value", "Olives,Basil", "a freed option checks again").await
}

e2e::scenario!(
    a_capped_checkbox_group_holds_its_picks_in_option_order,
    "/check-group/checkbox",
    a_capped_group_holds_its_picks_in_option_order
);

/// Every box is its own stop: Space checks the focused one.
async fn space_checks_the_focused_box<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("[role=group] > :nth-child(4) input").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#value", "Chili", "Space on the fourth box").await
}

e2e::scenario!(
    space_checks_the_focused_checkbox,
    "/check-group/checkbox",
    space_checks_the_focused_box
);

async fn a_switch_group_toggles_each_switch<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("[role=group] > :nth-child(3) [data-slot=track]")
        .await?;
    eventually_text(d, "#value", "Push,Sms", "Sms on").await?;
    d.click("[role=group] > :nth-child(2) [data-slot=track]")
        .await?;
    eventually_text(d, "#value", "Sms", "Push off").await
}

e2e::scenario!(
    a_switch_group_toggles_each_switch_in_option_order,
    "/check-group/switch",
    a_switch_group_toggles_each_switch
);

/// Each field but the opted-out one says required and shows the asterisk.
async fn the_fields_inherit_required<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "the fieldset's fields to mount", async |d| {
        Ok(d.attrs("fieldset input", "id").await?.len() == 3)
    })
    .await?;
    let required = d.attrs("fieldset input[required]", "id").await?;
    let optional = d.attrs("fieldset input:not([required])", "id").await?;
    let all = d.attrs("fieldset input", "id").await?;
    ensure!(
        required.len() == 2 && optional.len() == 1 && all.last() == optional.first(),
        "required {required:?}, optional {optional:?}: want the last field alone optional"
    );
    let stars = d
        .attrs("fieldset [data-slot=required]", "data-slot")
        .await?;
    ensure!(stars.len() == 2, "{} asterisks, want 2", stars.len());
    Ok(())
}

e2e::scenario!(
    a_required_fieldset_requires_its_fields,
    "/check-group/fieldset-required",
    the_fields_inherit_required
);

#[test]
fn a_checkbox_group_meets_the_baseline() {
    Suite::new("checkbox_group", "/check-group/checkbox")
        .focusable(BOXES)
        .run();
}

#[test]
fn a_switch_group_meets_the_baseline() {
    Suite::new("switch_group", "/check-group/switch")
        .focusable("[role=group] input")
        .run();
}
