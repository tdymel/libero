//! `Cascader`'s rendered contract while it is closed: the trigger is a `div`
//! with a listbox's wiring, the value it shows is the joined path to its
//! value, and what it *posts* is that one value.
//!
//! These are closed-state renders. Clicks and keys on the open list are
//! dispatched tests in `combobox.rs`; the column placement needs measurements
//! and is e2e's.

use std::collections::BTreeMap;

use crate::common::{body, render, render_with, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Cascader, CascaderOption, Fields, Form, Options},
};

fn categories() -> Vec<CascaderOption<String>> {
    vec![
        CascaderOption::new("food", "Food").children(vec![
            CascaderOption::new("fruit", "Fruit").children(vec![
                CascaderOption::new("apple", "Apple"),
                CascaderOption::new("pear", "Pear"),
            ]),
            CascaderOption::new("veg", "Veg").children(vec![CascaderOption::new("leek", "Leek")]),
        ]),
        CascaderOption::new("drink", "Drink").children(vec![CascaderOption::new("tea", "Tea")]),
    ]
}

/// A leaf's value, three levels down: the path to it is the cascader's to find.
fn picked() -> Option<String> {
    Some("apple".to_string())
}

/// What a test sets on the one cascader; the rest is the plain case.
#[derive(Clone, Default)]
struct Setup {
    value: Option<String>,
    name: &'static str,
    separator: Option<String>,
    /// `format_value` keeps the last label only.
    last_label: bool,
    disabled: bool,
    any_level: bool,
    clearable: bool,
}

fn cascader(setup: Setup) -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Category",
                placeholder: "Pick a category",
                data: categories(),
                value: setup.value,
                name: setup.name,
                separator: setup.separator,
                format_value: setup.last_label.then(|| {
                    Callback::new(|labels: Vec<String>| labels.last().cloned().unwrap_or_default())
                }),
                disabled: setup.disabled,
                any_level: setup.any_level,
                clearable: setup.clearable,
                onchange: move |_: Option<String>| {},
            }
        }
    }
}

/// The cascader's markup for `setup`, without the sheets.
fn shown(setup: Setup) -> String {
    body(&render_with(cascader, setup))
}

/// The leaf value picked, and nothing else set.
fn chosen() -> Setup {
    Setup {
        value: picked(),
        ..Setup::default()
    }
}

/// [`chosen`], posting as `category`.
fn posting() -> Setup {
    Setup {
        name: "category",
        ..chosen()
    }
}

/// The attributes of the one element carrying the combobox role.
fn trigger_of(html: &str) -> BTreeMap<String, String> {
    let triggers = tags_with(html, r#"role="combobox""#);
    assert_eq!(triggers.len(), 1, "{html}");
    triggers[0].clone()
}

/// The trigger is a `div`, which `<label for>` cannot name - so this field
/// takes the `use_field().labelled_by()` route `Select` and `RadioGroup` take,
/// and the trigger is a tab stop of its own.
#[test]
fn the_trigger_is_a_named_focusable_combobox() {
    let html = shown(chosen());
    let trigger = trigger_of(&html);

    assert!(tags_with(&html, "<div").contains(&trigger), "{trigger:?}");
    assert_eq!(trigger["tabindex"], "0", "{trigger:?}");
    assert_eq!(trigger["aria-haspopup"], "listbox", "{trigger:?}");
    assert_eq!(trigger["aria-expanded"], "false", "{trigger:?}");

    let label_id = trigger
        .get("aria-labelledby")
        .unwrap_or_else(|| panic!("the trigger is not named:\n{trigger:?}"));
    assert!(
        html.contains(&format!("id=\"{label_id}\"")),
        "aria-labelledby names an element that is not there:\n{html}"
    );
}

/// A closed list is not in the DOM, so `aria-controls` has nothing to name
/// until it opens (todo 360).
#[test]
fn a_closed_trigger_points_at_no_list() {
    let trigger = trigger_of(&shown(chosen()));
    assert!(!trigger.contains_key("aria-controls"), "{trigger:?}");
    // Nothing is highlighted while the list is closed, so there is no row for
    // `aria-activedescendant` to point at.
    assert!(
        !trigger.contains_key("aria-activedescendant"),
        "{trigger:?}"
    );
}

/// The value is one leaf's, but the value slot shows the whole path to it,
/// found in `data` - that is the difference between this and a `Select` over
/// the leaves.
#[test]
fn the_trigger_shows_the_joined_path() {
    assert!(
        shown(chosen()).contains("Food / Fruit / Apple"),
        "the default separator joins every level"
    );
    assert!(
        shown(Setup {
            separator: Some(" - ".into()),
            ..chosen()
        })
        .contains("Food - Fruit - Apple"),
        "`separator` reaches the trigger"
    );
    assert!(
        shown(Setup {
            last_label: true,
            ..chosen()
        })
        .contains(">Apple<"),
        "`format_value` replaces the join outright"
    );
    assert!(
        !shown(Setup {
            last_label: true,
            ..chosen()
        })
        .contains("Food / Fruit"),
        "and nothing else draws the path beside it"
    );
}

/// The placeholder is what an empty value shows, and so is a value no option
/// holds.
#[test]
fn a_value_that_is_not_in_the_tree_selects_nothing() {
    assert!(shown(Setup::default()).contains("Pick a category"));
    assert!(
        shown(Setup {
            value: Some("banana".into()),
            ..Setup::default()
        })
        .contains("Pick a category")
    );
    assert!(
        !shown(Setup {
            value: Some("banana".into()),
            ..Setup::default()
        })
        .contains("banana")
    );
    assert!(!shown(chosen()).contains("Pick a category"));
}

/// A `div` cannot carry a `name`, so one hidden input does - carrying the
/// value alone, not the path to it. Nothing selected posts nothing.
#[test]
fn the_value_posts_as_one_hidden_input() {
    let html = shown(posting());

    assert_eq!(
        html.matches("type=\"hidden\"").count(),
        1,
        "one input, not one per level:\n{html}"
    );
    assert!(html.contains("name=\"category\""), "{html}");
    assert!(
        html.contains("value=\"apple\""),
        "the value posts, not the label or the path:\n{html}"
    );
    assert!(!html.contains("value=\"food\""), "{html}");
    assert!(
        !shown(Setup::default()).contains("type=\"hidden\""),
        "nothing selected posts nothing"
    );
}

/// With `any_level` a branch is a value like any other: it posts its own
/// value, and the trigger shows the path down to it and no further.
#[test]
fn a_branch_value_shows_the_path_to_the_branch() {
    let html = shown(Setup {
        value: Some("fruit".into()),
        any_level: true,
        ..posting()
    });

    assert!(html.contains("Food / Fruit<"), "{html}");
    assert!(html.contains("value=\"fruit\""), "{html}");
}

/// A disabled field sends nothing and is not a tab stop, the way a native
/// disabled control behaves.
#[test]
fn a_disabled_cascader_neither_focuses_nor_posts() {
    let html = shown(Setup {
        disabled: true,
        ..posting()
    });
    let trigger = trigger_of(&html);

    assert!(!trigger.contains_key("tabindex"), "{trigger:?}");
    assert_eq!(trigger["aria-disabled"], "true", "{trigger:?}");
    assert_eq!(
        html.matches("type=\"hidden\"").count(),
        1,
        "the input is still there, disabled:\n{html}"
    );
    assert_eq!(
        html.matches("disabled=true").count(),
        1,
        "and it carries it:\n{html}"
    );
}

/// The x replaces the chevron rather than sitting beside it, and only while
/// there is something to clear.
#[test]
fn the_clear_button_appears_only_with_a_selection() {
    assert!(
        shown(Setup {
            clearable: true,
            ..chosen()
        })
        .contains("aria-label=\"Clear\""),
        "a chosen path is clearable"
    );
    assert!(
        !shown(Setup {
            clearable: true,
            ..Setup::default()
        })
        .contains("aria-label=\"Clear\""),
        "an empty one has nothing to clear"
    );
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Listing {
    category: Option<String>,
}

fn bound() -> Element {
    let listing = use_store(|| Listing {
        category: Some("leek".to_string()),
    });
    rsx! {
        LiberoProvider {
            Form {
                value: listing,
                Cascader { label: "Category", data: categories(), name: Listing::FIELDS.category() }
            }
        }
    }
}

/// Bound by `name`, the form's field holds one `Option<String>` - the value,
/// not the path - and the cascader finds the path to it for the trigger.
#[test]
fn a_form_field_holds_the_value_and_the_trigger_shows_its_path() {
    let html = body(&render(bound));

    assert!(html.contains("Food / Veg / Leek"), "{html}");
    assert!(html.contains("name=\"category\""), "{html}");
    assert!(html.contains("value=\"leek\""), "{html}");
}

/// A value that is not a `String` - anything a `Select` could hold.
#[derive(Clone, PartialEq, Debug)]
struct Aisle {
    id: u32,
}

impl Options for Aisle {
    fn label(&self) -> String {
        format!("Aisle {}", self.id)
    }

    fn value(&self) -> String {
        self.id.to_string()
    }
}

fn aisles() -> Element {
    rsx! {
        LiberoProvider {
            Cascader {
                label: "Aisle",
                name: "aisle",
                data: vec![
                    CascaderOption::new(Aisle { id: 1 }, "Food").children(vec![
                        CascaderOption::new(Aisle { id: 7 }, "Fruit"),
                    ]),
                ],
                value: Aisle { id: 7 },
                onchange: move |_: Option<Aisle>| {},
            }
        }
    }
}

/// The value can be any `T: Options`. The path to it is found by `==`, the
/// trigger shows the options' own labels, and what posts is
/// `Options::value()` - the `Select` contract.
#[test]
fn a_complex_value_finds_its_path_and_posts_its_options_value() {
    let html = body(&render(aisles));

    assert!(html.contains("Food / Fruit<"), "{html}");
    assert!(html.contains("name=\"aisle\""), "{html}");
    assert!(html.contains("value=\"7\""), "{html}");
}
