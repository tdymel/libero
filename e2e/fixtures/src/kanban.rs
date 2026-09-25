//! `Kanban`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Kanban, KanbanCard, KanbanColumn, KanbanMove, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/kanban", || rsx! { KanbanPage {} })];

const COLUMNS: [&str; 3] = ["To do", "Doing", "Done"];

/// Three columns, the last empty; `#order` lists them split by `|`, `#moves` each move.
#[component]
fn KanbanPage() -> Element {
    let mut cards = use_signal(|| vec![vec!["Alpha", "Beta", "Gamma"], vec!["Delta"], vec![]]);
    let mut moves = use_signal(String::new);
    let order = cards()
        .iter()
        .map(|column| column.join(" "))
        .collect::<Vec<_>>()
        .join(" | ");
    rsx! {
        Flex { direction: "column", gap: "md",
            Button { id: "before", "Before" }
            Kanban {
                id: "board",
                onmove: move |step: KanbanMove| {
                    moves.write().push_str(&format!(
                        "{}.{}>{}.{} ",
                        step.from_column, step.from, step.to_column, step.to
                    ));
                    step.apply(&mut cards.write());
                },
                for (column, label) in COLUMNS.into_iter().enumerate() {
                    KanbanColumn { key: "{label}", index: column, label, id: "column-{column}",
                        for (index, name) in cards()[column].clone().into_iter().enumerate() {
                            KanbanCard { key: "{name}", index, id: "{name}", label: name, Text { "{name}" } }
                        }
                    }
                }
            }
            Text { id: "order", "{order}" }
            Text { id: "moves", "{moves}" }
            Button { id: "after", "After" }
        }
    }
}
