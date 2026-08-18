use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, NavLink, Text},
    sx::sx,
};

#[component]
pub fn NavLinkPage() -> Element {
    rsx! {
        DocPage {
            title: "NavLink",
            lead: rsx! {
                Text {
                    "A navigation list item - "
                    Code { "Anchor" }
                    " plus a themed active/hover background and "
                    Code { "aria-current" }
                    ", for a sidebar or nav bar link. Colors come from the theme ("
                    Code { "Theme::nav_link" }
                    ") by default."
                }
            },
            DocSection {
                title: "Active state",
                Text {
                    sx: sx().color("grey.6"),
                    "Auto-detected by comparing "
                    Code { "to" }
                    " against the current route when "
                    Code { "active" }
                    " is unset - the first link below is this very page, so it reads as "
                    "active on its own.",
                }
                Flex {
                    direction: "column",
                    gap: "xs",
                    sx: sx().width("240px"),
                    NavLink { to: crate::Route::NavLinkPage {}, "This page (auto-active)" }
                    NavLink { to: crate::Route::GettingStarted {}, "Another page" }
                    NavLink {
                        to: "https://dioxuslabs.com",
                        target: "_blank",
                        active: true,
                        "Forced active (external link)",
                    }
                }
            }

            DocSection {
                title: "Colors",
                Text {
                    sx: sx().color("grey.6"),
                    "Forced active here to show the tint - color only shows once a link is "
                    "active.",
                }
                Flex {
                    direction: "column",
                    gap: "xs",
                    sx: sx().width("240px"),
                    NavLink { to: "#", active: true, "Primary (default)" }
                    NavLink { to: "#", active: true, color: "success", "Success" }
                    NavLink { to: "#", active: true, color: "error", "Error" }
                    NavLink { to: "#", active: true, color: "warning", "Warning" }
                }
            }

            DocSection {
                title: "Disabled",
                Flex {
                    direction: "column",
                    gap: "xs",
                    sx: sx().width("240px"),
                    NavLink { to: "#", disabled: true, "Disabled" }
                }
            }
        }
    }
}
