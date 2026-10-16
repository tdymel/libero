//! Blitz re-parses the `style` attribute when it is written (`flush_style_attribute`), which
//! wipes the style attributes (`set_style_property`) applied before it (todo 2804).

use dioxus::prelude::*;
use e2e::native::mount;

/// `height` sorts before `style` in the slot, `width` after it.
fn app() -> Element {
    rsx! {
        div { id: "box", height: "40px", style: "color: red", width: "20px" }
    }
}

fn plain_app() -> Element {
    rsx! {
        div { id: "box", height: "40px", width: "20px" }
    }
}

#[test]
fn style_attributes_alone_size_a_box() {
    let page = mount(plain_app);
    let (_, _, width, height) = page.rect("#box");
    assert_eq!((width, height), (20.0, 40.0));
}

#[test]
#[ignore = "needs Blitz: a style write wipes the style attributes applied before it"]
fn a_style_string_keeps_the_style_attributes_before_it() {
    let page = mount(app);
    let (_, _, width, height) = page.rect("#box");
    assert_eq!((width, height), (20.0, 40.0));
}
