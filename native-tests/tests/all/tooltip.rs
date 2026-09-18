//! `Tooltip`: a portaled bubble opened by hover and keyboard focus, placed on
//! its side of the trigger, closed by leaving and by Escape (SC 1.4.13).

use std::time::Duration;

use dioxus::prelude::*;
use libero::components::{Button, Tooltip};
use native_tests::{Key, Page, mount};

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

fn hover_open() -> Page {
    let mut page = mount(app);
    assert!(!page.exists(OPEN), "open at rest");
    page.hover(TRIGGER);
    page.wait(Duration::from_millis(100));
    assert!(
        page.exists(OPEN),
        "hovering did not open it:\n{}",
        page.tree()
    );
    page
}

#[test]
fn hover_opens_it_below_the_trigger_and_leaving_closes_it() {
    let mut page = hover_open();
    let (tx, ty, tw, th) = page.rect(TRIGGER);
    let (bx, by, bw, _) = page.rect(OPEN);
    let gap = by - (ty + th);
    assert!(
        (0.0..=16.0).contains(&gap) && ((bx + bw / 2.0) - (tx + tw / 2.0)).abs() <= 2.0,
        "trigger at ({tx}, {ty}) {tw}x{th}, bubble at ({bx}, {by}) w {bw}"
    );

    page.hover("#away");
    page.wait(Duration::from_millis(100));
    assert!(!page.exists(OPEN), "leaving did not close it");
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

#[test]
fn the_pointer_can_rest_on_the_bubble() {
    let mut page = hover_open();
    assert!(page.hits(OPEN), "the bubble takes no hit");
    page.hover(OPEN);
    page.wait(Duration::from_millis(100));
    assert!(page.exists(OPEN), "moving onto the bubble closed it");
}

#[test]
fn escape_closes_it_under_the_pointer() {
    let mut page = hover_open();
    page.focus("#before");
    page.press(Key::Escape);
    assert!(!page.exists(OPEN), "Escape did not close it");
    assert!(
        page.is_focused("#before"),
        "focus moved to {}",
        page.focus_owner()
    );
}

/// Blitz fires no `focusin` for Tab; the silent-focus check opens it (N6).
#[test]
fn tab_focus_opens_it() {
    let mut page = mount(app);
    page.focus("#before");
    page.tab();
    assert!(
        page.is_focused(TRIGGER),
        "Tab went to {}",
        page.focus_owner()
    );
    assert!(page.exists(OPEN), "Tab focus did not open it");
}
