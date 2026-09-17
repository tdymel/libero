use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{
    Anchor, Button, Code, CodeBlock, Flex, Menu, MenuEntry, MenuItem, Text, use_menu,
};

const ACTIONS: &str = r#"#[component]
fn FileActions() -> Element {
    let menu = use_menu();
    let mut last = use_signal(|| String::from("nothing yet"));
    let pick = move |name: &'static str| move |_| last.set(name.to_string());

    let items: Vec<MenuEntry> = vec![
        MenuItem::new("Rename").onselect(pick("Rename")).into(),
        MenuItem::new("Duplicate").onselect(pick("Duplicate")).into(),
        MenuItem::new("Delete").onselect(pick("Delete")).into(),
    ];

    rsx! {
        Flex { direction: "row", gap: "md",
            Menu { state: menu, items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "File" }
            }
            Text { "Last chosen: {last}" }
        }
    }
}"#;

/// `ACTIONS`, rendered.
#[component]
fn FileActions() -> Element {
    let menu = use_menu();
    let mut last = use_signal(|| String::from("nothing yet"));
    let pick = move |name: &'static str| move |_| last.set(name.to_string());

    let items: Vec<MenuEntry> = vec![
        MenuItem::new("Rename").onselect(pick("Rename")).into(),
        MenuItem::new("Duplicate")
            .onselect(pick("Duplicate"))
            .into(),
        MenuItem::new("Delete").onselect(pick("Delete")).into(),
    ];

    rsx! {
        Flex { direction: "row", gap: "md",
            Menu { state: menu, items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "File" }
            }
            Text { "Last chosen: {last}" }
        }
    }
}

#[component]
pub fn UseMenuPage() -> Element {
    rsx! {
        DocPage {
            title: "use_menu",
            source: "libero/src/components/overlay/menu/state.rs",
            markdown: "/md/use_menu.md",
            lead: rsx! {
                Text {
                    Code { source: "use_menu() -> MenuState" }
                    " keeps a "
                    Code { source: "Menu" }
                    "'s open state in your scope, so your own trigger opens it and carries "
                    "its aria wiring. Pass it as the menu's "
                    Code { source: "state" }
                    ". "
                    Anchor { to: Route::MenuPage {}, "Menu" }
                    " covers groups, checkboxes, submenus and the keyboard."
                }
            },

            DocSection {
                title: "Usage",
                FileActions {}
                CodeBlock { source: ACTIONS, language: "rust" }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "Spread "
                    Code { source: "a11y_attributes()" }
                    " on the trigger. It carries the trigger's id, "
                    Code { source: "aria-haspopup" }
                    " and "
                    Code { source: "aria-expanded" }
                    ", plus "
                    Code { source: "aria-controls" }
                    " while the menu is open. The "
                    Code { source: "Menu" }
                    " around the trigger handles the click and the keys that open it."
                }
            }
        }
    }
}
