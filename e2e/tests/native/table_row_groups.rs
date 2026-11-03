//! `Table` row grouping on Blitz: a group toggle in a `td` opens its rows, and
//! a thousand grouped rows render, sum and open; Enter on a column menu's
//! "Group by" entry groups the rows (G2). 10k grouped rows window (G3).

use dioxus::prelude::*;
use e2e::native::{Key, Page, mount};
use libero::components::{Aggregate, Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
    team: &'static str,
}

const TEAMS: [&str; 4] = ["Sales", "Ops", "Legal", "Labs"];

fn app() -> Element {
    let people: Vec<Person> = (0..1000)
        .map(|at| Person {
            name: format!("P{at}"),
            team: TEAMS[at % TEAMS.len()],
        })
        .collect();
    rsx! {
        div { width: "400px",
            Table {
                aria_label: "People",
                data: people,
                columns: vec![
                    column("Name").value(|p: &Person| p.name.clone()).row_header(),
                    column("Team").value(|p: &Person| p.team.to_string()),
                    column("One").value(|_: &Person| 1u32).aggregate(Aggregate::Sum),
                ],
                row_key: |p: &Person| p.name.clone(),
                default_row_grouping: vec!["Team".to_string()],
            }
        }
    }
}

const SALES: &str = "button[aria-label=\"Sales, 250 rows\"]";

#[test]
fn a_group_toggle_opens_its_rows_under_its_row() {
    let mut page = mount(app);
    assert!(!page.exists("tbody th"), "{}", page.tree());
    // Each closed group sums its 250 rows.
    assert_eq!(
        page.text("[data-group-aggregate]"),
        "Sum250",
        "{}",
        page.tree()
    );
    page.click(SALES);
    assert_eq!(page.attr(SALES, "aria-expanded").as_deref(), Some("true"));
    let (_, group_y, _, group_height) = page.rect("tr[data-row-group] td");
    let (_, row_y, _, _) = page.rect("tbody th");
    assert!(
        (row_y - (group_y + group_height)).abs() <= 1.0,
        "the first row ({row_y}) is not under its group row ({group_y} + {group_height})"
    );
    page.click(SALES);
    assert!(!page.exists("tbody th"), "{}", page.tree());
}

fn windowed() -> Element {
    let people: Vec<Person> = (0..10_000)
        .map(|at| Person {
            name: format!("P{at}"),
            team: TEAMS[at % TEAMS.len()],
        })
        .collect();
    rsx! {
        div { width: "400px",
            Table {
                aria_label: "People",
                max_height: "300px",
                virtual_row_height: 32.0,
                data: people,
                columns: vec![
                    column("Name").value(|p: &Person| p.name.clone()).row_header(),
                    column("Team").value(|p: &Person| p.team.to_string()),
                ],
                row_key: |p: &Person| p.name.clone(),
                default_row_grouping: vec!["Team".to_string()],
                default_group_expansion: 1,
            }
        }
    }
}

fn row_indices(page: &Page) -> Vec<usize> {
    page.query_all("tbody tr")
        .into_iter()
        .filter_map(|id| page.attr_of(id, "aria-rowindex")?.parse().ok())
        .collect()
}

/// G3: 10k grouped rows render a window on Blitz; a group row takes the pitch, and
/// a wheel moves the window on while the focused group row stays rendered.
#[test]
fn a_windowed_grouped_table_moves_on_a_wheel() {
    let mut page = mount(windowed);
    let windowed = page.wait_for(|page| {
        row_indices(page)
            .iter()
            .max()
            .is_some_and(|&last| last < 40)
    });
    assert!(windowed, "rendered {:?}", row_indices(&page));
    let group = "tr[aria-rowindex=\"2\"][data-row-group]";
    let (_, _, _, height) = page.rect(&format!("{group} td:last-child"));
    assert!((height - 32.0).abs() <= 1.0, "a group row is {height}px");
    // Shut and open again: a click focuses the toggle, where `focus()` fires no `focusin` here.
    let sales = "button[aria-label=\"Sales, 2500 rows\"]";
    page.click(sales);
    page.click(sales);
    assert_eq!(page.attr(sales, "aria-expanded").as_deref(), Some("true"));
    page.hover("tbody td");
    page.wheel("tbody td", 20_000.0);
    let moved = page.wait_for(|page| {
        let rows = row_indices(page);
        rows.contains(&2) && rows.iter().filter(|&&at| at > 400).count() > 5
    });
    assert!(moved, "the window stayed at {:?}", row_indices(&page));
}

fn selectable() -> Element {
    let people: Vec<Person> = (0..8)
        .map(|at| Person {
            name: format!("P{at}"),
            team: TEAMS[at % TEAMS.len()],
        })
        .collect();
    rsx! {
        Table {
            aria_label: "People",
            selectable: true,
            data: people,
            columns: vec![
                column("Name").value(|p: &Person| p.name.clone()).row_header(),
                column("Team").value(|p: &Person| p.team.to_string()),
            ],
            row_key: |p: &Person| p.name.clone(),
            default_row_grouping: vec!["Team".to_string()],
            default_group_expansion: 1,
        }
    }
}

/// G3: a click on a group's box selects its rows, then mixes once one is cleared.
#[test]
fn a_group_box_selects_its_rows() {
    let mut page = mount(selectable);
    const SALES_BOX: &str = "input[aria-label=\"Select group Sales, 2 rows\"]";
    // Sales, P0, P4, Ops, ...: the first row is Sales' group row.
    page.click("tbody tr:first-child td[data-select] span[aria-hidden]");
    assert_eq!(
        page.query_all("tr[aria-selected=\"true\"]").len(),
        2,
        "{}",
        page.tree()
    );
    page.click("tbody tr:nth-child(2) td[data-select] span[aria-hidden]");
    assert_eq!(
        page.query_all("tr[aria-selected=\"true\"]").len(),
        1,
        "{}",
        page.tree()
    );
    assert_eq!(
        page.attr(SALES_BOX, "aria-checked").as_deref(),
        Some("mixed"),
        "{}",
        page.tree()
    );
}

fn menu_app() -> Element {
    rsx! {
        Table {
            aria_label: "People",
            column_menu: true,
            data: vec![
                Person { name: "Ada".to_string(), team: "Sales" },
                Person { name: "Bo".to_string(), team: "Ops" },
            ],
            columns: vec![
                column("Name").value(|p: &Person| p.name.clone()).row_header(),
                column("Team").value(|p: &Person| p.team.to_string()),
            ],
        }
    }
}

#[test]
fn enter_on_group_by_groups_the_rows() {
    let mut page = mount(menu_app);
    page.click("button[aria-label=\"Team column options\"]");
    let entry = (0..12)
        .map(|index| format!("[role=menuitem][data-menu-index=\"{index}\"]"))
        .find(|item| page.exists(item) && page.text(item).trim() == "Group by Team")
        .unwrap_or_else(|| panic!("no Group by Team entry:\n{}", page.tree()));
    page.focus(&entry);
    page.press(Key::Named(NamedKey::Enter));
    assert!(
        page.exists("button[aria-label=\"Sales, 1 row\"]"),
        "{}",
        page.tree()
    );
    assert!(!page.exists("tbody th"), "{}", page.tree());
}
