//! `Checkbox`'s painted mark. Clicks and keys are shared scenarios (`checkbox::`,
//! `radio_group::`); links in labels are in `pointer.rs`.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::Checkbox;

const CHECKBOX: &str = "input[type=checkbox]";

/// Blitz's svg parser reads only `currentColor` as the current colour; the
/// lowercase keyword left the checked box without its mark.
#[test]
fn a_checked_box_paints_its_mark() {
    fn app() -> Element {
        rsx! {
            Checkbox { checked: true, onchange: |_| {}, label: "Remember me" }
        }
    }
    let page = mount(app);
    let mark = format!("{CHECKBOX} + [aria-hidden=true]");
    assert_eq!(
        page.painted_stroke(&format!("{mark} > svg")),
        page.computed(&mark, "color")
    );
}

/// Todo 5422: an `aria_label`-only box drops the label column's gap, so it is box-wide and
/// centred in its cell; a labelled box keeps the gap.
#[test]
fn an_aria_label_only_checkbox_is_box_wide_and_centred() {
    fn app() -> Element {
        rsx! {
            div { id: "cell", style: "display: flex; justify-content: center; width: 64px",
                Checkbox { id: "bare", aria_label: "Select this row" }
            }
            div { id: "labelled-cell", style: "display: flex; justify-content: center; width: 200px",
                Checkbox { id: "named", label: "Terms" }
            }
        }
    }
    let page = mount(app);
    let (cx, _, cw, _) = page.rect("#cell");
    let (bx, _, bw, _) = page.rect("#bare + [aria-hidden=true]");
    assert_eq!(bw, 18.0, "the box width");
    assert!(
        ((bx + bw / 2.0) - (cx + cw / 2.0)).abs() <= 0.5,
        "the box sits off the cell centre"
    );
    let (_, _, root_w, _) = page.rect("#cell > [data-state~='solo']");
    assert!(
        (root_w - bw).abs() <= 0.5,
        "the root is {root_w}px wide for an {bw}px box"
    );
    let (_, _, named_root_w, _) = page.rect("#labelled-cell > [data-state~='inline']");
    let (_, _, label_w, _) = page.rect("#labelled-cell label");
    assert!(
        named_root_w >= bw + label_w + 8.0 - 0.5,
        "the labelled root lost its gap: {named_root_w}px for {bw} + {label_w}"
    );
}
