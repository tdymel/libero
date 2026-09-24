use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Orientation, Sortable, SortableItem, SortableItemPart, Text},
    hooks::SortableMove,
};

/// The whole list as `Fruit` renders it, with the chosen orientation.
fn code(values: &DemoValues, _: &str) -> String {
    let orientation = match values.str("orientation").as_str() {
        "horizontal" => "\n        orientation: \"horizontal\",",
        _ => "",
    };
    let buttons = match values.str("move_buttons").as_str() {
        "false" => "\n        move_buttons: false,",
        _ => "",
    };
    format!(
        r#"let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum", "Cherry"]);

rsx! {{
    Sortable {{{orientation}{buttons}
        onreorder: move |step: SortableMove| step.apply(&mut fruit.write()),
        for (index, name) in fruit().into_iter().enumerate() {{
            SortableItem {{ key: "{{name}}", index, label: name, "{{name}}" }}
        }}
    }}
}}"#
    )
}

#[component]
fn Fruit(orientation: Orientation, move_buttons: bool) -> Element {
    let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum", "Cherry"]);
    rsx! {
        Sortable {
            orientation,
            move_buttons,
            onreorder: move |step: SortableMove| step.apply(&mut fruit.write()),
            for (index, name) in fruit().into_iter().enumerate() {
                SortableItem { key: "{name}", index, label: name, "{name}" }
            }
        }
    }
}

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
                        .doc("Fires on drop, or on a move button, when an item changed place. Apply the move to your data with `SortableMove::apply`; until then the list keeps its old order."),
                    prop("move_buttons", "bool")
                        .default("true")
                        .doc("Each item's two buttons that move it one slot without dragging (WCAG 2.5.7). Hidden, offer another way to reorder without a drag."),
                    prop("children", "Element").doc("The `SortableItem`s."),
                ]),
                props("SortableItem", vec![
                    prop("index", "usize")
                        .doc("The item's current position, from 0. Key the item by its data, not by this."),
                    prop("label", "Option<String>")
                        .default("\"Item {n}\"")
                        .doc("Names the item in the announcements. Unset, `SortableLabels::item` with its position when it was lifted."),
                    prop("parts", "Parts<SortableItemPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                    prop("children", "Element").doc("The item's content, between the handle and the move buttons."),
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
                    "Each item's handle is a button named by `SortableLabels::handle` (\"Reorder\"), at least 24px square (WCAG 2.5.8). It is described by `SortableLabels::instructions`, how to move by keyboard.",
                    "A `role=\"status\"` region announces each lift, move, drop and cancel with the item's `label` and its position, from the `SortableLabels` templates of the active `Localization`.",
                    "Each item has a move up and a move down button (back and forward in a row) for a single pointer (WCAG 2.5.7). The first item's move up and the last one's move down are disabled; the focus stays on the pressed button, or goes to the other one at the list's end.",
                    "A drag starts after the pointer moved 4px (8px for a touch), so a click on the handle stays a click.",
                    "Only the handle takes a touch. Swiping the rest of an item scrolls the page.",
                    "Moving focus off a lifted item's handle cancels the move.",
                    "The handle keeps the focus after a drop.",
                    "A dropped item slides from where it was let go into its slot, over the theme's transition duration. Under reduced motion it lands at once, and the other items jump to their new place instead of sliding.",
                ])
                .must([
                    "Give each `SortableItem` a `label`, or the announcements name it by position (\"Item 2\").",
                    "With `move_buttons: false`, give the reader another way to reorder without dragging, such as a menu.",
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
                Text {
                    "For your own markup, "
                    Code { source: "use_sortable" }
                    " gives the list's handlers and "
                    Code { source: "use_sortable_item(index)" }
                    " each item's handle, element, move buttons and "
                    Code { source: "style()" }
                    ". The component is those two hooks plus a grip button, two move buttons and a status region."
                }
            },
            Demo {
                component: "Sortable",
                children_text: "",
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("vertical"),
                    Control::switch("move_buttons").default("true"),
                ],
                render: move |values: DemoValues| rsx! {
                    Fruit {
                        orientation: Orientation::from(values.str("orientation").as_str()),
                        move_buttons: values.str("move_buttons") != "false",
                    }
                },
                wrap: Wrap(code),
            }
        }
    }
}
