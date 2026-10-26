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
    assert_eq!(markers, ["1. ", "4. ", "3. ", "1. ", "8. "]);
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
