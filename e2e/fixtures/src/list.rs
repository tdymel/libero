//! `List`: a three-level nest of differently sized lists, a nested list under an icon
//! item, and a raw `ul` inside an item.

use dioxus::prelude::*;
use libero::components::{Button, List, ListItem, ListSubheader};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/list", || rsx! { ListPage {} }),
    // Todo 4147: an action at the end of an item, narrow enough to wrap the body.
    (
        "/list/action",
        || rsx! {
            div { style: "width: 240px",
                List { id: "actions",
                    ListItem {
                        id: "acted",
                        icon: rsx! { "*" },
                        action: rsx! { Button { id: "remove", size: "xs", variant: "outlined", "Remove" } },
                        "Supercalifragilisticexpialidocious"
                    }
                    ListItem {
                        id: "plain-acted",
                        action: rsx! { Button { size: "xs", variant: "outlined", "Edit" } },
                        "Second"
                    }
                }
            }
        },
    ),
    // A subheader in an ordered list sits outside the number gutter. Todo 4172: the action
    // item still counts, so the last one is 3.
    (
        "/list/subheader",
        || rsx! {
            List { id: "ordered", ordered: true,
                ListSubheader { id: "subheader", "Steps" }
                ListItem { id: "first", "Install" }
                ListItem {
                    action: rsx! { Button { size: "xs", variant: "outlined", "Edit" } },
                    "Configure"
                }
                ListItem { "Deploy" }
            }
            // Todo 4213: a reversed list counts the action item down too, so the first is 3.
            List { id: "reversed", ordered: true, "reversed": true,
                ListItem { "Install" }
                ListItem {
                    action: rsx! { Button { size: "xs", variant: "outlined", "Edit" } },
                    "Configure"
                }
                ListItem { "Deploy" }
            }
        },
    ),
];

#[component]
fn ListPage() -> Element {
    rsx! {
        List { id: "outer", size: "xs",
            ListItem { "Level one"
                List { id: "middle", size: "lg",
                    ListItem { "Level two"
                        List { id: "inner", size: "xs",
                            ListItem { "Level three" }
                        }
                    }
                }
            }
            ListItem { "Raw list"
                ul { id: "raw",
                    li { "Plain item" }
                }
            }
            ListItem { icon: rsx! { "*" }, "With an icon"
                List { id: "iconed",
                    ListItem { "Nested under the icon item" }
                }
            }
        }
        // Todo 2923: an icon floats, and a long word wraps beside it, not below.
        div { style: "width: 160px",
            List { id: "long-list",
                ListItem { id: "long-item", icon: rsx! { "*" },
                    "Supercalifragilisticexpialidocious"
                }
            }
        }
    }
}
