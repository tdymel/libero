use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Anchor, Code, Container, Divider, Drawer, Flex, Header, Icon, Image, List, ListItem, Text,
        Title,
    },
    sx::sx,
};

static LOGO: Asset = asset!("/assets/logo.svg");

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

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[layout(AppShell)]
    #[route("/")]
    GettingStarted {},
    #[route("/icon")]
    IconPage {},
}

#[component]
fn App() -> Element {
    rsx! {
        document::Title { "Libero" }
        document::Link { rel: "icon", href: LOGO }
        LiberoProvider {
            Router::<Route> {}
        }
    }
}

#[component]
fn AppShell() -> Element {
    rsx! {
        Flex {
            direction: "column",
            sx: sx().height("100vh"),
            gap: "0",
            Header {
                color: "primary",
                sx: sx().gap("md"),
                Image { src: LOGO, sx: sx().width("auto").height("28px") }
                Title { variant: "h3", component: "span", "Libero" }
            }
            Flex {
                direction: "row",
                align: "stretch",
                sx: sx().flex("1").overflow("hidden"),
                Sidebar {}
                Container {
                    component: "main",
                    size: "sm",
                    sx: sx().flex("1").padding("48px 64px").overflow("auto"),
                    Outlet::<Route> {}
                }
            }
        }
    }
}

fn chevron() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M9 18l6-6-6-6" }
        }
    }
}

fn checkmark() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M5 12l5 5L20 7" }
        }
    }
}

#[component]
fn Sidebar() -> Element {
    rsx! {
        Drawer {
            variant: "static",
            anchor: "left",
            size: "sm",
            role: "navigation",
            sx: sx().flex_shrink("0").padding("32px 24px"),
            Flex {
                direction: "column",
                gap: "12px",
                Title { variant: "h3", component: "p", "Documentation" }
                List {
                    ListItem {
                        Anchor {
                            to: Route::GettingStarted {},
                            underline: "never",
                            sx: sx().display("flex").align_items("center").gap("6px").font_weight("600"),
                            Icon { variant: "transparent", size: "xs", color: "primary", {chevron()} }
                            "Getting Started"
                        }
                    }
                    ListItem {
                        Anchor {
                            to: Route::IconPage {},
                            underline: "never",
                            sx: sx().display("flex").align_items("center").gap("6px").font_weight("600"),
                            Icon { variant: "transparent", size: "xs", color: "primary", {chevron()} }
                            "Icon"
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
                Title { variant: "h1", "Getting Started" }
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

#[component]
fn IconPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "Icon" }
                Text {
                    "Wraps an svg child in a sized, colored badge. "
                    Code { "color" }
                    " sets the container's CSS color, which any child svg using "
                    Code { "currentColor" }
                    " for its fill/stroke then inherits."
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Variants" }
                Flex {
                    direction: "row",
                    gap: "24px",
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        Icon { variant: "filled", color: "primary", {checkmark()} }
                        Text { size: "sm", "Filled" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        Icon { variant: "outlined", color: "primary", {checkmark()} }
                        Text { size: "sm", "Outlined" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        Icon { variant: "transparent", color: "primary", {checkmark()} }
                        Text { size: "sm", "Transparent" }
                    }
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Sizes" }
                Flex {
                    direction: "row",
                    gap: "16px",
                    align: "center",
                    Icon { size: "xs", {checkmark()} }
                    Icon { size: "sm", {checkmark()} }
                    Icon { size: "md", {checkmark()} }
                    Icon { size: "lg", {checkmark()} }
                    Icon { size: "xl", {checkmark()} }
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Colors" }
                Flex {
                    direction: "row",
                    gap: "16px",
                    Icon { color: "primary", {checkmark()} }
                    Icon { color: "success", {checkmark()} }
                    Icon { color: "error", {checkmark()} }
                    Icon { color: "warning", {checkmark()} }
                }
            }
        }
    }
}
