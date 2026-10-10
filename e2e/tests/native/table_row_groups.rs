//! `Table` row grouping on Blitz: a group toggle in a `td` opens its rows, and
//! a thousand grouped rows render and open.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Table, column};

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
