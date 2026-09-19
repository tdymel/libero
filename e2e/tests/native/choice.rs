//! `Checkbox`'s painted mark. The box click, Space and the radio group's keys
//! are e2e's shared scenarios (`checkbox::`, `radio_group::`); links in labels
//! are in `pointer.rs`.

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
