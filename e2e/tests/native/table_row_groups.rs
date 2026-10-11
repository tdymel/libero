//! `Table` row grouping on Blitz: a group toggle in a `td` opens its rows, and
//! a thousand grouped rows render, sum and open; Enter on a column menu's
//! "Group by" entry groups the rows (G2).

use dioxus::prelude::*;
use e2e::native::{Key, mount};
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
    page.press(Key::Enter);
    assert!(
        page.exists("button[aria-label=\"Sales, 1 row\"]"),
        "{}",
        page.tree()
    );
    assert!(!page.exists("tbody th"), "{}", page.tree());
}
