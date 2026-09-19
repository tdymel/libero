//! `Tooltip` under Blitz's hit-testing and paint: the bubble and its bridge
//! take presses, a padded block's trigger hovers, a stacking scroller clips.
//! Hover, placement, Escape and Tab focus are e2e's shared scenarios.

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Button, Tooltip};

const TRIGGER: &str = "#save";
const OPEN: &str = "#save-tip:not([hidden])";

fn app() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        // Off the left edge, so the bubble can centre without clamping. A
        // margin: an inline box in a padded block takes no hits (`hit.rs`).
        div { height: "40px" }
        div { margin_left: "200px",
            Tooltip {
                label: rsx! { "Saves the draft" },
                label_id: "save-tip",
                side: "bottom",
                open_delay: 10,
                close_delay: 10,
                Button { id: "save", "aria-describedby": "save-tip", "Save" }
            }
        }
        div { height: "200px" }
        p { id: "away", "Away" }
    }
}

/// Todo 787: parley clamps a hit below a box's last line to that line, so a
/// bubble's text answers over its overflow. Only the bridge reaches past it,
/// and the web's bridge takes those presses too: the trigger and the page
/// beside and below the bubble keep theirs.
#[test]
fn the_bubble_takes_presses_only_over_itself_and_its_bridge() {
    fn above() -> Element {
        rsx! {
            div { height: "120px" }
            div { margin_left: "200px",
                Tooltip {
                    label: rsx! { "Saves the draft" },
                    label_id: "save-tip",
                    side: "top",
                    open_delay: 10,
                    Button { id: "save", "Save" }
                }
            }
        }
    }
    for app in [app as fn() -> Element, above] {
        let mut page = mount(app);
        page.hover(TRIGGER);
        page.wait(Duration::from_millis(100));
        let (tx, ty, tw, th) = page.rect(TRIGGER);
        let (bx, by, bw, bh) = page.rect(OPEN);
        let (tx, ty, tw, th) = (tx as f32, ty as f32, tw as f32, th as f32);
        let (bx, by, bw, bh) = (bx as f32, by as f32, bw as f32, bh as f32);
        let below = by > ty;
        let gap = if below {
            (ty + th + by) / 2.0
        } else {
            (by + bh + ty) / 2.0
        };
        let (on_trigger, past) = if below {
            (ty + th - 2.0, by + bh + 4.0)
        } else {
            (ty + 2.0, by - 4.0)
        };
        assert!(page.hits_at(OPEN, bx + bw / 2.0, gap), "the bridge");
        assert!(
            page.hits_at(TRIGGER, tx + tw / 2.0, on_trigger),
            "the trigger"
        );
        assert!(!page.hits_at(OPEN, bx + bw / 2.0, past), "past the bubble");
        assert!(!page.hits_at(OPEN, bx - 4.0, by + bh / 2.0), "beside");
        assert!(!page.hits_at(OPEN, bx - 4.0, gap), "beside the bridge");
    }
}

/// Blitz hit-tests no inline box in a padded block unless it is z-indexed, so
/// the wrapper is natively (todo 889).
#[test]
fn hover_opens_it_inside_a_padded_block() {
    fn padded() -> Element {
        rsx! {
            div { padding: "40px",
                "Text before "
                Tooltip {
                    label: rsx! { "Saves the draft" },
                    label_id: "save-tip",
                    open_delay: 10,
                    Button { id: "save", "Save" }
                }
            }
        }
    }
    let mut page = mount(padded);
    page.hover(TRIGGER);
    page.wait(Duration::from_millis(100));
    assert!(page.exists(OPEN), "hovering did not open it");
}

/// That `z-index` paints past a plain scroller's clip; a stacking-context
/// scroller still clips the trigger (todo 881, the docs Platform page).
#[test]
fn a_stacking_scroller_clips_the_trigger() {
    fn scrolled() -> Element {
        rsx! {
            div { style: "height: 80px; width: 200px; overflow-y: auto; position: relative; z-index: 0;",
                div { style: "height: 120px;" }
                Tooltip { label: rsx! { "Saves the draft" }, Button { id: "save", "Save" } }
            }
        }
    }
    let page = mount(scrolled);
    let (x, y, w, h) = page.rect(TRIGGER);
    assert!(y > 80.0, "the trigger is inside the frame at {y}");
    let centre = ((x + w / 2.0) as u32, (y + h / 2.0) as u32);
    assert_eq!(page.painted_pixel(centre.0, centre.1), "rgb(255, 255, 255)");
}
