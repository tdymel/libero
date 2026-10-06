//! `Kanban`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Kanban, KanbanCard, KanbanColumn, KanbanMove, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/kanban", || rsx! { KanbanPage { data: false } }),
    ("/kanban/data", || rsx! { KanbanPage { data: true } }),
];

const COLUMNS: [&str; 3] = ["To do", "Doing", "Done"];

/// Three columns, the last empty; `#order` lists them split by `|`, `#moves` each move.
/// `#refuse` makes the app ignore every move; `#add` puts Omega first in Done. With `data`,
/// `#remove` drops Beta from the data; `#hide` stops showing Alpha, the rest keep their index.
#[component]
fn KanbanPage(data: bool) -> Element {
    let mut cards = use_signal(|| vec![vec!["Alpha", "Beta", "Gamma"], vec!["Delta"], vec![]]);
    let mut moves = use_signal(String::new);
    let mut refuse = use_signal(|| false);
    let mut hide = use_signal(|| false);
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
                    if !refuse() {
                        step.apply(&mut cards.write());
                    }
                },
                for (column, label) in COLUMNS.into_iter().enumerate() {
                    KanbanColumn { key: "{label}", index: column, label, id: "column-{column}",
                        for (index, name) in cards()[column].clone().into_iter().enumerate().filter(|(_, name)| !hide() || *name != "Alpha") {
                            KanbanCard { key: "{name}", index, id: "{name}", label: name, Text { "{name}" } }
                        }
                    }
                }
            }
            Text { id: "order", "{order}" }
            Text { id: "moves", "{moves}" }
            Button { id: "after", "After" }
            Button { id: "refuse", onclick: move |_| refuse.set(true), "Refuse moves" }
            Button { id: "add", onclick: move |_| cards.write()[2].insert(0, "Omega"), "Add Omega" }
            if data {
                Button { id: "remove", onclick: move |_| cards.write()[0].retain(|name| *name != "Beta"), "Remove Beta" }
                Button { id: "hide", onclick: move |_| hide.set(true), "Hide Alpha" }
            }
        }
    }
}
