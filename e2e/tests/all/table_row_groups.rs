//! `Table` row grouping (1156 G1): a group row's toggle opens its rows under it,
//! says so, and keeps the focus; the sort reorders the groups.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;

const SALES: &str = "button[aria-label=\"Sales, 2 rows\"]";
const LIVE: &str = "table + [role=status]";

async fn opens_under_its_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    if d.exists("tbody th").await? {
        bail!("a group starts open");
    }
    d.click(SALES).await?;
    eventually_text(d, "#expanded", "Sales", "a toggle click").await?;
    eventually_text(d, LIVE, "Sales expanded", "the opened group").await?;
    // Sales, Ada, Cy, then the closed Ops and (empty) groups.
    eventually_text(d, "tbody tr:nth-child(2) th", "Ada", "the opened rows").await?;
    eventually_text(d, "tbody tr:nth-child(3) th", "Cy", "the opened rows").await?;
    if d.attr("tbody tr:nth-child(4)", "data-row-group")
        .await?
        .is_none()
    {
        bail!("Ops' group row does not follow Sales' rows");
    }
    if d.attr(SALES, "aria-expanded").await?.as_deref() != Some("true") {
        bail!("the open toggle is not expanded");
    }
    // One lead cell spans the checkbox column; the group row has no box.
    if d.exists("tr[data-row-group] input[type=checkbox]").await? {
        bail!("a group row has a checkbox");
    }
    let lead = d.rect("tr[data-row-group] td[data-group-lead]").await?;
    let check = d.rect("tbody tr:nth-child(2) td[data-select]").await?;
    if (lead.width - check.width).abs() > 1.0 {
        bail!("the lead cell {lead:?} is not as wide as the checkbox column {check:?}");
    }
    d.click(SALES).await?;
    eventually_text(d, "#expanded", "", "a second click").await?;
    eventually_text(d, LIVE, "Sales collapsed", "the closed group").await?;
    eventually(d, "the rows to go", async |d| {
        Ok(!d.exists("tbody th").await?)
    })
    .await
}

e2e::scenario!(
    a_group_toggle_opens_its_rows_under_it,
    "/table-row-groups",
    opens_under_its_row
);

/// Enter opens and Space closes; the focus stays on the toggle through both.
async fn keyboard_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SALES).await?;
    eventually_focused(d, SALES, "focus").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#expanded", "Sales", "Enter").await?;
    eventually_focused(d, SALES, "Enter").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#expanded", "", "Space").await?;
    eventually_focused(d, SALES, "Space").await
}

e2e::scenario!(
    the_keyboard_toggles_a_group_and_keeps_the_focus,
    "/table-row-groups",
    keyboard_toggles
);

/// Groups follow the sorted rows: Name descending puts Dan's empty group first.
async fn sort_orders_groups<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(
        d,
        "tbody tr:first-child [data-group-label]",
        "Sales",
        "source order",
    )
    .await?;
    let sort = "th[data-sortable] [data-sort-button]";
    d.click(sort).await?;
    d.click(sort).await?;
    eventually_text(
        d,
        "tbody tr:first-child [data-group-label]",
        "(empty)",
        "Name descending",
    )
    .await
}

e2e::scenario!(
    the_sort_orders_the_groups,
    "/table-row-groups",
    sort_orders_groups
);
