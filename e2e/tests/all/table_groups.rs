//! `Table` column groups and column spans (1156-2e): a group header is as wide
//! as its columns, a spanning cell as the columns it covers, and a capped table
//! keeps all its header rows stacked over the scrolled rows.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;

const AREA: &str = "[data-table-scroll]";
const PLACE: &str = "thead tr:nth-child(1) th[data-group]";
const CITY: &str = "thead tr:nth-child(2) th:nth-child(1)";
const REGION: &str = "thead tr:nth-child(2) th:nth-child(2)";
const HALF: &str = "thead tr:nth-child(2) th[data-group]";
const Q1: &str = "thead tr:nth-child(3) th:nth-child(1)";
const TOTAL: &str = "tbody tr:last-child th";
const SELECT_ALL: &str = "thead input[type=checkbox]";

async fn the_header_rows_stack_and_stick<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let place = d.rect(PLACE).await?.width;
    let columns = d.rect(CITY).await?.width + d.rect(REGION).await?.width;
    if (place - columns).abs() > 1.0 {
        bail!("the Place group is {place}px wide over {columns}px of columns");
    }
    let total = d.rect(TOTAL).await?.width;
    if (total - columns).abs() > 1.0 {
        bail!("the spanning total cell is {total}px wide over {columns}px of columns");
    }

    // The select-all box sits in the header, so the rows scroll from it.
    let row = d.rect("tbody td").await?.y;
    d.focus(SELECT_ALL).await?;
    eventually_focused(d, SELECT_ALL, "focus").await?;
    d.press(keyboard::PAGE_DOWN).await?;
    eventually(d, "the rows to scroll under the header rows", async |d| {
        let area = d.rect(AREA).await?.y;
        let top = d.rect(PLACE).await?.y;
        let half = d.rect(HALF).await?;
        let q1 = d.rect(Q1).await?.y;
        Ok(d.rect("tbody td").await?.y < row - 20.0
            && (top - area).abs() <= 1.0
            && (q1 - (half.y + half.height)).abs() <= 1.0)
    })
    .await?;
    let background = d.style(Q1, "background-color").await?;
    if background == "rgba(0, 0, 0, 0)" || background == "transparent" {
        bail!("the sticky header is see-through: {background}");
    }
    Ok(())
}

e2e::scenario!(
    a_grouped_header_spans_its_columns_and_sticks,
    "/table-groups",
    the_header_rows_stack_and_stick,
    android: skip("958: element identity on the WebView"),
    native: skip("Blitz scrolls on no key and sticks the last header row only: tests/native/table_groups.rs")
);

const PINNED_GROUP: &str = "thead tr:nth-child(1) th[data-group][data-pin=start]";
const REVENUE: &str = "thead tr:nth-child(1) th[data-group]:not([data-pin])";

/// Todo 1449: scrolled sideways, the pinned Place group holds over its pinned
/// columns while Revenue slides under it.
async fn the_pinned_group_holds<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let area = d.rect(AREA).await?.x;
    let place = d.rect(PINNED_GROUP).await?;
    let revenue = d.rect(REVENUE).await?.x;
    let columns = d.rect(CITY).await?.width + d.rect(REGION).await?.width;
    if (place.x - area).abs() > 1.0 || (place.width - columns).abs() > 1.0 {
        bail!(
            "Place is at {} and {}px wide; the area at {area}, its columns {columns}px",
            place.x,
            place.width
        );
    }
    d.focus("th[data-sortable] button").await?;
    for _ in 0..4 {
        d.press(keyboard::ARROW_RIGHT).await?;
    }
    let slid = eventually(
        d,
        "Revenue to slide under the held Place group",
        async |d| {
            let held = d.rect(PINNED_GROUP).await?.x;
            let city = d.rect(CITY).await?.x;
            Ok(d.rect(REVENUE).await?.x < revenue - 40.0
                && (held - place.x).abs() <= 1.0
                && (city - place.x).abs() <= 1.0)
        },
    )
    .await;
    if slid.is_err() {
        bail!(
            "Revenue {revenue} -> {}, Place {} -> {}, City at {}",
            d.rect(REVENUE).await?.x,
            place.x,
            d.rect(PINNED_GROUP).await?.x,
            d.rect(CITY).await?.x
        );
    }
    let background = d.style(PINNED_GROUP, "background-color").await?;
    if background == "rgba(0, 0, 0, 0)" || background == "transparent" {
        bail!("the pinned group is see-through: {background}");
    }
    Ok(())
}

e2e::scenario!(
    a_pinned_group_holds_while_the_body_scrolls_sideways,
    "/table-groups/pinned",
    the_pinned_group_holds,
    android: skip("958: element identity on the WebView")
);
