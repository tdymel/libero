//! `Table` column groups and column spans (1156-2e): a group header is as wide
//! as its columns, a spanning cell as the columns it covers, and a capped table
//! keeps all its header rows stacked over the scrolled rows.

use anyhow::{Result, bail};
use e2e::driver::{Driver, Platform, eventually, eventually_focused};
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

const FOOT_LEAD: &str = "tfoot td[data-footer-lead]";
const FOOT_Q1: &str = "tfoot td:nth-child(3)";
const LAST_ROW: &str = "tbody tr[aria-rowindex=\"201\"]";
const UNDER_ROW: &str = "tbody tr[aria-rowindex=\"6\"]";
const UNDER_BOX: &str = "tbody tr[aria-rowindex=\"6\"] input[type=checkbox]";
const HEAD_Q1: &str = "thead th:nth-child(3)";

/// Todo 2736: the footer lines up under its columns, sums every row and sticks to
/// the bottom of the capped table while the rows scroll under it.
async fn the_footer_aggregates_and_sticks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let sum = d.text(FOOT_Q1).await?;
    if !sum.contains("465") {
        bail!("the Q1 footer reads {sum:?}, not the sum 465 of all thirty rows");
    }
    let (foot, head) = (d.rect(FOOT_Q1).await?, d.rect(HEAD_Q1).await?);
    if (foot.x - head.x).abs() > 1.0 || (foot.width - head.width).abs() > 1.0 {
        bail!(
            "the footer cell is at {} and {}px wide under a header at {} and {}px",
            foot.x,
            foot.width,
            head.x,
            head.width
        );
    }
    let select = d.rect("thead th:nth-child(1)").await?.width;
    let lead = d.rect(FOOT_LEAD).await?.width;
    if (lead - select).abs() > 1.0 {
        bail!("the lead footer cell is {lead}px wide over a {select}px checkbox column");
    }
    let area = d.rect(AREA).await?;
    let bottom = d.rect(FOOT_Q1).await?;
    if ((area.y + area.height) - (bottom.y + bottom.height)).abs() > 2.0 {
        bail!(
            "the footer ends at {}, the area at {}",
            bottom.y + bottom.height,
            area.y + area.height
        );
    }
    let row = d.rect("tbody td").await?.y;
    d.focus(SELECT_ALL).await?;
    d.press(keyboard::PAGE_DOWN).await?;
    eventually(d, "the rows to scroll while the footer stays", async |d| {
        let foot = d.rect(FOOT_Q1).await?;
        let area = d.rect(AREA).await?;
        let edge = area.y + area.height;
        Ok(
            d.rect("tbody td").await?.y < row - 20.0
                && (edge - (foot.y + foot.height)).abs() <= 2.0,
        )
    })
    .await?;
    let background = d.style(FOOT_Q1, "background-color").await?;
    if background == "rgba(0, 0, 0, 0)" || background == "transparent" {
        bail!("the sticky footer is see-through: {background}");
    }
    Ok(())
}

e2e::scenario!(
    a_footer_lines_up_under_its_columns_and_sticks,
    "/table-groups/footer",
    the_footer_aggregates_and_sticks,
    android: skip("958: element identity on the WebView")
);

/// Todo 2754: a windowed table's footer holds at the bottom, the last row scrolls
/// to just above it, and the footer is the last counted row.
async fn the_windowed_footer_holds_and_ends_the_rows<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    let count = d.attr("table", "aria-rowcount").await?;
    if count.as_deref() != Some("202") {
        bail!("aria-rowcount is {count:?}, not the header, 200 rows and the footer");
    }
    let index = d.attr("tfoot tr", "aria-rowindex").await?;
    if index.as_deref() != Some("202") {
        bail!("the footer row is {index:?}, not row 202");
    }
    let area = d.rect(AREA).await?;
    let edge = area.y + area.height;
    let foot = d.rect(FOOT_Q1).await?;
    if (edge - (foot.y + foot.height)).abs() > 2.0 {
        bail!(
            "the footer ends at {}, the area at {edge}",
            foot.y + foot.height
        );
    }
    eventually(d, "the last row above the footer", async |d| {
        d.wheel("tbody tr", 20000.0).await?;
        if !d.exists(LAST_ROW).await? {
            return Ok(false);
        }
        let (row, foot) = (d.rect(LAST_ROW).await?, d.rect(FOOT_Q1).await?);
        Ok(row.y + row.height <= foot.y + 1.0)
    })
    .await?;
    let foot = d.rect(FOOT_Q1).await?;
    if (edge - (foot.y + foot.height)).abs() > 2.0 {
        bail!(
            "the footer ends at {}, the area at {edge}, after the scroll",
            foot.y + foot.height
        );
    }
    Ok(())
}

/// Todo 2754 (WCAG 2.4.11): a focus scroll brings a row under the footer up to just above it.
async fn a_focused_row_clears_the_footer<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (row, foot) = (d.rect(UNDER_ROW).await?, d.rect(FOOT_Q1).await?);
    if row.y + row.height <= foot.y {
        bail!("the row at {} is not under the footer at {}", row.y, foot.y);
    }
    eventually(d, "the scroll area to keep the footer clear", async |d| {
        let pad = d.style(AREA, "scroll-padding-bottom").await?;
        Ok(pad != "auto" && pad != "0px")
    })
    .await?;
    // Chromium centres a focused control, clearing its row; Blitz stops at the padding, so
    // there the control itself is what clears. Its own focus scroll ignores the padding
    // (1517): a Tab move reveals with it.
    let cleared = match d.platform() {
        Platform::Native => {
            d.focus(SELECT_ALL).await?;
            for _ in 0..40 {
                if d.is_focused(UNDER_BOX).await? {
                    break;
                }
                d.press(keyboard::TAB).await?;
            }
            UNDER_BOX
        }
        _ => {
            d.focus(UNDER_BOX).await?;
            UNDER_ROW
        }
    };
    eventually(d, "the focused row to scroll above the footer", async |d| {
        let (row, foot) = (d.rect(cleared).await?, d.rect(FOOT_Q1).await?);
        Ok(row.y + row.height <= foot.y + 1.0)
    })
    .await
}

e2e::scenario!(
    a_focused_windowed_row_clears_the_footer,
    "/table-groups/footer-window",
    a_focused_row_clears_the_footer,
    android: skip("958: element identity on the WebView")
);

e2e::scenario!(
    a_windowed_footer_holds_and_ends_the_rows,
    "/table-groups/footer-window",
    the_windowed_footer_holds_and_ends_the_rows,
    android: skip("958: element identity on the WebView")
);
