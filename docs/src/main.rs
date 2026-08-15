use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Box, Code, Container, Divider, Flex, List, ListItem, Text, Title},
    sx::sx,
};

fn main() {
    dioxus::launch(App);
}

const QUICK_START_EXAMPLE: &str = r#"fn App() -> Element {
    rsx! {
        LiberoProvider {
            Text { "Hello, Libero!" }
        }
    }
}"#;

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
            Flex {
                direction: "column",
                gap: "12px",
                Title { variant: "h6", "Documentation" }
                List {
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
}

#[component]
fn GettingStarted() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", id: "getting-started", "Getting Started" }
                Text {
                    "Libero is a Dioxus component library focused on developer experience, UX, accessibility, and configurability."
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Installation" }
                Text { "Add Libero to your project with cargo:" }
                Code { block: true, "cargo add libero" }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Quick Start" }
                Text {
                    "Wrap your app in "
                    Code { "LiberoProvider" }
                    " once, at the root - it registers the theme and every style your components use."
                }
                Code { block: true, {QUICK_START_EXAMPLE} }
            }

            Flex {
                direction: "column",
                gap: "16px",
                Divider {}
                Text {
                    sx: sx().color("grey.6"),
                    "More documentation is on the way."
                }
            }
        }
    }
}
