use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{
    Code, Kanban, KanbanCard, KanbanCardPart, KanbanColumn, KanbanColumnPart, KanbanMove, Text,
};

const COLUMNS: [&str; 3] = ["To do", "Doing", "Done"];

/// The whole board as `Board` renders it.
fn code(values: &DemoValues, _: &str) -> String {
    let buttons = match values.str("move_buttons").as_str() {
        "false" => "\n        move_buttons: false,",
        _ => "",
    };
    format!(
        r#"let columns = ["To do", "Doing", "Done"];
let mut cards = use_signal(|| vec![
    vec!["Write the brief", "Draw the board"],
    vec!["Review the API"],
    vec![],
]);

rsx! {{
    Kanban {{{buttons}
        onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
        for (column, label) in columns.into_iter().enumerate() {{
            KanbanColumn {{ key: "{{label}}", index: column, label,
                for (index, card) in cards()[column].clone().into_iter().enumerate() {{
                    KanbanCard {{ key: "{{card}}", index, label: card, "{{card}}" }}
                }}
            }}
        }}
    }}
}}"#
    )
}

#[component]
fn Board(move_buttons: bool) -> Element {
    let mut cards = use_signal(|| {
        vec![
            vec!["Write the brief", "Draw the board"],
            vec!["Review the API"],
            vec![],
        ]
    });
    rsx! {
        Kanban {
            move_buttons,
            onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
            for (column, label) in COLUMNS.into_iter().enumerate() {
                KanbanColumn { key: "{label}", index: column, label,
                    for (index, card) in cards()[column].clone().into_iter().enumerate() {
                        KanbanCard { key: "{card}", index, label: card, "{card}" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn KanbanPage() -> Element {
    rsx! {
        DocPage {
            title: "Kanban",
            source: "libero/src/components/data_display/kanban",
            markdown: "/md/kanban.md",
            properties: vec![
                props("Kanban", vec![
                    prop("onmove", "EventHandler<KanbanMove>")
                        .doc("Fires when a card changed place: on a drop, a move button or a Move to entry. Apply it to your data with `KanbanMove::apply`; until then the board keeps its old order."),
                    prop("move_buttons", "bool")
                        .default("true")
                        .doc("Each card's two buttons that move it one slot in its column without dragging (WCAG 2.5.7)."),
                    prop("children", "Element").doc("The `KanbanColumn`s."),
                ]),
                props("KanbanColumn", vec![
                    prop("index", "usize")
                        .doc("The column's position on the board, from 0. Key the column by its data, not by this."),
                    prop("label", "String")
                        .doc("The column's name: the header's text, the card list's accessible name, its entry in every Move to menu and the announcements."),
                    prop("header", "Option<Element>")
                        .doc("The header's content instead of `label`, e.g. with a count. `label` stays the list's name."),
                    prop("parts", "Parts<KanbanColumnPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").doc("The column's `KanbanCard`s."),
                ])
                .parts("KanbanColumnPart", vec![
                    (KanbanColumnPart::Header, "The header naming the column."),
                    (KanbanColumnPart::List, "The list holding the cards."),
                ]),
                props("KanbanCard", vec![
                    prop("index", "usize")
                        .doc("The card's position in its column, from 0. Key the card by its data, not by this."),
                    prop("label", "Option<String>")
                        .default("\"Item {n}\"")
                        .doc("Names the card in its controls and the announcements. Unset, `SortableLabels::item` with its position."),
                    prop("parts", "Parts<KanbanCardPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").doc("The card's content, between the handle and the move buttons."),
                ])
                .parts("KanbanCardPart", vec![
                    (KanbanCardPart::Handle, "The drag handle."),
                    (KanbanCardPart::Content, "The wrapper round the card's content."),
                    (KanbanCardPart::MoveEarlier, "The move up button."),
                    (KanbanCardPart::MoveLater, "The move down button."),
                    (KanbanCardPart::MoveTo, "The Move to column menu's trigger."),
                ]),
            ],
            accessibility: a11y()
                .key(["Space", "Enter"], "On a handle, lifts its card. Lifted, drops it where it is.")
                .key(["Up", "Down"], "Moves a lifted card one slot in its column.")
                .key(["Home", "End"], "Moves a lifted card to the top or bottom of its column.")
                .key(["Escape"], "Puts a lifted or dragged card back where it was.")
                .handles([
                    "Each column is a list named by its `label`; each card is a list item.",
                    "Each card's handle is a button named by `SortableLabels::handle` with the card's name (\"Reorder Write\"), at least 24px square (WCAG 2.5.8), described by `SortableLabels::instructions`.",
                    "Each card has a move up and a move down button in its column, and a Move to column menu button named by `KanbanLabels::move_to`, each naming the card (\"Move Write up\", \"Move Write to column\"). The menu lists every column by `label`, the card's own disabled. Neither needs a drag (WCAG 2.5.7).",
                    "A card moved to another column lands at its end, and its Move to button takes the focus there.",
                    "One `role=\"status\"` region for the board announces each lift, move, drop and cancel, and a move to another column with the column's name and the card's position, from the `SortableLabels` and `KanbanLabels` templates of the active `Localization`.",
                    "A drag starts after the pointer moved 4px (8px for a touch); only the handle takes a touch, so a swipe on the rest of a card scrolls.",
                    "In a narrow column, where the content would get less than 8rem, the move buttons wrap below it.",
                ])
                .must([
                    "Give each `KanbanCard` a `label`, or its controls and the announcements name it by position (\"Item 2\").",
                ])
                .limits([
                    "A pointer drag moves a card within its column only. Moving to another column is the Move to menu.",
                    "Columns keep their place: the board has no column reorder.",
                ]),
            lead: rsx! {
                Text {
                    "A board of columns. A card reorders in its column by dragging its handle, by keyboard, "
                    "or with its move buttons, and moves to another column with its Move to menu. "
                    Code { source: "onmove" }
                    " gets a "
                    Code { source: "KanbanMove" }
                    " with the old and new column and index, and "
                    Code { source: "apply" }
                    " does it to a "
                    Code { source: "Vec<Vec<T>>" }
                    ". The cards are your content: the board draws no card look."
                }
            },
            Demo {
                component: "Kanban",
                children_text: "",
                controls: vec![Control::switch("move_buttons").default("true")],
                render: move |values: DemoValues| rsx! {
                    Board { move_buttons: values.str("move_buttons") != "false" }
                },
                wrap: Wrap(code),
            }
        }
    }
}
