//! `ListItem`'s floated icon natively (todo 2980): beside the body at the start edge,
//! flipped under `dir=rtl`, and a long word wraps beside it.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{List, ListItem};

fn app() -> Element {
    rsx! {
        for dir in ["ltr", "rtl"] {
            div { dir, style: "width: 160px",
                List {
                    ListItem { id: "{dir}", icon: rsx! { "*" }, "Supercalifragilisticexpialidocious" }
                }
            }
        }
    }
}

fn ordered() -> Element {
    rsx! {
        List { ordered: true,
            ListItem { id: "first", "Install" }
            ListItem { id: "iconed", icon: rsx! { "*" }, "Check" }
            ListItem {
                id: "acted",
                action: rsx! { button { "Edit" } },
                "Configure"
            }
            ListItem { id: "last", "Deploy" }
        }
        List { ordered: true, "reversed": true,
            ListItem { id: "down-first", "Install" }
            ListItem { icon: rsx! { "*" }, "Check" }
            ListItem { id: "down-last", "Deploy" }
        }
        List { ordered: true, "start": 7,
            ListItem { icon: rsx! { "*" }, "Check" }
            ListItem { id: "started", "Deploy" }
        }
    }
}

/// Todo 4213: an icon or an action item counts in the numbering, as on the web, also
/// counting down or from `start`.
#[test]
fn an_icon_or_action_item_counts_in_an_ordered_list() {
    let page = mount(ordered);
    let markers = ["#first", "#last", "#down-first", "#down-last", "#started"]
        .map(|selector| page.list_marker(selector).unwrap_or_default());
    assert_eq!(
        markers,
        ["1.\u{a0}", "4.\u{a0}", "3.\u{a0}", "1.\u{a0}", "8.\u{a0}"]
    );
}

/// Todo 4559: a flex action item has no marker, so it draws its number as a leading span.
#[test]
fn an_action_item_draws_its_own_number() {
    let page = mount(ordered);
    assert_eq!(
        page.laid_out_text("#acted > [data-slot=marker]"),
        "3.\u{a0}"
    );
    assert!(!page.exists("#iconed > [data-slot=marker]"));
    assert_eq!(
        page.computed("#first > [data-slot=marker]", "display"),
        "none"
    );
}

fn numbered() -> Element {
    rsx! {
        List { ordered: true,
            ListItem { id: "plain", span { id: "plain-text", "First" } }
            ListItem {
                id: "action",
                action: rsx! { button { "Edit" } },
                span { id: "action-text", "Second" }
            }
        }
    }
}

/// The number's glyphs (id, x) left of the item's text, and the text's left edge.
fn number_and_text(page: &Page, item: &str) -> (Vec<(u32, f64)>, f64) {
    let text = page.rect(&format!("#{item}-text")).0;
    let number = page
        .painted_glyphs(&format!("#{item}"))
        .into_iter()
        .filter(|(_, x)| *x < text - 0.5)
        .collect();
    (number, text)
}

/// Todo 5199: an action item's number and text line up with a plain item's, and a
/// plain number keeps its gap before the text ("1.First" lost it).
#[test]
fn every_number_and_text_share_one_column() {
    let page = mount(numbered);
    let (plain, plain_text) = number_and_text(&page, "plain");
    let (action, action_text) = number_and_text(&page, "action");
    let shown = format!("plain {plain:?} at {plain_text}, action {action:?} at {action_text}");
    // The '.' is the glyph both numbers share.
    let dot = |number: &[(u32, f64)], other: &[(u32, f64)]| {
        number
            .iter()
            .find(|(id, _)| other.iter().any(|(other, _)| other == id))
            .map(|(_, x)| *x)
    };
    let (Some(plain_dot), Some(action_dot)) = (dot(&plain, &action), dot(&action, &plain)) else {
        panic!("no '.' in both numbers: {shown}");
    };
    assert!(
        (plain_dot - action_dot).abs() < 1.0 && (plain_text - action_text).abs() < 1.0,
        "{shown}"
    );
    // The space follows the '.'; its width is the font's (macOS's is 3px, not 4.5).
    for (number, dot) in [(&plain, plain_dot), (&action, action_dot)] {
        assert!(number.iter().any(|(_, x)| *x > dot), "{shown}");
    }
}

/// The icon's, the body's and the column's boxes as (left, top, right, bottom).
fn boxes(page: &Page, dir: &str) -> [(f64, f64, f64, f64); 3] {
    let edges = |selector: String| {
        let (x, y, width, height) = page.rect(&selector);
        (x, y, x + width, y + height)
    };
    [
        edges(format!("#{dir} [data-slot=icon]")),
        edges(format!("#{dir} [data-slot=body]")),
        edges(format!("#{dir}")),
    ]
}

#[test]
fn the_icon_sits_beside_the_body_at_the_start_edge() {
    let page = mount(app);
    let [icon, body, item] = boxes(&page, "ltr");
    assert!(
        icon.0 <= item.0 + 1.0 && body.0 >= icon.2 && body.1 < icon.3 && body.2 <= item.2 + 1.0,
        "ltr icon {icon:?} body {body:?} item {item:?}"
    );
    let [icon, body, item] = boxes(&page, "rtl");
    assert!(
        icon.2 >= item.2 - 1.0 && body.2 <= icon.0 && body.1 < icon.3 && body.0 >= item.0 - 1.0,
        "rtl icon {icon:?} body {body:?} item {item:?}"
    );
}

fn rtl_numbered() -> Element {
    rsx! {
        div { dir: "rtl", style: "width: 240px; padding: 0 48px",
            List { ordered: true,
                ListItem { id: "rtl-plain", "Install" }
                ListItem {
                    id: "rtl-acted",
                    action: rsx! { button { "Edit" } },
                    "Configure"
                }
            }
        }
    }
}

/// Todo 5395: under `dir=rtl` a plain item's number hangs in its inline-start (right) gutter,
/// as an action item's does, and is painted there; Blitz would paint the string marker at the left.
#[test]
fn a_plain_number_hangs_in_the_gutter_at_the_right_under_rtl() {
    let page = mount(rtl_numbered);
    for item in ["rtl-plain", "rtl-acted"] {
        let (x, _, width, _) = page.rect(&format!("#{item}"));
        let edge = x + width;
        let (marker_x, _, marker_width, _) = page.rect(&format!("#{item} > [data-slot=marker]"));
        assert!(
            marker_x >= edge - 1.0 && marker_x + marker_width <= edge + 33.0,
            "{item}: item right edge {edge}, marker {marker_x}+{marker_width}"
        );
        let glyphs = page.painted_glyphs(&format!("#{item}"));
        assert!(
            glyphs.iter().any(|(_, at)| *at >= edge - 1.0),
            "{item}: no glyph right of {edge}: {glyphs:?}"
        );
    }
}

/// The ltr plain item keeps its string marker and shows no span.
#[test]
fn a_plain_number_keeps_its_string_marker_under_ltr() {
    let page = mount(ordered);
    assert_eq!(page.list_marker("#first").as_deref(), Some("1.\u{a0}"));
    assert_eq!(
        page.computed("#first > [data-slot=marker]", "display"),
        "none"
    );
}
