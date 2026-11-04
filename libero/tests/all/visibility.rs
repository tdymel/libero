//! `hidden_from` / `visible_from` on every component (todo 5647): the root carries the word, the
//! page one shared rule per word, and a plain page neither.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Title, *},
};

#[derive(Clone, Debug, PartialEq, Options)]
enum Step {
    One,
    Two,
}

/// How many elements carry `word` in their `data-state`.
fn carriers(html: &str, word: &str) -> usize {
    body(html)
        .split("data-state=\"")
        .skip(1)
        .filter(|rest| {
            let state = &rest[..rest.find('"').unwrap_or(0)];
            state.split(' ').any(|state_word| state_word == word)
        })
        .count()
}

fn assert_carried(html: &str, word: &str, expected: usize) {
    assert_eq!(carriers(html, word), expected, "{word} in {}", body(html));
    let rule = format!("[data-state~=\"{word}\"]{{display:none !important;}}");
    assert_eq!(html.matches(&rule).count(), 1, "one shared rule for {word}");
}

/// Plain components of every family; the page `plain_page_is_unchanged` dumps.
fn plain_page() -> Element {
    rsx! {
        LiberoProvider {
            Button { "Save" }
            ActionIcon { aria_label: "Add", "+" }
            ButtonGroup { Button { "A" } Button { "B" } }
            Badge { "New" }
            Avatar { name: "Ada Lovelace" }
            Alert { title: "Heads up", "Body" }
            Loader {}
            Skeleton {}
            ProgressBar { value: 40.0 }
            TextField { label: "Name" }
            Checkbox { label: "Agree" }
            Switch { label: "On" }
            Chip { "Tag" }
            Paper { "Paper" }
            Flex { "Flex" }
            GridZone { "Grid" }
            Center { "Center" }
            Container { "Container" }
            Divider {}
            Text { "Text" }
            Title { "Title" }
            Kbd { "K" }
            Code { source: "code" }
            Mark { "mark" }
            NavLink { to: "https://example.com", "Docs" }
            DataList { DataListItem { label: rsx! { "Term" }, "Value" } }
        }
    }
}

#[test]
fn plain_page_is_unchanged() {
    let html = render(plain_page);
    assert_eq!(carriers(&html, "hidden-from-md"), 0);
    assert!(!html.contains("hidden-from") && !html.contains("visible-from"));
    // Compared byte for byte against main by the R5 bundle (cmp); kept for the next sweep.
    if let Ok(path) = std::env::var("LIBERO_SSR_DUMP") {
        std::fs::write(path, &html).expect("dump the plain page");
    }
}

#[test]
fn buttons_carry_the_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { hidden_from: "md", "Save" }
                ActionIcon { hidden_from: "md", aria_label: "Add", "+" }
                ButtonGroup { hidden_from: "md", Button { "A" } }
                Chip { hidden_from: "md", "Tag" }
                Burger { hidden_from: "md" }
                Button { visible_from: "sm", "Menu" }
            }
        }
    }
    let html = render(app);
    assert_carried(&html, "hidden-from-md", 5);
    assert_carried(&html, "visible-from-sm", 1);
}

#[test]
fn data_display_carries_the_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Badge { hidden_from: "md", "New" }
                Avatar { hidden_from: "md", name: "Ada" }
                Indicator { hidden_from: "md" }
                Accordion::<Step> { hidden_from: "md", open: AccordionOpen::One(None), onchange: |_| {} }
                DataList { hidden_from: "md", DataListItem { visible_from: "lg", label: rsx! { "Term" }, "Value" } }
            }
        }
    }
    let html = render(app);
    assert_carried(&html, "hidden-from-md", 5);
    assert_carried(&html, "visible-from-lg", 1);
}

#[test]
fn feedback_carries_the_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Alert { hidden_from: "md", title: "Heads up" }
                Loader { hidden_from: "md" }
                Skeleton { hidden_from: "md" }
                ProgressBar { hidden_from: "md", value: 40.0 }
            }
        }
    }
    assert_carried(&render(app), "hidden-from-md", 4);
}

#[test]
fn form_fields_carry_the_words_on_their_wrapper() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { hidden_from: "md", label: "Name" }
                Checkbox { hidden_from: "md", label: "Agree" }
                Switch { hidden_from: "md", label: "On" }
                Slider { hidden_from: "md", value: 25.0, aria_label: "Level", oninput: move |_| {} }
            }
        }
    }
    assert_carried(&render(app), "hidden-from-md", 4);
}

#[test]
fn layout_carries_the_words_over_its_own_display() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { hidden_from: "md", "Box" }
                Paper { hidden_from: "md", "Paper" }
                Flex { hidden_from: "md", "Flex" }
                GridZone { hidden_from: "md", "Grid" }
                Center { hidden_from: "md", "Center" }
                Container { hidden_from: "md", "Container" }
                Divider { hidden_from: "md" }
            }
        }
    }
    // `!important` beats Flex's and GridZone's own `display`.
    assert_carried(&render(app), "hidden-from-md", 7);
}

#[test]
fn navigation_and_anchors_carry_the_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NavLink { hidden_from: "md", to: "https://example.com", "Docs" }
                Anchor { hidden_from: "md", to: "https://example.com", "Home" }
                NavigationBar {
                    NavigationBarItem { hidden_from: "md", to: "https://example.com/a", icon: rsx! { "A" }, "A" }
                }
            }
        }
    }
    assert_carried(&render(app), "hidden-from-md", 3);
}

#[test]
fn typography_carries_the_words() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { hidden_from: "md", "Text" }
                Title { hidden_from: "md", "Title" }
                Kbd { hidden_from: "md", "K" }
                Code { hidden_from: "md", source: "code" }
                Mark { hidden_from: "md", "mark" }
            }
        }
    }
    assert_carried(&render(app), "hidden-from-md", 5);
}
