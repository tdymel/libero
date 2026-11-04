//! Responsive Tree size, Stepper label_position, Tooltip and HoverCard gap and CircularProgress
//! thickness (todos 5676-5678, 5680): Rust-read props render their base on the server, Tree's
//! size writes run words and range copies.

use crate::common::{body, render};

use std::collections::HashSet;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Button, CircularProgress, HoverCard, Options, StepLabelPosition, Stepper, Tooltip, Tree,
        TreeItem, TreeNode, TreeNodeRenderArgs,
    },
    hooks::Side,
    theme::{Size, responsive},
};

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

macro_rules! page {
    ($ring:expr, $label:expr, $gap:expr) => {{
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    CircularProgress { aria_label: "Upload", value: 40.0, thickness: $ring }
                    Stepper::<Stage> { aria_label: "Steps", value: Some(Stage::Account), label_position: $label }
                    Tooltip { label: rsx! { "Saves" }, label_id: "tip", open: true, side: Side::Bottom, gap: $gap,
                        Button { "Save" }
                    }
                    HoverCard { aria_label: "Profile", open: true, side: Side::Bottom, gap: $gap, content: rsx! { "Ada" },
                        Button { "Ada" }
                    }
                }
            }
        }
        render(app)
    }};
}

#[test]
fn rust_read_props_render_their_base_value_on_the_server() {
    let plain = page!(Size::Xs, StepLabelPosition::Below, Size::Xs);
    let ranged = page!(
        responsive(Size::Xs).md(Size::Xl),
        responsive(StepLabelPosition::Below).md(StepLabelPosition::Side),
        responsive(Size::Xs).md(Size::Xl)
    );
    assert_eq!(body(&plain), body(&ranged));
}

macro_rules! tree_page {
    ($size:expr) => {{
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Tree {
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
            }
        }
        render(app)
    }};
}

/// Run words (`base-md:size-sm`) in the markup.
fn run_words(html: &str) -> Vec<String> {
    body(html)
        .split(r#"data-state=""#)
        .skip(1)
        .flat_map(|rest| {
            rest[..rest.find('"').unwrap_or(0)]
                .split(' ')
                .map(str::to_string)
        })
        .filter(|word| word.contains(':') && word.contains('-'))
        .collect()
}

#[test]
fn a_plain_tree_carries_no_run_word_or_copy() {
    let html = tree_page!(Size::Lg);
    assert!(run_words(&html).is_empty(), "{:?}", run_words(&html));
    assert!(!html.contains("md-:size"), "no range copy");
}

#[test]
fn a_responsive_tree_size_writes_run_words_and_copies_for_the_rows_the_groups_and_the_boxes() {
    let html = tree_page!(responsive(Size::Sm).md(Size::Xl));
    let words = run_words(&html);
    assert!(
        words.iter().any(|word| word == "base-md:size-sm"),
        "{words:?}"
    );
    assert!(words.iter().any(|word| word == "md-:size-xl"), "{words:?}");
    let copies = html.matches(r#"[data-state~="md-:size-xl"]"#).count();
    // The List, the guide padding on the row, the group's gap and the checkbox box.
    assert!(copies >= 4, "{copies} copies");
    assert!(
        html.contains("--lsx-tree-guide-padding"),
        "guide padding copy"
    );
    assert!(html.contains("padding-block-start"), "group gap copy");
}
