//! `Kanban`.

use dioxus::prelude::*;
use libero::components::{Button, Flex, Kanban, KanbanCard, KanbanColumn, KanbanMove, Text};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/kanban", || rsx! { KanbanPage { data: false, wrapped: false } }),
    ("/kanban/data", || rsx! { KanbanPage { data: true, wrapped: false } }),
    ("/kanban/columns", || rsx! { KanbanPage { data: true, wrapped: true } }),
];

const COLUMNS: [&str; 3] = ["To do", "Doing", "Done"];

type Cards = Signal<Vec<Vec<&'static str>>>;

#[derive(Clone, Copy)]
struct Data {
    cards: Cards,
    hide: Signal<bool>,
}

/// Three columns, the last empty; `#order` lists them split by `|`, `#moves` each move.
/// `#refuse` makes the app ignore every move; `#add` puts Omega first in Done. With `data`,
/// `#remove` drops Beta from the data; `#hide` stops showing Alpha, the rest keep their index.
/// With `wrapped`, the data is read by the columns and `#order`, not the page: a data change
/// renders those only, not the board.
#[component]
fn KanbanPage(data: bool, wrapped: bool) -> Element {
    let mut cards: Cards = use_signal(|| vec![vec!["Alpha", "Beta", "Gamma"], vec!["Delta"], vec![]]);
    let mut moves = use_signal(String::new);
    let mut refuse = use_signal(|| false);
    let mut hide = use_signal(|| false);
    use_context_provider(|| Data { cards, hide });
    if !wrapped {
        let _ = cards();
    }
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
                    Lane { key: "{label}", column, label }
                }
            }
            Order {}
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

#[component]
fn Order() -> Element {
    let order = use_context::<Data>()
        .cards
        .read()
        .iter()
        .map(|column| column.join(" "))
        .collect::<Vec<_>>()
        .join(" | ");
    rsx! { Text { id: "order", "{order}" } }
}

#[component]
fn Lane(column: usize, label: &'static str) -> Element {
    let Data { cards, hide } = use_context::<Data>();
    let shown = cards()[column].clone();
    rsx! {
        KanbanColumn { index: column, label, id: "column-{column}",
            for (index, name) in shown.into_iter().enumerate().filter(|(_, name)| !hide() || *name != "Alpha") {
                KanbanCard { key: "{name}", index, id: "{name}", label: name, Text { "{name}" } }
            }
        }
    }
}
