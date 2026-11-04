//! Responsive Tree size, Stepper label_position, Tooltip and HoverCard gap and CircularProgress
//! thickness (todos 5676-5678, 5680): each responsive element sits beside two plain twins,
//! `-narrow` (its base value) and `-wide` (its `md` value).

use std::collections::HashSet;

use dioxus::prelude::*;
use libero::components::{
    Button, CircularProgress, HoverCard, Options, StepLabelPosition, Stepper, Tooltip, Tree,
    TreeItem, TreeNode, TreeNodeRenderArgs,
};
use libero::hooks::Side;
use libero::theme::{Size, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive-r4f", || rsx! { R4fPage {} })];

#[derive(Clone, Debug, PartialEq, Options)]
enum Stage {
    Account,
    Review,
}

fn nodes() -> Vec<TreeNode<String>> {
    vec![
        TreeNode::new("src", "src".to_string())
            .children(vec![TreeNode::new("main", "main.rs".to_string())]),
    ]
}

macro_rules! tree {
    ($id:expr, $size:expr) => {
        rsx! {
            Tree {
                id: $id,
                aria_label: "Files",
                size: $size,
                guides: true,
                data: nodes(),
                default_expanded: HashSet::from(["src".to_string()]),
                checked: HashSet::from(["main".to_string()]),
                oncheckedchange: |_| {},
                render_node: move |args: TreeNodeRenderArgs<String>| rsx! { TreeItem { "{args.data}" } },
            }
        }
    };
}

macro_rules! tip {
    ($id:literal, $gap:expr) => {
        rsx! {
            Tooltip {
                label: rsx! { "Saves" },
                label_id: concat!($id, "-label"),
                open: true,
                side: Side::Bottom,
                gap: $gap,
                Button { id: concat!($id, "-trigger"), "aria-describedby": concat!($id, "-label"), "Save" }
            }
        }
    };
}

macro_rules! card {
    ($id:literal, $gap:expr) => {
        rsx! {
            HoverCard {
                aria_label: "Profile",
                id: $id,
                open: true,
                side: Side::Bottom,
                gap: $gap,
                content: rsx! { "Ada" },
                Button { id: concat!($id, "-trigger"), "Ada" }
            }
        }
    };
}

#[component]
fn R4fPage() -> Element {
    let sized = responsive(Size::Xs).md(Size::Xl);
    rsx! {
        div { style: "padding: 24px 120px; display: flex; gap: 160px",
            {tip!("r4-tip", sized)}
            {tip!("r4-tip-narrow", Size::Xs)}
            {tip!("r4-tip-wide", Size::Xl)}
        }
        div { style: "padding: 24px 120px; display: flex; gap: 160px",
            {card!("r4-card", sized)}
            {card!("r4-card-narrow", Size::Xs)}
            {card!("r4-card-wide", Size::Xl)}
        }
        div { style: "padding: 160px 24px 24px; display: flex; gap: 24px; align-items: flex-start",
            CircularProgress { id: "r4-ring", aria_label: "Upload", value: 40.0, size: Size::Xl, thickness: sized }
            CircularProgress { id: "r4-ring-narrow", aria_label: "Upload", value: 40.0, size: Size::Xl, thickness: Size::Xs }
            CircularProgress { id: "r4-ring-wide", aria_label: "Upload", value: 40.0, size: Size::Xl, thickness: Size::Xl }
        }
        div { style: "padding: 24px; display: flex; gap: 48px",
            {tree!("r4-tree", sized)}
            {tree!("r4-tree-narrow", Size::Xs)}
            {tree!("r4-tree-wide", Size::Xl)}
        }
        div { style: "padding: 24px",
            Stepper::<Stage> {
                id: "r4-stepper",
                aria_label: "Steps",
                value: Some(Stage::Account),
                label_position: responsive(StepLabelPosition::Below).md(StepLabelPosition::Side),
            }
        }
    }
}
