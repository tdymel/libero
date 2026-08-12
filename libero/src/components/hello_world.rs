use dioxus::prelude::*;

use crate::{SxLayer, context::use_sx, sx::sx};

pub fn HelloWorld() -> Element {
    let sx = sx().background("primary").padding_top("md").width("200px");
    let class = use_sx(&sx, SxLayer::UserStatic);

    rsx! {
        div {
            class: class,
            "Hello World!"
        }
    }
}
