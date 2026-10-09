use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, a11y, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Orientation, SortableItemPart, Text};

mod demo;
use demo::Fruit;

#[component]
pub fn SortablePage() -> Element {
    rsx! {
        DocPage {
            title: "Sortable",
            source: "libero/src/components/data_display/sortable",
            markdown: "/md/sortable.md",
            properties: vec![
                props("Sortable", vec![
                    prop("orientation", "Orientation")
                        .default("vertical")
                        .doc("`vertical` stacks the items, `horizontal` puts them in a row."),
                    prop("onreorder", "EventHandler<SortableMove>")
                        .default("required")
                        .doc("Fires on drop, or on a move button, when an item changed place. Apply the move to your data with `SortableMove::apply`; until then the list keeps its old order."),
                    prop("move_buttons", "bool")
                        .default("true")
                        .doc("Each item's two buttons that move it one slot without dragging (WCAG 2.5.7). Hidden, offer another way to reorder without a drag."),
                    prop("children", "Element").default("required").doc("The `SortableItem`s."),
                ]),
                props("SortableItem", vec![
                    prop("index", "usize")
                        .default("required")
                        .doc("The item's current position. The items run exactly 0 to n - 1: a gap or a repeat stops every drag and lift, so number a filtered view or a page from 0. Key the item by its data, not by this."),
                    prop("label", "Option<String>")
                        .default("\"Item {n}\"")
                        .doc("Names the item in its controls and the announcements. Unset, the handle reads the item's content (\"Reorder Apple\"), and the move buttons and announcements `SortableLabels::item` with its position when it was lifted."),
                    prop("parts", "Parts<SortableItemPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").default("required").doc("The item's content, between the handle and the move buttons."),
                ])
                .parts("SortableItemPart", vec![
                    (SortableItemPart::Handle, "The drag handle."),
                    (SortableItemPart::Content, "The wrapper round the item's content."),
                    (SortableItemPart::MoveEarlier, "The move up or back button."),
                    (SortableItemPart::MoveLater, "The move down or forward button."),
                ]),
            ],
            accessibility: a11y()
                .key(["Space", "Enter"], "On a handle, lifts its item. Lifted, drops it where it is.")
                .key(["Up", "Down"], "Moves a lifted item one slot, in a vertical list.")
                .key(["Left", "Right"], "Moves a lifted item one slot, in a horizontal list; mirrored right to left.")
                .key(["Home", "End"], "Moves a lifted item to the first or last slot.")
                .key(["Escape"], "Puts a lifted or dragged item back where it was.")
                .handles([
                    "Each item's handle is a button of at least 24px (WCAG 2.5.8) named with the item, \"Reorder Apple\", and described by how to move it with the keys. On a touch screen with the move buttons shown, the description points to them instead.",
                    "Each item has Move up and Move down buttons (back and forward in a row), named with the item, \"Move Apple up\", so one pointer reorders without a drag (WCAG 2.5.7). At the list's ends the button that cannot move is disabled, and focus goes to the other one.",
                    "A status region says each lift, move, drop and cancel with the item and its place. A key that would move an item past either end says it stays. The words come from `SortableLabels` in the active `Localization`.",
                    "A horizontal list wider than its container scrolls inside itself and never widens the page (WCAG 1.4.10).",
                    "A drag starts after 4px of pointer movement (8px on touch), so a click on the handle stays a click. Only the handle takes a touch: a swipe on the rest of an item scrolls the page.",
                    "Focus leaving a lifted item's handle cancels the move. After a drop the handle keeps focus.",
                    "A lifted item scrolls into view at each key, in a long list and in a row that scrolls inside itself. Holding Space or Enter lifts or drops once.",
                    "A dropped item slides into its slot. Under reduced motion it lands at once, and the others jump instead of sliding.",
                ])
                .must([
                    "Give each `SortableItem` a `label`, or its controls and the announcements name it by position (\"Item 2\").",
                    "With `move_buttons: false`, give the reader another way to reorder without dragging, such as a menu.",
                ])
                .example("A fruit list where each `SortableItem` has `label: name`. Tab reaches \"Reorder Apple\"; Space lifts it, Down moves it, and the status region says where Apple is now; Space drops it, and focus stays on the handle.")
                .limits([
                    "A pointer drag does not scroll the page or a container at its edge. Keep a list within the view, and use the keys or the move buttons for a long one.",
                ]),
            lead: rsx! {
                Text {
                    "A list the user reorders by dragging each item's handle, by keyboard, or "
                    "with each item's move buttons. The other items step aside while one drags. On drop "
                    Code { source: "onreorder" }
                    " gets a "
                    Code { source: "SortableMove" }
                    " with the old and new index, and "
                    Code { source: "apply" }
                    " does it to a "
                    Code { source: "Vec" }
                    "."
                }
            },
            Demo {
                component: "Fruit",
                children_text: "",
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("vertical"),
                    // No `move_buttons` switch: off, the page would need another way to
                    // reorder without a drag (WCAG 2.5.7).
                ],
                render: move |values: DemoValues| rsx! {
                    Fruit {
                        orientation: Orientation::from(values.str("orientation").as_str()),
                    }
                },
                file: DemoFile(include_str!("sortable/demo.rs")),
            }

            DocSection {
                title: "Your own markup",
                Text {
                    "For your own markup, "
                    Code { source: "use_sortable" }
                    " gives the list's handlers and "
                    Code { source: "use_sortable_item(index)" }
                    " each item's handle, element, move buttons and "
                    Code { source: "style()" }
                    ". The component is those two hooks plus a grip button, two move buttons and a status region."
                }
            }
        }
    }
}
