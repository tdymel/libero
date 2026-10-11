//! `Table` row grouping (1156 G1): a group row's toggle opens its rows under it,
//! says so, and keeps the focus; the sort reorders the groups. G2: the column
//! menu groups by a column and stops. G4: a group row sums its rows under the column.

use anyhow::{Result, bail};
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
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
    // The group's box sits in the checkbox column, with no lead cell beside it.
    if d.exists("tr[data-row-group] td[data-group-lead]").await? {
        bail!("a lead cell beside the group's box");
    }
    let group = d.rect("tr[data-row-group] td[data-select]").await?;
    let check = d.rect("tbody tr:nth-child(2) td[data-select]").await?;
    if (group.x - check.x).abs() > 1.0 || (group.width - check.width).abs() > 1.0 {
        bail!("the group's box cell {group:?} is not over the row boxes {check:?}");
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
    sort_orders_groups,
    ios: skip("3416: the simulator's tap or drag did not drive the page")
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
    menu_groups,
    ios: skip("3416: the simulator's tap or drag did not drive the page")
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

const SALES_BOX: &str = "input[aria-label=\"Select group Sales, 2 rows\"]";
/// Sales' drawn box: Sales leads the rows, and Blitz cannot parse `:has(...)`.
const SALES_BOX_MARK: &str =
    "tbody tr[data-row-group]:first-child td[data-select] span[aria-hidden]";

/// G3: a closed group's box selects its rows and says so; one row off mixes it.
async fn box_selects_the_group<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(SALES_BOX_MARK).await?;
    eventually_text(d, "#selection", "Ada,Cy", "a click on Sales' box").await?;
    eventually_text(d, LIVE, "2 rows selected", "the group's selection").await?;
    eventually_focused(d, SALES_BOX, "a click on Sales' box").await?;
    d.click(SALES).await?;
    eventually_text(d, "tbody tr:nth-child(2) th", "Ada", "the opened rows").await?;
    d.click("tbody tr:nth-child(2) td[data-select] span[aria-hidden]")
        .await?;
    eventually_text(d, "#selection", "Cy", "a click on Ada's box").await?;
    eventually(d, "Sales' box to mix", async |d| match d.platform() {
        Platform::Native => {
            Ok(d.attr(SALES_BOX, "aria-checked").await?.as_deref() == Some("mixed"))
        }
        _ => Ok(d
            .evaluate(&format!(
                "document.querySelector({SALES_BOX:?}).indeterminate"
            ))
            .await?
            == true),
    })
    .await?;
    // A mixed box selects the rest.
    d.click(SALES_BOX_MARK).await?;
    eventually_text(d, "#selection", "Cy,Ada", "a click on the mixed box").await
}

e2e::scenario!(
    a_group_box_selects_the_groups_rows,
    "/table-row-groups",
    box_selects_the_group
);

/// G3: 10k rows in four open groups render a window; the group rows take the 40px
/// pitch and count in `aria-rowindex`, and End brings the last row in.
async fn windowed_groups<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const SALES_ROW: &str = "tr[aria-rowindex=\"2\"][data-row-group]";
    const LAST: &str = "tr[aria-rowindex=\"10005\"]";
    eventually(d, "only the first rows to render", async |d| {
        Ok(
            d.attr("table", "aria-rowcount").await?.as_deref() == Some("10005")
                && d.exists(SALES_ROW).await?
                && !d.exists("tr[aria-rowindex=\"60\"]").await?,
        )
    })
    .await?;
    let group = d.rect(&format!("{SALES_ROW} td:last-child")).await?;
    if (group.height - 40.0).abs() > 1.0 {
        bail!("a group row is {}px, not its 40px row_height", group.height);
    }
    // Ops follows Sales' 2500 rows.
    let ops = "tr[aria-rowindex=\"2503\"][data-row-group]";
    let sales_toggle = "button[aria-label=\"Sales, 2500 rows\"]";
    d.focus(sales_toggle).await?;
    d.press(keyboard::END).await?;
    eventually(d, "the last row to show", async |d| {
        Ok(d.exists(LAST).await? && !d.exists("tr[aria-rowindex=\"3\"]").await?)
    })
    .await?;
    // The focused group row stays rendered out of view.
    eventually_focused(d, sales_toggle, "End").await?;
    d.press(keyboard::HOME).await?;
    eventually(d, "the first rows to come back", async |d| {
        Ok(d.exists("tr[aria-rowindex=\"3\"]").await? && !d.exists(LAST).await?)
    })
    .await?;
    if d.exists(ops).await? {
        bail!("Ops' group row rendered at the top");
    }
    d.click(&format!("{SALES_ROW} td[data-select] span[aria-hidden]"))
        .await?;
    eventually_text(d, "#selection", "2500", "a click on Sales' box").await
}

e2e::scenario!(
    a_windowed_grouped_table_renders_the_rows_in_view,
    "/table-row-groups/windowed",
    windowed_groups,
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);

/// 3303: a group closed above the focused row leaves the row where it was.
async fn held_in_view<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // Person 52: Ops' third row, after Sales' group row and 50 rows and Ops' group row.
    const ROW: &str = "tr[aria-rowindex=\"56\"]";
    let input = format!("{ROW} td[data-select] input");
    d.scroll_to("[data-table-scroll]", 0.0, 52.0 * 40.0).await?;
    eventually(d, "Ops' rows to render", async |d| d.exists(&input).await).await?;
    d.focus(&input).await?;
    eventually_focused(d, &input, "focus").await?;
    let before = d.rect(ROW).await?;
    d.type_text("x").await?;
    // The header row and 154 body rows.
    eventually(d, "Sales to close", async |d| {
        Ok(d.attr("table", "aria-rowcount").await?.as_deref() == Some("155"))
    })
    .await?;
    // Sales' 50 rows gone, Person 52 sits 50 places up.
    const MOVED: &str = "tr[aria-rowindex=\"6\"]";
    eventually_focused(d, &format!("{MOVED} td[data-select] input"), "the close").await?;
    eventually(d, "the focused row where it was", async |d| {
        Ok(d.text(&format!("{MOVED} th")).await?.trim() == "Person 52"
            && (d.rect(MOVED).await?.y - before.y).abs() <= 2.0)
    })
    .await
}

e2e::scenario!(
    a_group_closed_above_the_focused_row_leaves_it_in_place,
    "/table-row-groups/held",
    held_in_view
);
