//! `Center` keeps its child centred natively, with `safe` alignment, and spills an oversized one toward the end.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::{
    components::{Box as LBox, Center},
    sx::sx,
};

fn app() -> Element {
    rsx! {
        Center { id: "fits", sx: sx().width("300px").height("100px"),
            LBox { id: "small", sx: sx().width("100px").height("20px") }
        }
        Center { id: "overflows", sx: sx().width("100px").height("40px"),
            LBox { id: "big", sx: sx().width("200px").height("80px").flex_shrink("0") }
        }
    }
}

#[test]
fn a_fitting_child_is_centred_on_both_axes() {
    let page = mount(app);
    let (x, y, _, _) = page.rect("#fits");
    let (cx, cy, _, _) = page.rect("#small");
    assert_eq!((cx - x, cy - y), (100.0, 40.0));
}

#[test]
fn an_oversized_child_starts_at_the_start_edges() {
    let page = mount(app);
    let (x, y, _, _) = page.rect("#overflows");
    let (cx, cy, _, _) = page.rect("#big");
    assert_eq!((cx - x, cy - y), (0.0, 0.0));
}
