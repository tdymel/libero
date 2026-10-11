//! `Table` select column (S1): a header filter picks one option; the menu's Filter set
//! to Is any of lists the options as checkboxes.

use anyhow::Result;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;

use crate::table::{PICKER, pick_later};

async fn a_select_filter_narrows_the_rows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const HEADER: &str = "[data-filter-cell] :is(select, [role=combobox])";
    const POPOVER: &str = "[data-filter-popover]";
    const BOXES: &str = "[data-filter-popover] [role=group] input[type=checkbox]";
    const COMPACT: &str = "[data-filter-cell] [role=combobox]";
    const SUMMARY: &str = "[data-filter-cell] [data-filter-summary]";
    const LISTBOX: &str = "[role=listbox]";
    d.focus(HEADER).await?;
    // Any, then Admin.
    pick_later(d, 1).await?;
    eventually_text(d, "#filters", "Is admin []", "an option in the header").await?;
    eventually_text(d, "tbody tr:first-child th", "Ada", "Admin").await?;
    eventually(d, "the two admins", async |d| {
        Ok(d.exists("tbody tr:nth-child(2)").await? && !d.exists("tbody tr:nth-child(3)").await?)
    })
    .await?;

    d.focus("[aria-label=\"Role column options\"]").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, "[role=menuitem]", "the opened column menu").await?;
    d.press(keyboard::HOME).await?;
    eventually_text(d, "[role=menuitem]:focus", "Filter", "Home").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, &format!("{POPOVER} {PICKER}"), "the opened filter").await?;
    // Is, Is not, then Is any of.
    pick_later(d, 2).await?;
    eventually_text(d, "#filters", "IsAnyOf admin []", "Is any of").await?;
    eventually(d, "the option checkboxes", async |d| d.exists(BOXES).await).await?;
    d.focus(BOXES).await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#filters", "IsAnyOf admin [admin]", "the first box").await?;
    d.press(keyboard::TAB).await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(
        d,
        "#filters",
        "IsAnyOf admin [admin,editor]",
        "the second box",
    )
    .await?;
    eventually(d, "four of the five staff", async |d| {
        Ok(d.exists("tbody tr:nth-child(4)").await? && !d.exists("tbody tr:nth-child(5)").await?)
    })
    .await?;

    // The header's row now holds the picks on one line, not a checkbox column.
    eventually_text(d, SUMMARY, "Admin +1", "the header's summary").await?;
    d.focus(COMPACT).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually(d, "the dropdown", async |d| d.exists(LISTBOX).await).await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "the dropdown closed", async |d| {
        Ok(!d.exists(LISTBOX).await?)
    })
    .await?;
    eventually_focused(d, COMPACT, "the header's field after Escape").await?;
    eventually_text(
        d,
        "#filters",
        "IsAnyOf admin [admin,editor]",
        "Escape keeps the filter",
    )
    .await
}

e2e::scenario!(
    a_select_filter_narrows_a_table,
    "/table-filter/select",
    a_select_filter_narrows_the_rows,
    android: skip("958: element identity on the WebView"),
    macos: skip("2862: WKWebView: ArrowDown does not open a select filter's list"),
    ios: skip("3416: the simulator's tap or drag did not drive the page")
);
