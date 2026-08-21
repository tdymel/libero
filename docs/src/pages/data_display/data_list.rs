use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Chip, Code, DataList, DataListItem, Text},
    sx::sx,
};

#[component]
pub fn DataListPage() -> Element {
    let phones = vec!["555-1234", "555-5678"];

    rsx! {
        DocPage {
            title: "DataList",
            lead: rsx! {
                Text {
                    "Renders a "
                    Code { source: "dl" }
                    " of term/description pairs. Unlike "
                    Code { source: "List" }
                    ", a term can have more than one description - "
                    Code { source: "DataListItem" }
                    "'s "
                    Code { source: "children" }
                    " is a "
                    Code { source: "Vec<Element>" }
                    ", not a single "
                    Code { source: "Element" }
                    ", so writing more than one child gives each one its own "
                    Code { source: "dd" }
                    " with no extra ceremony over a single description - including from a "
                    Code { source: "for" }
                    " loop, which flattens the same way."
                    // The split is a fork-only capability, so on main the page
                    // says what this build actually does instead.
                    if !cfg!(feature = "dioxus-fork") {
                        " Needs the "
                        Code { source: "dioxus-fork" }
                        " build - against upstream dioxus main the descriptions collapse into a single "
                        Code { source: "dd" }
                        "."
                    }
                }
            },
            DocSection {
                title: "Vertical (default)",
                DataList {
                    DataListItem {
                        label: rsx! { "Status" },
                        Chip { variant: "filled", color: "success", size: "xs", "Active" }
                    }
                    DataListItem { label: rsx! { "Owner" }, "Jamie Chen" }
                    DataListItem {
                        label: rsx! { "Phone" },
                        for phone in &phones {
                            "{phone}"
                        }
                    }
                }
            }
            DocSection {
                title: "Horizontal",
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
