use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Box, Container, Divider, List, ListItem, Text, Title},
    sx::{Sx, sx},
};

fn main() {
    dioxus::launch(App);
}

fn monospace_sx() -> Sx {
    sx().font_family("ui-monospace, SFMono-Regular, Menlo, Consolas, monospace")
}

#[component]
fn App() -> Element {
    rsx! {
        LiberoProvider {
            Box {
                sx: sx().display("flex").height("100vh"),
                Container {
                    size: "sm",
                    sx: sx().flex("1").padding("48px 64px").overflow("auto"),
                    GettingStarted {}
                }
                Divider { vertical: true }
                Sidebar {}
            }
        }
    }
}

#[component]
fn Sidebar() -> Element {
    rsx! {
        Box {
            component: "aside",
            sx: sx().width("240px").flex_shrink("0").padding("32px 24px"),
            Title { variant: "h6", "Documentation" }
            List {
                sx: sx().margin_top("12px"),
                ListItem {
                    Box {
                        component: "a",
                        href: "#getting-started",
                        sx: sx()
                            .color("primary.6")
                            .font_weight("600")
                            .text_decoration("none"),
                        "Getting Started"
                    }
                }
            }
        }
    }
}

#[component]
fn CodeBlock(children: Element) -> Element {
    rsx! {
        Box {
            component: "pre",
            sx: sx()
                .margin("8px 0 0 0")
                .background("grey.1")
                .border("1px solid")
                .border_color("grey.3")
                .border_radius("6px")
                .padding("12px 16px")
                .overflow("auto")
                .and(monospace_sx())
                .font_size("0.875rem"),
            Box {
                component: "code",
                {children}
            }
        }
    }
}

#[component]
fn GettingStarted() -> Element {
    rsx! {
        Title { variant: "h1", id: "getting-started", "Getting Started" }
        Text {
            sx: sx().margin_top("16px"),
            "Libero is a Dioxus component library focused on developer experience, UX, accessibility, and configurability."
        }

        Title { variant: "h2", sx: sx().margin_top("32px"), "Installation" }
        Text { sx: sx().margin_top("8px"), "Add Libero to your project with cargo:" }
        CodeBlock { "cargo add libero" }

        Title { variant: "h2", sx: sx().margin_top("32px"), "Quick Start" }
        Text {
            sx: sx().margin_top("8px"),
            "Wrap your app in "
            Box { component: "code", sx: monospace_sx().background("grey.1").border_radius("4px").padding("2px 6px"), "LiberoProvider" }
            " once, at the root - it registers the theme and every style your components use."
        }
        CodeBlock {
            "fn App() -> Element {{\n    rsx! {{\n        LiberoProvider {{\n            Text {{ \"Hello, Libero!\" }}\n        }}\n    }}\n}}"
        }

        Divider { sx: sx().margin_top("32px") }
        Text {
            sx: sx().margin_top("16px").color("grey.6"),
            "More documentation is on the way."
        }
    }
}
