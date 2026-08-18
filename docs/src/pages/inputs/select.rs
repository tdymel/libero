use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Flex, Option as SelectOption, Select, Text};

#[component]
pub fn SelectPage() -> Element {
    let mut value = use_signal(|| "sm".to_string());

    rsx! {
        DocPage {
            title: "Select",
            lead: rsx! {
                Text { "A styled native select, wrapped in its own label when label is set." }
            },
            DocSection {
                title: "Example",
                Select {
                    label: "Size",
                    value: value(),
                    onchange: move |v| value.set(v),
                    SelectOption { value: "xs", "Extra small" }
                    SelectOption { value: "sm", "Small" }
                    SelectOption { value: "md", "Medium" }
                    SelectOption { value: "lg", "Large" }
                    SelectOption { value: "xl", "Extra large" }
                }
                Text { "Selected: {value()}" }
            }
            DocSection {
                title: "Sizes",
                Flex {
                    direction: "row",
                    gap: "md",
                    align: "flex-start",
                    Select { size: "xs", value: "a", SelectOption { value: "a", "Option A" } }
                    Select { size: "md", value: "a", SelectOption { value: "a", "Option A" } }
                    Select { size: "xl", value: "a", SelectOption { value: "a", "Option A" } }
                }
            }
        }
    }
}
