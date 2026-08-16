use dioxus::prelude::*;
use libero::components::{Flex, Option as SelectOption, Select, Text, Title};

#[component]
pub fn SelectPage() -> Element {
    let mut value = use_signal(|| "sm".to_string());

    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Select" }
                Text { "A styled native select, wrapped in its own label when label is set." }
            }
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Example" }
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
            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Sizes" }
                Flex {
                    direction: "row",
                    gap: "12px",
                    align: "flex-start",
                    Select { size: "xs", value: "a", SelectOption { value: "a", "Option A" } }
                    Select { size: "md", value: "a", SelectOption { value: "a", "Option A" } }
                    Select { size: "xl", value: "a", SelectOption { value: "a", "Option A" } }
                }
            }
        }
    }
}
