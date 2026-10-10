//! `Kanban`.

use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Kanban, KanbanCard, KanbanColumn, KanbanMove, Text},
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/kanban", || rsx! { KanbanPage { data: false, wrapped: false } }),
    ("/kanban/data", || rsx! { KanbanPage { data: true, wrapped: false } }),
    ("/kanban/columns", || rsx! { KanbanPage { data: true, wrapped: true } }),
    ("/kanban/capped", || rsx! { CappedPage {} }),
    ("/kanban/menu-only", || rsx! { MenuOnlyPage {} }),
    ("/kanban/rtl", || rsx! { div { dir: "rtl", KanbanPage { data: false, wrapped: false } } }),
];

/// Two columns with `move_buttons` off, so Move to is a touch screen's only way across.
#[component]
fn MenuOnlyPage() -> Element {
    let mut cards: Cards = use_signal(|| vec![vec!["Alpha", "Beta"], vec![]]);
    rsx! {
        Kanban { id: "board", move_buttons: false, onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
            for (column, label) in ["To do", "Done"].into_iter().enumerate() {
                KanbanColumn { key: "{label}", index: column, label,
                    for (index, name) in cards()[column].clone().into_iter().enumerate() {
                        KanbanCard { key: "{name}", index, id: "{name}", label: name, Text { "{name}" } }
                    }
                }
            }
        }
    }
}

/// A 200px board whose one column holds 20 cards, most of them below the board's edge;
/// `#order` lists the cards.
#[component]
fn CappedPage() -> Element {
    let mut cards: Signal<Vec<Vec<usize>>> = use_signal(|| vec![(0..20).collect()]);
    let order = cards.read()[0]
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(" ");
    rsx! {
        Kanban { id: "board", sx: sx().height("200px"), onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
            KanbanColumn { index: 0, label: "Long",
                for (index, card) in cards()[0].clone().into_iter().enumerate() {
                    KanbanCard { key: "{card}", index, id: "card-{card}", label: "Card {card}",
                        Text { "Card {card}" }
                    }
                }
            }
        }
        Text { id: "order", "{order}" }
    }
}

const COLUMNS: [&str; 3] = ["To do", "Doing", "Done"];

type Cards = Signal<Vec<Vec<&'static str>>>;

#[derive(Clone, Copy)]
struct Data {
    cards: Cards,
    hidden: Signal<Vec<&'static str>>,
}

/// Three columns, the last empty; `#order` lists them split by `|`, `#moves` each move.
/// `#refuse` makes the app ignore every move; `#add` puts Omega first in Done. With `data`,
/// `#remove` drops Beta from the data; `#hide` stops showing Alpha, the rest keep their index.
/// With `wrapped`, the data is read by the columns and `#order`, not the page: a data change
/// renders those only, not the board. `/kanban/rtl` is the plain page in a right-to-left box.
#[component]
fn KanbanPage(data: bool, wrapped: bool) -> Element {
    let mut cards: Cards = use_signal(|| vec![vec!["Alpha", "Beta", "Gamma"], vec!["Delta"], vec![]]);
    let mut moves = use_signal(String::new);
    let mut refuse = use_signal(|| false);
    let mut hidden = use_signal(Vec::new);
    use_context_provider(|| Data { cards, hidden });
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
                Button { id: "hide", onclick: move |_| hidden.write().push("Alpha"), "Hide Alpha" }
                Button { id: "hide-gamma", onclick: move |_| hidden.write().push("Gamma"), "Hide Gamma" }
                Button { id: "grow", onclick: move |_| cards.write()[0].push("Zeta"), "Add Zeta" }
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
    let Data { cards, hidden } = use_context::<Data>();
    let shown = cards()[column].clone();
    rsx! {
        KanbanColumn { index: column, label, id: "column-{column}",
            for (index, name) in shown.into_iter().enumerate().filter(|(_, name)| !hidden().contains(name)) {
                KanbanCard { key: "{name}", index, id: "{name}", label: name, Text { "{name}" } }
            }
        }
    }
}
