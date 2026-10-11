//! `Table` row grouping (1156 G1): a group row's toggle opens its rows under it,
//! says so, and keeps the focus; the sort reorders the groups. G2: the column
//! menu groups by a column and stops. G4: a group row sums its rows under the column.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;

use crate::table::menu_entry;

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

/// G2: "Group by Team" groups the rows and says so; "Stop grouping by Team" undoes it.
/// Name opted out with `row_groupable(false)`.
async fn menu_groups<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("button[aria-label=\"Team column options\"]")
        .await?;
    let entry = menu_entry(d, "Group by Team").await?;
    d.click(&entry).await?;
    eventually_text(d, "#grouping", "Team", "Group by Team").await?;
    eventually_text(d, LIVE, "Rows grouped by Team", "the grouping").await?;
    eventually(d, "the group rows", async |d| {
        Ok(d.exists(SALES).await? && !d.exists("tbody th").await?)
    })
    .await?;

    d.click("button[aria-label=\"Name column options\"]")
        .await?;
    eventually(d, "Name's menu", async |d| {
        d.exists("[role=menuitem]").await
    })
    .await?;
    for index in 0..12 {
        let item = format!("[role=menuitem][data-menu-index=\"{index}\"]");
        if d.exists(&item).await? && d.text(&item).await?.contains("Group") {
            bail!("Name opted out of grouping but its menu offers it");
        }
    }
    d.press(keyboard::ESCAPE).await?;

    d.click("button[aria-label=\"Team column options\"]")
        .await?;
    let entry = menu_entry(d, "Stop grouping by Team").await?;
    d.click(&entry).await?;
    eventually_text(d, "#grouping", "", "Stop grouping by Team").await?;
    eventually(d, "the plain rows", async |d| {
        Ok(!d.exists("tr[data-row-group]").await? && d.exists("tbody th").await?)
    })
    .await
}

e2e::scenario!(
    the_column_menu_groups_by_a_column,
    "/table-row-groups/menu",
    menu_groups
);

/// G4: Sales' row shows the sum of its two ages in the Age column, right-aligned
/// with the ages under it once open.
async fn sums_under_the_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let sum = "tbody tr:first-child td:last-child [data-group-aggregate]";
    eventually_text(d, sum, "Sum55", "Sales' Age sum").await?;
    d.click(SALES).await?;
    eventually_text(d, "tbody tr:nth-child(2) th", "Ada", "the opened rows").await?;
    let cell = d.rect("tbody tr:first-child td:last-child").await?;
    let age = d.rect("tbody tr:nth-child(2) td:last-child").await?;
    let text = d.rect(sum).await?;
    if (cell.x - age.x).abs() > 1.0 || (cell.width - age.width).abs() > 1.0 {
        bail!("the sum's cell {cell:?} is not over the Age cells {age:?}");
    }
    if (text.x + text.width - (cell.x + cell.width)).abs() > 24.0 {
        bail!("the sum {text:?} does not sit at the end of its cell {cell:?}");
    }
    Ok(())
}

e2e::scenario!(
    a_group_row_sums_its_rows_under_the_column,
    "/table-row-groups",
    sums_under_the_column
);
