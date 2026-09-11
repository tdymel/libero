//! `Splitter`: a drag moves the divider and leaves it focused for the arrow
//! keys (todo 431).

use dioxus::prelude::*;
use libero::components::{Button, Splitter, Text};
use native_tests::{Key, mount};

const DIVIDER: &str = "[role=separator]";

fn app() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        div { width: "320px", height: "160px",
            Splitter {
                initial_size: 50.0,
                aria_label: "Resize panes",
                panel_a: rsx! { Text { "Pane A" } },
                panel_b: rsx! { Text { "Pane B" } },
            }
        }
    }
}

fn value(page: &native_tests::Page) -> f64 {
    page.attr(DIVIDER, "aria-valuenow")
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("no aria-valuenow:\n{}", page.tree()))
}

#[test]
fn a_drag_moves_the_divider_and_leaves_it_focused() {
    let mut page = mount(app);
    page.focus("#before");
    let start = value(&page);
    page.drag(DIVIDER, 40.0, 0.0);
    let dragged = value(&page);
    assert!(
        dragged > start + 5.0,
        "the drag moved it {start} -> {dragged}"
    );
    assert!(
        page.is_focused(DIVIDER),
        "focus is on {}",
        page.focus_owner()
    );

    page.press(Key::ArrowRight);
    assert!(value(&page) > dragged, "ArrowRight did not move it");
}

#[test]
fn the_arrows_and_home_end_move_a_focused_divider() {
    let mut page = mount(app);
    page.focus(DIVIDER);
    let start = value(&page);
    page.press(Key::ArrowLeft);
    assert!(value(&page) < start, "ArrowLeft did not move it");
    page.press(Key::Home);
    let min = value(&page);
    page.press(Key::End);
    assert!(value(&page) > min, "End did not move it past Home");
}
