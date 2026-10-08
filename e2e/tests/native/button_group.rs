//! `ButtonGroup` on Blitz: `Copy`'s `display: contents` wrapper and status span (todos 2636, 2318)
//! and the vertical stretch selector reaching a tooltip icon's wrapper (todo 2330).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{ActionIcon, Button, ButtonGroup, Copy};

const STATUS: &str = "[role=status]";

fn app() -> Element {
    rsx! {
        ButtonGroup { id: "copying", variant: "outlined", "aria-label": "Copying",
            Button { id: "c1", "Install" }
            Copy { id: "c2", value: "cargo add libero", aria_label: "Copy the command" }
        }
        ButtonGroup {
            id: "vertical",
            variant: "outlined",
            orientation: "vertical",
            "aria-label": "Vertical",
            Button { id: "vm1", "A wider label" }
            ActionIcon { id: "vm2", aria_label: "Two", "2" }
            ActionIcon { id: "vm5", aria_label: "Five", tooltip: true, "5" }
        }
    }
}

fn square(corner: &str) -> bool {
    corner.split_whitespace().all(|part| part == "0px")
}

fn round(page: &e2e::native::Page, selector: &str, property: &str) -> bool {
    !square(&page.computed(selector, property))
}

#[test]
fn copy_ends_a_group_round() {
    let page = mount(app);
    assert!(
        round(&page, "#c2", "border-top-right-radius"),
        "Copy's end corner is square: {}",
        page.computed("#c2", "border-top-right-radius")
    );
    assert!(
        !round(&page, "#c1", "border-top-right-radius"),
        "the button before Copy rounds its end"
    );
    assert!(!round(&page, "#c2", "border-top-left-radius"));
}

#[test]
fn copy_sits_flush_against_the_button_before_it() {
    let page = mount(app);
    let (c1, c2) = (page.rect("#c1"), page.rect("#c2"));
    let seam = c2.0 - (c1.0 + c1.2);
    assert!(
        (seam + 1.0).abs() < 0.5,
        "not flush: {c1:?} {c2:?} (seam {seam})"
    );
    assert!((c1.3 - c2.3).abs() < 0.5, "heights differ: {c1:?} {c2:?}");
}

#[test]
fn a_copy_press_keeps_the_group_shape() {
    let mut page = mount(app);
    page.click("#c2");
    let told = page.wait_for(|page| !page.text(STATUS).trim().is_empty());
    assert!(told, "no status after the press:\n{}", page.tree());
    assert!(round(&page, "#c2", "border-top-right-radius"));
    assert!(!round(&page, "#c1", "border-top-right-radius"));
}

#[test]
fn a_vertical_group_stretches_a_tooltip_icon() {
    let page = mount(app);
    let widest = page.rect("#vm1").2;
    for item in ["#vm2", "#vm5"] {
        let width = page.rect(item).2;
        assert!(
            (width - widest).abs() < 0.5,
            "{item} is {width}px, not {widest}px"
        );
    }
}
