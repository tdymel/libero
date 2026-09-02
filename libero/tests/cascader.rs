//! `Cascader`'s rendered contract while it is closed: the trigger is a `div`
//! with a listbox's wiring, the value it shows is the joined path, and the
//! value it *posts* is one hidden input per level.
//!
//! The open list cannot be reached from here. It is portaled by `use_popover`
//! and only exists after a click, and the columns are placed by measurements
//! the SSR harness answers `Unsupported` for - so everything below the trigger
//! is browser work by construction ([[codebase/testing]]).

mod common;

use common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Cascader, CascaderPick, TreeNode},
};

fn categories() -> Vec<TreeNode<&'static str>> {
    vec![
        TreeNode::new("food", "Food").children(vec![
            TreeNode::new("fruit", "Fruit").children(vec![
                TreeNode::new("apple", "Apple"),
                TreeNode::new("pear", "Pear"),
            ]),
            TreeNode::new("veg", "Veg").children(vec![TreeNode::new("leek", "Leek")]),
        ]),
        TreeNode::new("drink", "Drink").children(vec![TreeNode::new("tea", "Tea")]),
    ]
}

fn picked() -> Vec<String> {
    vec!["food".to_string(), "fruit".to_string(), "apple".to_string()]
}

fn chosen() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                placeholder: "Pick a category",
                data: categories(),
                value: picked(),
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

fn empty() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                placeholder: "Pick a category",
                data: categories(),
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

fn dashed() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                data: categories(),
                value: picked(),
                separator: " - ",
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

fn formatted() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                data: categories(),
                value: picked(),
                format_value: move |nodes: Vec<&'static str>| nodes.last().copied().unwrap_or("").to_string(),
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

fn posting() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                name: "category",
                data: categories(),
                value: picked(),
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

fn off() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                name: "category",
                data: categories(),
                value: picked(),
                disabled: true,
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

fn stale() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                placeholder: "Pick a category",
                data: categories(),
                // `apple` exists, but not as a root - a path is a path, not a
                // bag of ids.
                value: vec!["apple".to_string()],
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

fn clearable() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                data: categories(),
                value: picked(),
                clearable: true,
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

fn clearable_empty() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                data: categories(),
                clearable: true,
                onchange: move |_: CascaderPick<&'static str>| {},
            }
        }
    }
}

/// The open tag of the element carrying the combobox role.
fn trigger_of(html: &str) -> String {
    let at = html
        .find("role=\"combobox\"")
        .unwrap_or_else(|| panic!("no combobox in the rendered output:\n{html}"));
    let start = html[..at].rfind('<').expect("an unterminated tag");
    let end = at + html[at..].find('>').expect("an unterminated tag");
    html[start..=end].to_string()
}

/// The trigger is a `div`, which `<label for>` cannot name - so this field
/// takes the `use_field().labelled_by()` route `Select` and `RadioGroup` take,
/// and the trigger is a tab stop of its own.
#[test]
fn the_trigger_is_a_named_focusable_combobox() {
    let html = body(&render(chosen));
    let trigger = trigger_of(&html);

    assert!(trigger.starts_with("<div"), "{trigger}");
    assert!(trigger.contains("tabindex=\"0\""), "{trigger}");
    assert!(trigger.contains("aria-haspopup=\"listbox\""), "{trigger}");
    assert!(trigger.contains("aria-expanded=\"false\""), "{trigger}");

    let at = trigger
        .find("aria-labelledby=\"")
        .unwrap_or_else(|| panic!("the trigger is not named:\n{trigger}"));
    let rest = &trigger[at + "aria-labelledby=\"".len()..];
    let label_id = &rest[..rest.find('"').expect("an unterminated value")];
    assert!(
        html.contains(&format!("id=\"{label_id}\"")),
        "aria-labelledby names an element that is not there:\n{html}"
    );
}

/// `aria-controls` names the list even while it is closed, which is what makes
/// it one stable target for both layouts - the columns' wrapper in one and the
/// single listbox in the other.
#[test]
fn the_trigger_points_at_one_list() {
    let trigger = trigger_of(&body(&render(chosen)));
    assert!(trigger.contains("aria-controls=\""), "{trigger}");
    // Nothing is highlighted while the list is closed, so there is no row for
    // `aria-activedescendant` to point at.
    assert!(!trigger.contains("aria-activedescendant"), "{trigger}");
}

/// The value slot shows the whole path, not the leaf - that is the difference
/// between this and a `Select` over the leaves.
#[test]
fn the_trigger_shows_the_joined_path() {
    assert!(
        body(&render(chosen)).contains("Food / Fruit / Apple"),
        "the default separator joins every level"
    );
    assert!(
        body(&render(dashed)).contains("Food - Fruit - Apple"),
        "`separator` reaches the trigger"
    );
    assert!(
        body(&render(formatted)).contains(">Apple<"),
        "`format_value` replaces the join outright"
    );
    assert!(
        !body(&render(formatted)).contains("Food / Fruit"),
        "and nothing else draws the path beside it"
    );
}

/// The placeholder is what an empty value shows, and an unresolvable one is
/// empty: `value` is a *path*, so an id that exists deeper in the tree is not
/// a root and selects nothing.
#[test]
fn a_value_that_is_not_a_path_selects_nothing() {
    assert!(body(&render(empty)).contains("Pick a category"));
    assert!(body(&render(stale)).contains("Pick a category"));
    assert!(!body(&render(stale)).contains("Apple"));
    assert!(!body(&render(chosen)).contains("Pick a category"));
}

/// A `div` cannot carry a `name`. One hidden input per level, all sharing it -
/// a repeated name is an ordered list on the wire, which is the shape
/// `MultiSelect` and `TagsField` already post.
#[test]
fn the_path_posts_one_hidden_input_per_level() {
    let html = body(&render(posting));

    assert_eq!(
        html.matches("type=\"hidden\"").count(),
        3,
        "one per level of the path:\n{html}"
    );
    assert_eq!(
        html.matches("name=\"category\"").count(),
        3,
        "all of them share the field's name:\n{html}"
    );
    for id in ["food", "fruit", "apple"] {
        assert!(
            html.contains(&format!("value=\"{id}\"")),
            "the ids post, not the labels:\n{html}"
        );
    }
}

/// A disabled field sends nothing and is not a tab stop, the way a native
/// disabled control behaves.
#[test]
fn a_disabled_cascader_neither_focuses_nor_posts() {
    let html = body(&render(off));
    let trigger = trigger_of(&html);

    assert!(!trigger.contains("tabindex"), "{trigger}");
    assert!(trigger.contains("aria-disabled=\"true\""), "{trigger}");
    assert_eq!(
        html.matches("type=\"hidden\"").count(),
        3,
        "the inputs are still there, disabled:\n{html}"
    );
    assert_eq!(
        html.matches("disabled=true").count(),
        3,
        "and every one of them carries it:\n{html}"
    );
}

/// The x replaces the chevron rather than sitting beside it, and only while
/// there is something to clear.
#[test]
fn the_clear_button_appears_only_with_a_selection() {
    assert!(
        body(&render(clearable)).contains("aria-label=\"Clear\""),
        "a chosen path is clearable"
    );
    assert!(
        !body(&render(clearable_empty)).contains("aria-label=\"Clear\""),
        "an empty one has nothing to clear"
    );
}
