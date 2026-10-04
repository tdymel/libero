use dioxus::prelude::*;
use libero::{
    components::{Button, Text},
    hooks::{use_element, use_fullscreen},
};

#[component]
pub fn Panel() -> Element {
    let panel = use_element();
    let fullscreen = use_fullscreen(panel);
    let drawn = if fullscreen.is_drawn() {
        "position: fixed; inset: 0; z-index: 1000;"
    } else {
        ""
    };
    rsx! {
        div {
            onmounted: panel.mount(),
            style: "{drawn} display: flex; flex-direction: column; gap: 8px; padding: 16px; background: Canvas; color: CanvasText; border: 1px solid GrayText; border-radius: 8px;",
            ..fullscreen.attributes(),
            Text { "A chart, a map or a slide deck." }
            Button {
                variant: "outlined",
                onclick: move |_| fullscreen.toggle(),
                if fullscreen.is_fullscreen() { "Exit fullscreen" } else { "Fullscreen" }
            }
        }
    }
}
