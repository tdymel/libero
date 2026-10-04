use dioxus::prelude::*;
use libero::{
    components::{Box, Text},
    sx::sx,
};

pub fn demo_slides() -> Vec<Element> {
    (1..=6)
        .map(|n| {
            rsx! {
                Box {
                    sx: sx()
                        .display("flex")
                        .align_items("center")
                        .justify_content("center")
                        // A floor, not the size: as `height`, a vertical 300px slide painted only 160px.
                        .min_height("160px")
                        .height("100%")
                        .background(format!("primary.{n}"))
                        .color(format!("primary-contrast.{n}")),
                    Text { "Slide {n}" }
                }
            }
        })
        .collect()
}
