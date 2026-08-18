use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Center, Text},
    sx::sx,
};

#[component]
pub fn CenterPage() -> Element {
    rsx! {
        DocPage {
            title: "Center",
            lead: rsx! {
                Text { "Centers its child both horizontally and vertically." }
            },
            DocSection {
                title: "Block",
                Center {
                    sx: sx().width("100%").height("120px").background("primary.1"),
                    Box { sx: sx().padding("8px 16px").background("primary"), "Centered" }
                }
            }
            DocSection {
                title: "Inline",
                Text { "`inline` uses `inline-flex` instead of `flex`, so the box doesn't stretch to fill its parent's width." }
                Center {
                    inline: true,
                    sx: sx().background("primary.1"),
                    Box { sx: sx().padding("8px 16px").background("primary"), "Centered" }
                }
            }
        }
    }
}
