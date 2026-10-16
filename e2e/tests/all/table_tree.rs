//! `Table` tree rows (1156 TR1): a row's toggle opens its children under it, says
//! so and keeps the focus; a filter keeps and opens a match's ancestors. A treegrid
//! (TR2): the rows are stops the arrows move and open, its cells too with `cell_focus`.
//! TR3: a row's box selects its subtree, a row loads its children on demand, a tree windows.

use anyhow::{Result, bail};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, ax};

const SITE: &str = "button[aria-label=\"site, 2 rows\"]";
const LIVE: &str = "table + [role=status]";

async fn opens_under_its_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "tbody tr:nth-child(2) th", "launch", "closed roots").await?;
    d.click(SITE).await?;
    eventually_text(d, "#expanded", "site", "a toggle click").await?;
    eventually_text(d, LIVE, "site expanded", "the opened row").await?;
    eventually_text(d, "tbody tr:nth-child(2) th", "design", "the children").await?;
    eventually_text(d, "tbody tr:nth-child(3) th", "build", "the children").await?;
    eventually_text(d, "tbody tr:nth-child(4) th", "launch", "the next root").await?;
    if d.attr("tbody tr:nth-child(2)", "data-level")
        .await?
        .as_deref()
        != Some("1")
    {
        bail!("a child is not on level 1");
    }
    if d.attr(SITE, "aria-expanded").await?.as_deref() != Some("true") {
        bail!("the open toggle is not expanded");
    }
    // A child's toggle sits one indent further in than its parent's.
    let parent = d.rect(SITE).await?;
    let child = d.rect("button[aria-label=\"build, 2 rows\"]").await?;
    if child.x <= parent.x {
        bail!("the child's toggle {child:?} is not indented past {parent:?}");
    }
    d.click(SITE).await?;
    eventually_text(d, "#expanded", "", "a second click").await?;
    eventually_text(d, LIVE, "site collapsed", "the closed row").await?;
    eventually_text(
        d,
        "tbody tr:nth-child(2) th",
        "launch",
        "the children to go",
    )
    .await
}

e2e::scenario!(
    a_tree_toggle_opens_its_children_under_it,
    "/table-tree",
    opens_under_its_row
);

/// Enter opens and Space closes; the focus stays on the toggle through both.
async fn keyboard_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SITE).await?;
    eventually_focused(d, SITE, "focus").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#expanded", "site", "Enter").await?;
    eventually_focused(d, SITE, "Enter").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#expanded", "", "Space").await?;
    eventually_focused(d, SITE, "Space").await
}

e2e::scenario!(
    the_keyboard_toggles_a_tree_row_and_keeps_the_focus,
    "/table-tree",
    keyboard_toggles
);

/// The 16px chevron is pressed in a 24x24 box (WCAG 2.5.8).
#[test]
fn the_tree_toggle_meets_the_target_size() {
    Suite::new("table_tree", "/table-tree")
        .targets(SITE)
        .no_snapshot()
        .run();
}

/// "front" matches deep down: site and build show above it, opened, while the rest go.
async fn filter_shows_ancestors<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("input[type=search]").await?;
    eventually_focused(d, "input[type=search]", "a click").await?;
    d.type_text("front").await?;
    let mut seen = String::new();
    let shown = eventually(d, "the ancestors and the match", async |d| {
        seen = d.text("tbody").await?;
        Ok(seen.split_whitespace().collect::<String>() == "siteAdabuildDifrontDi")
    })
    .await;
    if let Err(error) = shown {
        bail!("{error}: the body reads {seen:?}");
    }
    // Counted by its kept children: one.
    let site = "button[aria-label=\"site, 1 row\"]";
    if d.attr(site, "aria-expanded").await?.as_deref() != Some("true") {
        bail!("a kept ancestor is not open");
    }
    Ok(())
}

e2e::scenario!(
    a_filter_shows_a_match_under_its_ancestors,
    "/table-tree",
    filter_shows_ancestors
);

/// The body row `row`, from 1.
fn row(row: usize) -> String {
    format!("tbody tr:nth-child({row})")
}

async fn on_row<D: Driver>(d: &mut D, at: usize, after: &str) -> Result<()> {
    eventually_focused(d, &row(at), after).await
}

/// Right opens then steps in, Left closes then steps out, Home and End reach the ends.
async fn row_keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "tbody tr:nth-child(2) th", "launch", "closed roots").await?;
    if d.attr(&row(1), "tabindex").await?.as_deref() != Some("0") {
        bail!("the first row is not the stop");
    }
    d.focus(&row(1)).await?;
    on_row(d, 1, "focus").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_text(d, "#expanded", "site", "ArrowRight on a closed row").await?;
    eventually_text(d, LIVE, "site expanded", "the opened row").await?;
    on_row(d, 1, "ArrowRight on a closed row").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    on_row(d, 2, "ArrowRight on an open row").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    on_row(d, 3, "ArrowDown").await?;
    eventually_text(d, "tbody tr:nth-child(3) th", "build", "the second child").await?;
    for (name, value) in [
        ("aria-level", "2"),
        ("aria-posinset", "2"),
        ("aria-setsize", "2"),
    ] {
        if d.attr(&row(3), name).await?.as_deref() != Some(value) {
            bail!("the second child's {name} is not {value}");
        }
    }
    d.press(keyboard::ARROW_LEFT).await?;
    on_row(d, 1, "ArrowLeft on a closed child").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    eventually_text(d, "#expanded", "", "ArrowLeft on an open row").await?;
    on_row(d, 1, "ArrowLeft on an open row").await?;
    d.press(keyboard::END).await?;
    on_row(d, 2, "End").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    on_row(d, 2, "ArrowDown at the end").await?;
    d.press(keyboard::HOME).await?;
    on_row(d, 1, "Home").await?;
    // Closed from outside, a hidden focused row hands the focus to its shown ancestor.
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    on_row(d, 2, "ArrowRight into the children").await?;
    d.type_text("c").await?;
    eventually_text(d, "#expanded", "", "the outside close").await?;
    on_row(d, 1, "the outside close").await?;
    // The toggles are no Tab stops: Tab leaves the table.
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#after", "Tab").await
}

e2e::scenario!(
    the_arrows_move_and_open_the_tree_rows,
    "/table-tree",
    row_keys,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// With `cell_focus`: Left from a first cell takes its row, Right from an open row its first cell.
async fn rows_and_cells<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("[data-cell-stop]").await?;
    eventually_focused(d, "thead tr > :nth-child(1)", "the header stop").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_focused(d, &cell(1, 1), "ArrowDown from the header").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    on_row(d, 1, "ArrowLeft from the first cell").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_text(d, "#expanded", "site", "ArrowRight on a closed row").await?;
    on_row(d, 1, "ArrowRight on a closed row").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, &cell(1, 1), "ArrowRight on an open row").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, &cell(1, 2), "ArrowRight in the row").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_focused(d, &cell(2, 2), "ArrowDown").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    eventually_focused(d, &cell(2, 1), "the first ArrowLeft").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    on_row(d, 2, "ArrowLeft twice").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    on_row(d, 3, "ArrowDown between rows").await
}

/// The body cell in row `row`, column `col`, both from 1.
fn cell(row: usize, col: usize) -> String {
    format!("tbody tr:nth-child({row}) > :nth-child({col})")
}

e2e::scenario!(
    a_tree_with_cell_focus_moves_between_rows_and_cells,
    "/table-tree-cells",
    rows_and_cells,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// Body row `n`'s box, from 1.
fn select_box(n: usize) -> String {
    format!("tbody tr:nth-child({n}) td[data-select] span[aria-hidden]")
}

/// Whether body row `n`'s box is mixed: a browser reads the property alone.
async fn mixed<D: Driver>(d: &mut D, n: usize) -> Result<bool> {
    let input = format!("tbody tr:nth-child({n}) td[data-select] input");
    match d.platform() {
        Platform::Native => Ok(d.attr(&input, "aria-checked").await?.as_deref() == Some("mixed")),
        _ => Ok(d
            .evaluate(&format!("document.querySelector('{input}').indeterminate"))
            .await?
            == true),
    }
}

/// A row's box takes its descendants along and fills or empties its ancestors' boxes.
async fn tree_selection<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // site open: site, design, build, launch.
    eventually_text(d, &format!("{} th", row(3)), "build", "site open").await?;
    d.click(&select_box(3)).await?;
    eventually_text(d, "#selection", "back,build,front", "build's box").await?;
    eventually(d, "site mixed with build's rows", async |d| {
        mixed(d, 1).await
    })
    .await?;
    d.click(&select_box(2)).await?;
    eventually_text(
        d,
        "#selection",
        "back,build,design,front,site,wire",
        "design's box filling site",
    )
    .await?;
    eventually(d, "site no longer mixed", async |d| Ok(!mixed(d, 1).await?)).await?;
    d.click(&select_box(3)).await?;
    eventually_text(d, "#selection", "design,wire", "build's box off").await?;
    eventually(d, "site mixed again", async |d| mixed(d, 1).await).await
}

e2e::scenario!(
    a_tree_box_selects_its_descendants_and_mixes_its_ancestors,
    "/table-tree-select",
    tree_selection,
    ios: skip("3416: flaky: a tap on a tree box sometimes never reaches the page, three times over")
);

/// A row with `row_has_children` opens before its children load, then shows them.
async fn lazy_children<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // Named alone until its children load: "0 rows" would be false (3304).
    let launch = "button[aria-label=\"launch\"]";
    d.click(launch).await?;
    eventually_text(d, "#expanded", "site,launch", "the ask to open").await?;
    eventually(d, "the open row to be busy", async |d| {
        Ok(d.attr(&row(4), "aria-busy").await?.as_deref() == Some("true"))
    })
    .await?;
    d.click("#load").await?;
    eventually_text(d, &format!("{} th", row(5)), "ship", "the loaded children").await?;
    eventually_text(d, &format!("{} th", row(6)), "tell", "the loaded children").await?;
    if d.attr(&row(4), "aria-busy").await?.is_some() {
        bail!("the row stays busy with its children shown");
    }
    if d.attr("button[aria-label=\"launch, 2 rows\"]", "aria-expanded")
        .await?
        .as_deref()
        != Some("true")
    {
        bail!("the loaded row's toggle is not counted and open");
    }
    Ok(())
}

e2e::scenario!(
    a_row_opens_before_its_children_load,
    "/table-tree-select",
    lazy_children
);

/// The focused row reads `text` and shows in the scroll area.
async fn shown_focused<D: Driver>(d: &mut D, text: &str, after: &str) -> Result<()> {
    eventually(
        d,
        &format!("{text} focused and shown after {after}"),
        async |d| {
            if d.text("tbody tr:focus th").await?.trim() != text {
                return Ok(false);
            }
            let (row, area) = (
                d.rect("tbody tr:focus").await?,
                d.rect("[data-table-scroll]").await?,
            );
            Ok(row.y >= area.y - 1.0 && row.y + row.height <= area.y + area.height + 1.0)
        },
    )
    .await
}

/// The arrows reach rows the window has not rendered, and open rows keep their place.
async fn windowed_keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let focused = "tbody tr:focus th";
    let state = d.attr("table", "data-state").await?.unwrap_or_default();
    if !state.split_whitespace().any(|state| state == "windowed") {
        bail!("the tree is not windowed: {state:?}");
    }
    d.focus(&row(1)).await?;
    on_row(d, 1, "focus").await?;
    d.press(keyboard::END).await?;
    shown_focused(d, "r39 c3", "End past the window").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    shown_focused(d, "r39", "ArrowLeft to the parent").await?;
    if d.attr("tbody tr:focus", "aria-posinset").await?.as_deref() != Some("40") {
        bail!("the last root is not 40th of its set");
    }
    // A collapse far above the window keeps the stop, and its row in view.
    d.type_text("x").await?;
    // The header row and 196 body rows.
    eventually(d, "r0 to close", async |d| {
        Ok(d.attr("table", "aria-rowcount").await?.as_deref() == Some("197"))
    })
    .await?;
    shown_focused(d, "r39", "the collapse above").await?;
    d.press(keyboard::HOME).await?;
    eventually_text(d, focused, "r0", "Home").await
}

e2e::scenario!(
    the_arrows_move_through_a_windowed_tree,
    "/table-tree-window",
    windowed_keys,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

#[test]
fn the_treegrid_meets_the_baseline() {
    Suite::new("table_tree", "/table-tree")
        .focusable("tbody tr[tabindex=\"0\"]")
        .run();
}

/// Chromium reads the tree as a treegrid, each row with its level and state.
#[test]
fn the_ax_tree_names_a_treegrid() {
    block_on(async {
        let fixture = Fixture::open("/table-tree", Viewport::Desktop)
            .await
            .unwrap();
        let tree = ax::snapshot(&fixture.page, "table").await.unwrap();
        assert!(tree.starts_with("treegrid \"Tasks\""), "{tree}");
        assert!(tree.contains("[expanded=false] [level=1]"), "{tree}");
        assert!(tree.contains("row \"launch Ada\" [level=1]"), "{tree}");
        fixture
            .console
            .assert_clean("reading the treegrid")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
