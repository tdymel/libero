use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Flex, Icon, List, ListItem, Text, Title},
    sx::sx,
    theme::{ICON_SIZE, Size},
};

use crate::icons::{ChevronIcon, FileIcon, FolderIcon};

// `ListItem` is a plain block-level `<li>` - a nested `List` placed inside
// one stacks below its label instead of beside it, which is what lets these
// two compose without any tree-specific code.
#[component]
fn TreeFolder(label: &'static str, expanded: bool, children: Element) -> Element {
    rsx! {
        ListItem {
            Flex { direction: "row", align: "center", gap: "sm",
                Icon {
                    variant: "transparent",
                    size: "xs",
                    color: "grey.6",
                    sx: sx().transition("transform 120ms ease")
                        .transform(if expanded { "rotate(90deg)" } else { "rotate(0deg)" }),
                    ChevronIcon {}
                }
                Icon { variant: "transparent", size: "sm", color: "primary", FolderIcon {} }
                Text { {label} }
            }
            if expanded {
                List { {children} }
            }
        }
    }
}

#[component]
fn TreeFile(label: &'static str) -> Element {
    rsx! {
        ListItem {
            Flex { direction: "row", align: "center", gap: "sm",
                // Matches the chevron's reserved width so file labels align
                // under folder labels, not under the chevron.
                Box { sx: sx().flex_shrink("0").width(ICON_SIZE.value(Size::Xs)) }
                Icon { variant: "transparent", size: "sm", color: "grey.6", FileIcon {} }
                Text { {label} }
            }
        }
    }
}

#[component]
pub fn ListPage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xxl",
            Flex {
                direction: "column",
                gap: "lg",
                Title { size: "xxl", "List" }
                Text {
                    "Renders a "
                    Code { "ul" }
                    "/"
                    Code { "li" }
                    " pair with the browser's default list styling removed - nested lists "
                    "indent relative to their own content. "
                    Code { "size" }
                    " (xs-xxl, default "
                    Code { "md" }
                    ") controls item gap and nested-list indent together."
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "Example" }
                List {
                    ListItem { "First item" }
                    ListItem { "Second item" }
                    ListItem {
                        "Third item, with a nested list"
                        List {
                            ListItem { "Nested one" }
                            ListItem { "Nested two" }
                        }
                    }
                }
            }
            Flex {
                direction: "column",
                gap: "sm",
                Title { size: "xl", "File tree" }
                Text {
                    "Purely visual - no interactivity or state, just "
                    Code { "List" }
                    "/"
                    Code { "ListItem" }
                    " nested arbitrarily deep, which is exactly the composition a future "
                    Code { "Tree" }
                    " component would build on."
                }
                List {
                    TreeFolder { label: "src", expanded: true,
                        TreeFolder { label: "components", expanded: true,
                            TreeFile { label: "list.rs" }
                            TreeFile { label: "list_item.rs" }
                            TreeFile { label: "button.rs" }
                        }
                        TreeFolder { label: "hooks", expanded: false }
                        TreeFile { label: "lib.rs" }
                        TreeFile { label: "main.rs" }
                    }
                    TreeFile { label: "Cargo.toml" }
                    TreeFile { label: "README.md" }
                }
            }
        }
    }
}
