//! `Spotlight`: Ctrl+K opens the palette from wherever focus is, and a
//! narrowed list keeps its labels whole.

use dioxus::prelude::*;
use libero::components::{
    Button, SpotlightAction, SpotlightOptions, spotlight_filter, use_spotlight,
};
use native_tests::{Key, Modifiers, mount};

const PALETTE: &str = "[role=dialog]";

fn app() -> Element {
    let all = use_hook(|| {
        vec![
            SpotlightAction::new("Home"),
            SpotlightAction::new("Changelog"),
        ]
    });
    let _spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        aria_label: Some("Command palette".into()),
        ..Default::default()
    });
    rsx! {
        Button { id: "page", "Page" }
    }
}

// Two rows keep their place when "G" drops the others: their labels are the ones that broke.
fn narrowing() -> Element {
    let all =
        use_hook(|| {
            let mut all = vec![
                SpotlightAction::new("Getting Started").group("About"),
                SpotlightAction::new("Theming").group("About"),
                SpotlightAction::new("Getting Started").group("Form"),
                SpotlightAction::new("Select").group("Inputs"),
                SpotlightAction::new("MultiSelect").group("Inputs"),
            ];
            // A list this long, as in the docs: an eight-row tail never broke.
            all.extend((0..60).map(|n| {
                SpotlightAction::new(format!("Page {n}")).group(format!("Group {}", n / 8))
            }));
            all
        });
    let _spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| {
            spotlight_filter(&query, &all)
        })),
        aria_label: Some("Command palette".into()),
        ..Default::default()
    });
    rsx! {
        Button { id: "page", "Page" }
    }
}

// Todo 627: the painted text, not the box, wrapped; so this counts the label's laid-out lines.
#[test]
fn a_narrowed_list_keeps_each_label_on_one_line() {
    let mut page = mount(narrowing);
    page.focus("#page");
    page.press_with(Key::Character("k".into()), Modifiers::CONTROL);
    page.press_before_layout(Key::Character("G".into()));
    assert!(page.exists("[role=option]"), "no rows:\n{}", page.tree());
    assert_eq!(page.wrapped_text("[role=option]"), Vec::<String>::new());
}

#[test]
fn ctrl_k_opens_the_palette() {
    let mut page = mount(app);
    page.focus("#page");
    page.press_with(Key::Character("k".into()), Modifiers::CONTROL);
    assert!(
        page.exists(PALETTE),
        "Ctrl+K did not open it:\n{}",
        page.tree()
    );
}
