//! `use_timeout`, `use_interval` and the debounce hooks, as their docs pages use them.

use dioxus::prelude::*;
use libero::hooks::{
    use_debounced_callback, use_debounced_value, use_interval, use_throttled_value, use_timeout,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    (
        "/use-timers/quick",
        || rsx! { Typing { debounce: 120, save: 120, throttle: 120 } },
    ),
    // Delays nothing else on the page schedules, so the held clock takes each timer alone.
    (
        "/use-timers/held",
        || rsx! { Typing { debounce: DEBOUNCE_MS, save: SAVE_MS, throttle: THROTTLE_MS } },
    ),
    ("/use-timers/interval", || rsx! { Timers {} }),
];

pub const DEBOUNCE_MS: u64 = 717;
pub const SAVE_MS: u64 = 727;
pub const THROTTLE_MS: u64 = 737;

#[component]
fn Typing(debounce: u64, save: u64, throttle: u64) -> Element {
    let mut typed = use_signal(String::new);
    let mut saved = use_signal(String::new);
    let settled = use_debounced_value(typed.into(), debounce);
    let throttled = use_throttled_value(typed.into(), throttle);
    let save_later = use_debounced_callback(move |text: String| saved.set(text), save);

    rsx! {
        input {
            id: "field",
            oninput: move |event| {
                typed.set(event.value());
                save_later.call(event.value());
            },
        }
        p { id: "typed", "{typed}" }
        p { id: "settled", "{settled}" }
        p { id: "throttled", "{throttled}" }
        p { id: "saved", "{saved}" }
    }
}

#[component]
fn Timers() -> Element {
    let mut ticks = use_signal(|| 0);
    let mut flashing = use_signal(|| false);
    let interval = use_interval(move || ticks += 1, 100);
    let flash = use_timeout(move || flashing.set(false), 150);

    rsx! {
        button { id: "toggle", onclick: move |_| interval.toggle(),
            if interval.active() {
                "Stop"
            } else {
                "Start"
            }
        }
        button {
            id: "flash",
            onclick: move |_| {
                flashing.set(true);
                flash.start();
            },
            "Flash"
        }
        p { id: "ticks", "{ticks}" }
        p { id: "flashing", if flashing() { "on" } else { "off" } }
    }
}
