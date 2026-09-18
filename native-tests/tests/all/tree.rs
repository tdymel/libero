//! `Tree`'s chevron turns by a state-driven `transform` when a row expands,
//! and Enter on a leaf activates the link it renders (todo 4): natively that
//! is `ElementApi::click` queueing `UiEvent::Activate`.

use dioxus::prelude::*;
use libero::components::{Tree, TreeItem, TreeNode, TreeNodeRenderArgs};
use native_tests::{Key, mount};

const BRANCH: &str = "[role=treeitem][aria-expanded]";
const CHEVRON: &str = "[role=treeitem][aria-expanded] [data-tree-chevron]";
const ROW: &str = "[role=treeitem]";

fn app() -> Element {
    let data = vec![
        TreeNode::new("src", "src").children(vec![TreeNode::new("src/lib.rs", "lib.rs")]),
        TreeNode::new("README.md", "README.md"),
    ];
    rsx! {
        Tree {
            aria_label: "Files",
            data,
        }
    }
}

fn link_app() -> Element {
    let mut activated = use_signal(|| String::from("none"));
    let data = vec![
        TreeNode::new("here", "Here"),
        TreeNode::new("elsewhere", "Elsewhere"),
    ];
    rsx! {
        output { id: "activated", "{activated}" }
        Tree {
            aria_label: "Pages",
            data,
            render_node: move |args: TreeNodeRenderArgs<&'static str>| {
                let id = args.id.to_string();
                rsx! {
                    a {
                        tabindex: args.tabindex,
                        onclick: move |event: MouseEvent| {
                            event.prevent_default();
                            activated.set(id.clone());
                        },
                        "{args.data}"
                    }
                }
            },
        }
    }
}

/// Blitz's UA sheet centres a button's content; a row starts at its edge.
#[test]
fn a_row_starts_its_content_at_the_edge() {
    let page = mount(|| {
        rsx! {
            Tree {
                aria_label: "Files",
                data: vec![TreeNode::new("README.md", "README.md")],
                render_node: |args: TreeNodeRenderArgs<&'static str>| rsx! {
                    TreeItem { span { "{args.data}" } }
                },
            }
        }
    });
    let row = page.rect("[role=treeitem] button");
    let first = page.rect("[role=treeitem] button > *");
    assert_eq!(
        first.0,
        row.0,
        "row {row:?}, first child {first:?}\n{}",
        page.tree()
    );
}

#[test]
fn the_chevron_turns_when_the_branch_expands() {
    let mut page = mount(app);
    let collapsed = page.computed(CHEVRON, "transform");
    page.focus("[tabindex='0']");
    page.press(Key::ArrowRight);
    assert_eq!(
        page.attr(BRANCH, "aria-expanded").as_deref(),
        Some("true"),
        "{}",
        page.tree()
    );
    page.advance(1.0);
    let expanded = page.computed(CHEVRON, "transform");
    assert_ne!(collapsed, expanded, "the chevron stayed at {collapsed}");
}

#[test]
fn the_painted_chevron_turns_too() {
    let mut page = mount(app);
    let collapsed = page.painted_transform(CHEVRON);
    page.focus("[tabindex='0']");
    page.press(Key::ArrowRight);
    page.advance(1.0);
    let expanded = page.painted_transform(CHEVRON);
    assert_ne!(
        collapsed, expanded,
        "the painted chevron stayed at {collapsed:?}"
    );
}

#[test]
fn enter_on_a_leaf_clicks_its_link() {
    let mut page = mount(link_app);
    page.tab();
    assert!(
        page.is_focused(ROW),
        "Tab reached {}, not a row",
        page.focus_owner()
    );

    page.press(Key::ArrowDown);
    page.press(Key::Enter);
    assert_eq!(page.text("#activated"), "elsewhere", "{}", page.tree());
}
