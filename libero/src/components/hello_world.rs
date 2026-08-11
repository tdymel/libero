use dioxus::prelude::*;

use crate::{context::use_sx, sx::sx};

const HELLO_WORLD_SX: crate::sx::Sx = sx()
    .background("primary")
    .padding_top("md")
    .width("200px")
    .build();

pub fn HelloWorld() -> Element {
    use_sx(&HELLO_WORLD_SX);

    rsx! {
        div {
            class: HELLO_WORLD_SX.class_name(),
            "Hello World!"
        }
    }
}
