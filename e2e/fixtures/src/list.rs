//! `List`: a three-level nest of differently sized lists, a nested list under an icon
//! item, and a raw `ul` inside an item.

use dioxus::prelude::*;
use libero::components::{List, ListItem};

use crate::Routes;

pub const ROUTES: Routes = &[("/list", || rsx! { ListPage {} })];

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
