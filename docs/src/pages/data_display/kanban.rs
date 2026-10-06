use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, a11y, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, KanbanCardPart, KanbanColumnPart, Text};

mod demo;
use demo::Board;

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
                        .default("required")
                        .doc("Fires when a card changed place: on a drop, a move button or a Move to entry. Apply it to your data with `KanbanMove::apply`; until then the board keeps its old order."),
                    prop("move_buttons", "bool")
                        .default("true")
                        .doc("Each card's two buttons that move it one slot in its column without dragging (WCAG 2.5.7)."),
                    prop("children", "Element").default("required").doc("The `KanbanColumn`s."),
                ]),
                props("KanbanColumn", vec![
                    prop("index", "usize")
                        .default("required")
                        .doc("The column's position on the board, from 0. Key the column by its data, not by this."),
                    prop("label", "String")
                        .default("required")
                        .doc("The column's name: the header's text, the card list's accessible name, its entry in every Move to menu and the announcements."),
                    prop("header", "Option<Element>")
                        .default("None")
                        .doc("The header's content instead of `label`, e.g. with a count. `label` stays the list's name."),
                    prop("parts", "Parts<KanbanColumnPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").default("required").doc("The column's `KanbanCard`s."),
                ])
                .parts("KanbanColumnPart", vec![
                    (KanbanColumnPart::Header, "The header naming the column."),
                    (KanbanColumnPart::List, "The list holding the cards."),
                ]),
                props("KanbanCard", vec![
                    prop("index", "usize")
                        .default("required")
                        .doc("The card's position in its column's data, from 0. Key the card by its data, not by this. A filtered column may skip indices: a drag and Move to land by the cards it shows."),
                    prop("label", "Option<String>")
                        .default("\"Item {n}\"")
                        .doc("Names the card in its controls and the announcements. Unset, the handle reads the card's content, and the other controls and announcements `SortableLabels::item` with its position."),
                    prop("parts", "Parts<KanbanCardPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").default("required").doc("The card's content, between the handle and the move buttons."),
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
                    "Each column is a list named by its `label`, and each card a list item.",
                    "Each card's handle is a button of at least 24px (WCAG 2.5.8) named with the card, \"Reorder Write\", and described by how to move it with the keys. On a touch screen with `move_buttons` on, the description points to the move buttons instead.",
                    "Each card has Move up and Move down buttons and a Move to column menu, named with the card, \"Move Write up\", \"Move Write to column\", so nothing needs a drag (WCAG 2.5.7). The menu lists every column, the card's own disabled.",
                    "A card sent to another column by the menu lands at its end, and focus follows to its Move to button. A dragged card lands where it was let go, and its handle keeps focus.",
                    "One status region for the board says each lift, move, drop and cancel, with the column and the card's place when the column changes. The words come from `SortableLabels` and `KanbanLabels` in the active `Localization`.",
                    "A drag starts after 4px of pointer movement (8px on touch). Only the handle takes a touch: a swipe on the rest of a card scrolls. The column under the card's centre takes it; let go off the board, it goes back.",
                    "On a board wider than its container, a card dragged near a side edge scrolls the board that way.",
                    "In a narrow column the move buttons wrap below the card's content.",
                ])
                .must([
                    "Give each `KanbanCard` a `label`, or its controls and the announcements name it by position (\"Item 2\").",
                ])
                .example("A task board where each `KanbanCard` has `label: title`. On the \"Write\" card, Move to column opens a menu of the columns; picking Done moves the card to the end of Done, the status region says so, and focus lands on the card's Move to button there.")
                .limits([
                    "The keyboard drag moves a card within its column only. Moving to another column is the Move to menu.",
                    "A drag scrolls the board sideways only, not the page: to reach a card or column above or below the window, use the Move to menu.",
                    "Columns keep their place: the board has no column reorder.",
                    "In a filtered column a dragged card lands just before the next card shown, or just after the last one; Move to lands after the last card shown, before any hidden ones that follow.",
                ]),
            lead: rsx! {
                Text {
                    "A board of columns. A card moves by dragging its handle, in its column or to another; "
                    "by keyboard or with its move buttons in its column; and to another column with its Move to menu. "
                    Code { source: "onmove" }
                    " gets a "
                    Code { source: "KanbanMove" }
                    " with the old and new column and index, and "
                    Code { source: "apply" }
                    " does it to a "
                    Code { source: "Vec<Vec<T>>" }
                    "."
                }
            },
            Demo {
                component: "Board",
                children_text: "",
                file: DemoFile(include_str!("kanban/demo.rs")),
                controls: vec![Control::switch("move_buttons").default("true")],
                render: move |values: DemoValues| rsx! {
                    Board { move_buttons: values.str("move_buttons") != "false" }
                },
                wide_preview: true,
            }

            DocSection {
                title: "Cards",
                Text {
                    "The cards are your content: the board draws no card look. The demo builds Jira-style "
                    "issue cards from Text, Badge, Icon and Avatar."
                }
            }
        }
    }
}
