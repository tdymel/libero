use dioxus::prelude::*;
use libero::{
    components::{Flex, Kbd, List, ListItem, Table, Text, Title, column},
    sx::sx,
};

use super::prop_doc::prose;

/// One row of the Accessibility tab's keyboard table.
#[derive(Clone, PartialEq)]
struct KeyRow {
    keys: Vec<String>,
    action: String,
}

/// A docs page's Accessibility tab: the keys, what Libero handles, what the
/// caller must do, and known limits. Empty parts are not drawn.
///
/// ```ignore
/// a11y()
///     .key(["Escape"], "Dismisses the modal.")
///     .handles(["Focus returns to the trigger."])
///     .must(["Name the `Dialog` with its `title`."])
/// ```
#[derive(Clone, PartialEq, Default)]
pub struct A11yDoc {
    keys: Vec<KeyRow>,
    handles: Vec<String>,
    must: Vec<String>,
    example: Option<String>,
    limits: Vec<String>,
}

/// Starts an Accessibility tab.
pub fn a11y() -> A11yDoc {
    A11yDoc::default()
}

fn strings<S: Into<String>>(items: impl IntoIterator<Item = S>) -> Vec<String> {
    items.into_iter().map(Into::into).collect()
}

impl A11yDoc {
    /// A keyboard row. Each key is an alternative; `"Shift+Tab"` is a chord.
    pub fn key<S: Into<String>>(
        mut self,
        keys: impl IntoIterator<Item = S>,
        action: impl Into<String>,
    ) -> Self {
        self.keys.push(KeyRow {
            keys: strings(keys),
            action: action.into(),
        });
        self
    }

    /// What the component does itself, so the caller does not redo it.
    pub fn handles<S: Into<String>>(mut self, items: impl IntoIterator<Item = S>) -> Self {
        self.handles.extend(strings(items));
        self
    }

    /// What the caller has to do.
    pub fn must<S: Into<String>>(mut self, items: impl IntoIterator<Item = S>) -> Self {
        self.must.extend(strings(items));
        self
    }

    /// One concrete case of the split above: what the app writes, what a user then gets.
    pub fn example(mut self, text: impl Into<String>) -> Self {
        self.example = Some(text.into());
        self
    }

    /// Known gaps, such as a platform that lacks something.
    pub fn limits<S: Into<String>>(mut self, items: impl IntoIterator<Item = S>) -> Self {
        self.limits.extend(strings(items));
        self
    }
}

/// The Accessibility tab's body.
#[component]
pub fn A11yPanel(doc: A11yDoc) -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "xl",
            if !doc.keys.is_empty() {
                Flex {
                    direction: "column",
                    gap: "sm",
                    // h2 under the page's h1, as `PropertyTable`'s group titles.
                    Title { size: "lg", component: "h2", "Keyboard" }
                    Table {
                        aria_label: "Keyboard",
                        sx: sx().selector("& td", sx().vertical_align("top")),
                        data: doc.keys,
                        columns: vec![
                            column("Key")
                                .value(|row: &KeyRow| row.keys.join(" or "))
                                .render(|row: &KeyRow| rsx! { KeyCell { keys: row.keys.clone() } }),
                            column("Action")
                                .value(|row: &KeyRow| row.action.clone())
                                .render(|row: &KeyRow| prose(&row.action)),
                        ],
                    }
                }
            }
            A11yList { title: "Libero handles", items: doc.handles }
            A11yList { title: "You must", items: doc.must }
            if let Some(example) = doc.example {
                Flex {
                    direction: "column",
                    gap: "sm",
                    Title { size: "lg", component: "h2", "Example" }
                    Text { {prose(&example)} }
                }
            }
            A11yList { title: "Limits", items: doc.limits }
        }
    }
}

/// Alternatives joined by "or", a chord's keys by "+".
#[component]
fn KeyCell(keys: Vec<String>) -> Element {
    rsx! {
        Flex { direction: "row", wrap: "wrap", gap: "xs", align: "baseline", sx: sx().white_space("nowrap"),
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    Text { component: "span", sx: sx().color("text-dimmed"), "or" }
                }
                span {
                    for (j, part) in key.split('+').enumerate() {
                        if j > 0 {
                            "+"
                        }
                        Kbd { "{part}" }
                    }
                }
            }
        }
    }
}

#[component]
fn A11yList(title: String, items: Vec<String>) -> Element {
    if items.is_empty() {
        return rsx! {};
    }
    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            Title { size: "lg", component: "h2", "{title}" }
            List {
                size: "sm",
                // `List` draws no markers of its own; these are plain bullets.
                sx: sx().list_style_type("disc").padding_inline_start("1.5em"),
                for item in items {
                    ListItem { {prose(&item)} }
                }
            }
        }
    }
}
