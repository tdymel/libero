//! `use_hotkeys`, as its docs page uses it: `mod+k` skipped in text entry, `ctrl+j` heard there.

use dioxus::prelude::*;
use libero::hooks::{Hotkey, use_hotkeys};

use crate::Routes;

pub const ROUTES: Routes = &[("/use-hotkeys", || rsx! { Hotkeys {} })];

#[component]
fn Listener(mut open: Signal<u32>, mut typing: Signal<u32>) -> Element {
    use_hotkeys([
        Hotkey::new("mod+k", move || open += 1),
        Hotkey::new("ctrl+j", move || typing += 1).include_editable(true),
    ]);
    rsx! {}
}

#[component]
fn Sibling(mut count: Signal<u32>) -> Element {
    use_hotkeys([Hotkey::new("mod+k", move || count += 1)]);
    rsx! {}
}

#[component]
fn Hotkeys() -> Element {
    let open = use_signal(|| 0);
    let typing = use_signal(|| 0);
    let other = use_signal(|| 0);
    let mut listening = use_signal(|| true);

    rsx! {
        Sibling { count: other }
        p { id: "other", "{other}" }
        button { id: "plain", "Plain" }
        input { id: "field" }
        button { id: "remove", onclick: move |_| listening.set(false), "Remove" }
        if listening() {
            Listener { open, typing }
        }
        p { id: "open", "{open}" }
        p { id: "typing", "{typing}" }
    }
}
