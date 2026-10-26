use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Orientation, Text};

mod demo;
use demo::Fruit;

#[component]
pub fn UseSortablePage() -> Element {
    rsx! {
        DocPage {
            title: "Sortable list",
            source: "libero/src/components/data_display/sortable/use_sortable.rs",
            markdown: "/md/use_sortable.md",
            accessibility: a11y()
                .key(["Space", "Enter"], "On a handle, lifts its item. Lifted, drops it where it is.")
                .key(["Up", "Down"], "Moves a lifted item one slot, in a vertical list.")
                .key(["Left", "Right"], "Moves a lifted item one slot, in a horizontal list; mirrored right to left.")
                .key(["Home", "End"], "Moves a lifted item to the first or last slot.")
                .key(["Escape"], "Puts a lifted or dragged item back where it was.")
                .handles([
                    "A drag starts after 4px of pointer movement (8px on touch), so a click on the handle stays a click.",
                    "Focus leaving a lifted item's handle cancels the move. After a drop the handle keeps focus.",
                    "A lifted item scrolls into view at each key. Holding Space or Enter lifts or drops once.",
                    "`announcement` says each lift, move, drop and cancel with the item and its place, in the active `Localization`. A key that would move an item past either end says it stays.",
                    "`instructions` and `touch_instructions` are the localized key help for the grab handle's description.",
                ])
                .must([
                    "Make the grab handle a focusable button of at least 24px (WCAG 2.5.8) with `drag_handle_sx()`, named with the item, as \"Reorder Apple\", and described by `instructions`.",
                    "Render `announcement` in a `role=\"status\"` region.",
                    "Offer a way to reorder without dragging (WCAG 2.5.7): the move buttons, disabled by `first` and `last` at the list's ends, or a menu.",
                    "Name each item with `use_labelled_sortable_item`, or the announcements say \"Item 2\" while your handle reads \"Reorder Apple\".",
                    "Number the items exactly 0 to n - 1 and key each by its data, not its index, so a reorder moves the node and the handle keeps its focus.",
                ])
                .example("A fruit list whose items call `use_labelled_sortable_item(index, Some(name))`. Tab reaches \"Reorder Apple\"; Space lifts it, Down moves it, and the status region says where Apple is now; Space drops it, and focus stays on the handle.")
                .limits([
                    "A pointer drag does not scroll the page or a container at its edge. Keep a list within the view, and use the keys or the move buttons for a long one.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_sortable(options: SortableOptions) -> SortableHandle" }
                    " makes your own markup a list the user reorders by dragging, as "
                    Code { source: "Sortable" }
                    " does. Spread its element and pointer handlers on the list, and call "
                    Code { source: "use_sortable_item(index)" }
                    " in each item's component for its grab handle, element, move buttons and "
                    Code { source: "style()" }
                    ". On drop "
                    Code { source: "onreorder" }
                    " gets a "
                    Code { source: "SortableMove" }
                    ", and "
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
                ],
                render: move |values: DemoValues| rsx! {
                    Fruit {
                        orientation: Orientation::from(values.str("orientation").as_str()),
                    }
                },
                file: DemoFile(include_str!("use_sortable/demo.rs")),
            }
        }
    }
}
