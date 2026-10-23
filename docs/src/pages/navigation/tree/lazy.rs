use std::collections::HashSet;

use dioxus::prelude::*;
use libero::{
    components::{Flex, Text, Tree, TreeNode},
    hooks::use_timeout,
    sx::sx,
};

/// A folder's children once loaded, or a disabled "Loading…" row that keeps it a branch.
fn folder(id: &str, loaded: bool, files: &[&str]) -> TreeNode<String> {
    let children = match loaded {
        true => files
            .iter()
            .map(|file| TreeNode::new(format!("{id}/{file}"), file.to_string()))
            .collect(),
        false => {
            vec![TreeNode::new(format!("{id}/loading"), "Loading…".to_string()).disabled(true)]
        }
    };
    TreeNode::new(id, id.to_string()).children(children)
}

#[component]
pub fn LazyFiles() -> Element {
    let mut expanded = use_signal(HashSet::<String>::new);
    let mut loaded = use_signal(HashSet::<String>::new);
    let mut pending = use_signal(HashSet::<String>::new);
    // Stands in for a fetch: whatever opened meanwhile arrives at once.
    let fetch = use_timeout(
        move || {
            let arrived = pending.take();
            loaded.write().extend(arrived);
        },
        800,
    );
    let data = vec![
        folder("src", loaded.read().contains("src"), &["lib.rs", "main.rs"]),
        folder("docs", loaded.read().contains("docs"), &["guide.md"]),
    ];
    let status = match pending.read().is_empty() {
        true => String::new(),
        false => "Loading…".to_string(),
    };

    rsx! {
        Flex { direction: "column", gap: "sm", sx: sx().width("100%").max_width("320px"),
            Tree {
                aria_label: "Lazy files",
                data,
                expanded: expanded(),
                onexpandedchange: move |next: HashSet<String>| {
                    let opened = next.iter().filter(|id| !loaded.read().contains(*id));
                    let opened: Vec<String> = opened.cloned().collect();
                    if !opened.is_empty() {
                        pending.write().extend(opened);
                        fetch.start();
                    }
                    expanded.set(next);
                },
            }
            // Always mounted, so a screen reader hears the text it gains.
            Text { role: "status", size: "sm", "{status}" }
        }
    }
}
