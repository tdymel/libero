use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, Orientation, Sortable, SortableItem, Text},
    hooks::SortableMove,
};

/// The whole list as `Fruit` renders it, with the chosen orientation.
fn code(values: &DemoValues, _: &str) -> String {
    let orientation = match values.str("orientation").as_str() {
        "horizontal" => "\n        orientation: \"horizontal\",",
        _ => "",
    };
    format!(
        r#"let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum", "Cherry"]);

rsx! {{
    Sortable {{{orientation}
        onreorder: move |step: SortableMove| step.apply(&mut fruit.write()),
        for (index, name) in fruit().into_iter().enumerate() {{
            SortableItem {{ key: "{{name}}", index, "{{name}}" }}
        }}
    }}
}}"#
    )
}

#[component]
fn Fruit(orientation: Orientation) -> Element {
    let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum", "Cherry"]);
    rsx! {
        Sortable {
            orientation,
            onreorder: move |step: SortableMove| step.apply(&mut fruit.write()),
            for (index, name) in fruit().into_iter().enumerate() {
                SortableItem { key: "{name}", index, "{name}" }
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
                        .doc("Fires on drop when an item changed place. Apply the move to your data with `SortableMove::apply`; until then the list keeps its old order."),
                    prop("children", "Element").doc("The `SortableItem`s."),
                ]),
                props("SortableItem", vec![
                    prop("index", "usize")
                        .doc("The item's current position, from 0. Key the item by its data, not by this."),
                    prop("children", "Element").doc("The item's content, after the handle."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "Each item's handle is a button named by `SortableLabels::handle` (\"Reorder\"), at least 24px square (WCAG 2.5.8).",
                    "A drag starts after the pointer moved 4px (8px for a touch), so a click on the handle stays a click.",
                    "Only the handle takes a touch. Swiping the rest of an item scrolls the page.",
                    "The handle keeps the focus after a drop.",
                    "Under reduced motion the other items jump to their new place instead of sliding.",
                ])
                .must([
                    "Offer another way to reorder for now, such as move up and down buttons: the keyboard and announcements are not in yet.",
                ]),
            lead: rsx! {
                Text {
                    "A list the user reorders by dragging each item's handle. The other items "
                    "step aside while one drags. On drop "
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
                    " each item's handle, element and "
                    Code { source: "style()" }
                    ". The component is those two hooks plus a grip button."
                }
            },
            Demo {
                component: "Sortable",
                children_text: "",
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("vertical"),
                ],
                render: move |values: DemoValues| rsx! {
                    Fruit { orientation: Orientation::from(values.str("orientation").as_str()) }
                },
                wrap: Wrap(code),
            }
        }
    }
}
