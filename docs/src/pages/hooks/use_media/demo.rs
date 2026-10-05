use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    hooks::use_media,
};

#[component]
pub fn OwnPlayer() -> Element {
    let media = use_media();
    let time = media.current_time() as u64;
    let status = match (media.error(), media.buffering()) {
        (Some(_), _) => "The audio could not be played.",
        (None, true) => "Loading",
        (None, false) => "",
    };
    rsx! {
        audio { src: crate::samples::SAMPLE_AUDIO, onmounted: media.mount(), ..media.attributes() }
        Flex { gap: "sm", align: "center",
            Button {
                onclick: move |_| media.toggle(),
                if media.paused() { "Play" } else { "Pause" }
            }
            Button { variant: "outlined", onclick: move |_| media.seek(0.0), "Restart" }
            Text { "{time / 60}:{time % 60:02}" }
            div { role: "status", "{status}" }
        }
    }
}
