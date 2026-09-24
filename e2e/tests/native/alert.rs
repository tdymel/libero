//! `Alert`'s `parts` on Blitz: the part selectors match natively, the instance `sx`
//! wins a tie, and a nested `Alert` is not reached.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Alert, AlertPart, Part, Parts};
use libero::sx::sx;

const TITLE: &str = "#styled > [data-slot=body] > [data-slot=title]";

fn app() -> Element {
    rsx! {
        Alert { id: "styled",
            title: "Styled parts",
            parts: Parts::new()
                .part(AlertPart::Title, sx().font_style("italic").color("#c80000")),
            sx: sx().selector(AlertPart::Title.selector(), sx().color("#0000c8")),
            "Parts styled."
            Alert { id: "nested", title: "Nested", "Not reached." }
        }
    }
}

#[test]
fn parts_style_the_inner_parts_natively() {
    let page = mount(app);
    assert_eq!(page.computed(TITLE, "font-style"), "italic");
    assert_eq!(
        page.computed(TITLE, "color"),
        "rgb(0, 0, 200)",
        "the instance sx should win the tie"
    );
    assert_eq!(
        page.computed("#nested [data-slot=title]", "font-style"),
        "normal"
    );
}
