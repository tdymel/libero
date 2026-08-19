use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Flex, Sidebar, Text},
    sx::sx,
};

#[component]
pub fn SidebarPage() -> Element {
    rsx! {
        DocPage {
            title: "Sidebar",
            lead: rsx! {
                Text {
                    "An in-flow panel bordering one edge of its parent, scrolling its own "
                    "content. side picks the border and the size axis - the panel's actual "
                    "position is your layout's, so place it at the matching end of the DOM."
                }
            },
            DocSection {
                title: "Left",
                Flex {
                    direction: "row",
                    sx: sx().height("120px").border("1px solid").border_color("grey.3"),
                    Sidebar {
                        size: "xs",
                        Text { "Navigation" }
                    }
                    Flex {
                        direction: "column",
                        sx: sx().flex("1").padding("12px"),
                        Text { "Rest of the layout" }
                    }
                }
            }
            DocSection {
                title: "Right",
                Flex {
                    direction: "row",
                    sx: sx().height("120px").border("1px solid").border_color("grey.3"),
                    Flex {
                        direction: "column",
                        sx: sx().flex("1").padding("12px"),
                        Text { "Rest of the layout" }
                    }
                    Sidebar {
                        side: "right",
                        size: "xs",
                        Text { "Inspector" }
                    }
                }
            }
        }
    }
}
