use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Flex},
    hooks::use_element,
    platform::ElementApi,
    sx::sx,
};

#[component]
pub fn Measure() -> Element {
    let panel = use_element();
    let mut size = use_signal(String::new);

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Box {
                onmounted: panel.mount(),
                sx: sx().width("240px").height("96px").padding("md")
                    .background("muted.1").border_radius("8px"),
                style: "resize: both; overflow: auto",
                "Drag the corner to resize me."
            }
            Button {
                variant: "outlined",
                onclick: move |_| {
                    let read = panel.dimensions();
                    spawn(async move {
                        if let Ok(box_size) = read.await {
                            size.set(format!("{:.0} × {:.0} px", box_size.width, box_size.height));
                        }
                    });
                },
                "Measure"
            }
            span { role: "status", "{size}" }
        }
    }
}
