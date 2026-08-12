use dioxus::prelude::*;

use crate::{SxLayer, context::use_sx, sx::sx};

const HELLO_WORLD_SX: crate::sx::Sx = sx().background("primary").padding_top("md").width("200px");

pub fn HelloWorld() -> Element {
    let class = use_sx(&HELLO_WORLD_SX, SxLayer::UserStatic);

    rsx! {
        div {
            class: class,
            "Hello World!"
        }
    }
}
