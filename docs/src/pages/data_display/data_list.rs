use dioxus::prelude::*;
use libero::{
    components::{Code, DataList, DataListItem, Flex, Text, Title},
    sx::sx,
};

#[component]
pub fn DataListPage() -> Element {
    let phones = vec!["555-1234", "555-5678"];

    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { size: "xxl", "DataList" }
                Text {
                    "Renders a "
                    Code { "dl" }
                    " of term/description pairs. Unlike "
                    Code { "List" }
                    ", a term can have more than one description - "
                    Code { "DataListItem" }
                    "'s "
                    Code { "children" }
                    " is a "
                    Code { "Vec<Element>" }
                    ", not a single "
                    Code { "Element" }
                    ", so writing more than one child gives each one its own "
                    Code { "dd" }
                    " with no extra ceremony over a single description - including from a "
                    Code { "for" }
                    " loop, which flattens the same way."
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Vertical (default)" }
                DataList {
                    DataListItem { label: rsx! { "Status" }, "Active" }
                    DataListItem { label: rsx! { "Owner" }, "Jamie Chen" }
                    DataListItem {
                        label: rsx! { "Phone" },
                        for phone in &phones {
                            "{phone}"
                        }
                    }
                }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { size: "xl", "Horizontal" }
                Text {
                    "Terms and descriptions sit in two aligned columns. A term with several "
                    "descriptions still lines up correctly - each extra description just adds "
                    "another row under the value column, without repeating the term."
                }
                DataList {
                    orientation: "horizontal",
                    sx: sx().max_width("360px"),
                    DataListItem { label: rsx! { "Status" }, "Active" }
                    DataListItem { label: rsx! { "Owner" }, "Jamie Chen" }
                    DataListItem {
                        label: rsx! { "Phone" },
                        for phone in &phones {
                            "{phone}"
                        }
                    }
                }
            }
        }
    }
}
