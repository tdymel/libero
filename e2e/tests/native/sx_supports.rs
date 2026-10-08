//! `Sx::supports` under Blitz (todo 1161): the `@supports` block is parsed and judged by stylo.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::{components::Box, sx::sx};

fn app() -> Element {
    rsx! {
        Box {
            id: "known",
            sx: sx().width("10px").supports("(width: 1px)", sx().width("20px")),
        }
        Box {
            id: "unknown",
            sx: sx().width("10px").supports("(no-such-property: 1px)", sx().width("20px")),
        }
        Box {
            id: "not",
            sx: sx().width("10px").supports("not (no-such-property: 1px)", sx().width("20px")),
        }
    }
}

#[test]
fn a_supports_block_applies_only_when_stylo_knows_the_condition() {
    let page = mount(app);
    assert_eq!(page.computed("#known", "width"), "20px");
    assert_eq!(page.computed("#unknown", "width"), "10px");
    assert_eq!(page.computed("#not", "width"), "20px");
}
