//! `Flex` as another element than a `div`.

use dioxus::prelude::*;
use libero::components::{Box as LBox, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/flex", || rsx! { FlexPage {} })];

/// Todo 4102: a Flex as a list, its items `li`.
#[component]
fn FlexPage() -> Element {
    rsx! {
        Flex { id: "flex-list", component: "ul", direction: "row", aria_label: "Tags",
            LBox { component: "li", "One" }
            LBox { component: "li", "Two" }
        }
    }
}
