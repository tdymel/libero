use dioxus::prelude::*;
use libero::{
    components::{Button, Flex},
    hooks::{LongPressOptions, use_long_press},
};

#[component]
pub fn HoldToCount() -> Element {
    let mut taps = use_signal(|| 0);
    let mut holds = use_signal(|| 0);
    let press = use_long_press(
        Callback::new(move |()| holds += 1),
        LongPressOptions::default(),
    );

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Button {
                onpointerdown: move |event| press.onpointerdown.call(event),
                onpointermove: move |event| press.onpointermove.call(event),
                onpointerup: move |event| press.onpointerup.call(event),
                onpointerleave: move |event| press.onpointerleave.call(event),
                onpointercancel: move |event| press.onpointercancel.call(event),
                oncontextmenu: move |event| press.oncontextmenu.call(event),
                onclick: move |event| {
                    if !press.onclick.call(event) {
                        taps += 1;
                    }
                },
                if (press.pressing)() {
                    "Keep holding"
                } else {
                    "Tap, or hold"
                }
            }
            // The same action without holding (WCAG 2.5.1, 2.1.1).
            Button { variant: "outlined", onclick: move |_| holds += 1, "Count a hold" }
            div { role: "status", "{taps} taps, {holds} holds" }
        }
    }
}
