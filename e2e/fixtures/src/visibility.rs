//! `hidden_from` / `visible_from` on one component per family (todo 5647), and on the ones
//! with their own `display`: Flex, GridZone and an open Dialog.

use dioxus::prelude::*;
use libero::components::{Title, *};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/visibility", || rsx! { VisibilityPage {} }),
    ("/visibility/cascader", || rsx! { CascaderVisibilityPage {} }),
];

#[component]
fn VisibilityPage() -> Element {
    rsx! {
        Button { hidden_from: "md", "Button" }
        ActionIcon { hidden_from: "md", aria_label: "Add", "+" }
        Badge { hidden_from: "md", "Badge" }
        Avatar { hidden_from: "md", name: "Ada Lovelace" }
        Alert { hidden_from: "md", title: "Alert" }
        Loader { hidden_from: "md" }
        TextField { hidden_from: "md", label: "Field" }
        Checkbox { hidden_from: "md", label: "Check" }
        Paper { hidden_from: "md", "Paper" }
        Flex { hidden_from: "md", "Flex" }
        GridZone { hidden_from: "md", "Grid" }
        NavLink { hidden_from: "md", to: "https://example.com", "NavLink" }
        Anchor { hidden_from: "md", to: "https://example.com", "Anchor" }
        Text { hidden_from: "md", "Text" }
        Title { hidden_from: "md", "Title" }
        Button { visible_from: "md", "Wide button" }
        Badge { visible_from: "md", "Wide badge" }
        Flex { visible_from: "md", "Wide flex" }
        Dialog { hidden_from: "md", aria_label: "Narrow dialog", onclose: |_| {}, "Dialog" }
    }
}

/// Todo 5703: `CascaderDropdown`'s `states` and `visible_from` land on its open box.
#[component]
fn CascaderVisibilityPage() -> Element {
    let mut ore = use_signal(|| None::<String>);
    let cascader = use_cascader(
        vec![
            CascaderOption::new("t1".to_string(), "Tier 1")
                .children(vec![CascaderOption::new("iron".to_string(), "Iron ore")]),
        ],
        ore(),
        move |next| ore.set(next),
    );
    rsx! {
        CascaderDropdown {
            state: cascader.clone(),
            visible_from: "md",
            states: States::new().with("ore-box", true),
            Button { attributes: cascader.a11y_attributes(), "Ore" }
        }
    }
}
