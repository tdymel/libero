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
    let mut width = use_signal(|| 240);

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Box {
                onmounted: panel.mount(),
                sx: sx().height("96px").padding("md").max_width("100%")
                    .background("muted.1").border_radius("8px"),
                // A drag on the corner resizes it too, where the browser offers one.
                style: "width: {width}px; resize: both; overflow: auto",
                "Resize me."
            }
            Flex { direction: "row", gap: "sm", wrap: "wrap",
                Button { variant: "outlined", onclick: move |_| width.set(160), "Narrow" }
                Button { variant: "outlined", onclick: move |_| width.set(240), "Medium" }
                Button { variant: "outlined", onclick: move |_| width.set(320), "Wide" }
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
