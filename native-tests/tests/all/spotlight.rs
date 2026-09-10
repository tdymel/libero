//! `Spotlight`'s hotkey: Ctrl+K opens the palette from wherever focus is.

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

#[test]
#[ignore = "Blitz: no document key notification, so no hotkey can be heard"]
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
