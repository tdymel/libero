//! `use_long_press` on a button that also counts its clicks.

use dioxus::prelude::*;
use libero::hooks::{LongPressOptions, use_long_press};

use crate::Routes;

pub const ROUTES: Routes = &[("/use-long-press/basic", || rsx! { Pressable {} })];

#[component]
fn Pressable() -> Element {
    let mut taps = use_signal(|| 0);
    let mut holds = use_signal(|| 0);
    let press = use_long_press(
        Callback::new(move |()| holds += 1),
        LongPressOptions::default(),
    );
    rsx! {
        button {
            id: "target",
            "data-pressing": "{press.pressing}",
            style: "width: 160px; height: 80px; user-select: none; -webkit-touch-callout: none;",
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
            "Hold"
        }
        p { id: "holds", "{holds}" }
        p { id: "taps", "{taps}" }
    }
}
