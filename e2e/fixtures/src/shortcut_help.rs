//! `ShortcutHelp` inline, in a narrow and a wide column, and one with a long keymap.

use dioxus::prelude::*;
use libero::components::{Shortcut, ShortcutHelp};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/shortcut-help", || rsx! { Widths {} }),
    ("/shortcut-help-grow", || rsx! { Growing {} }),
];

/// A short list that a button swaps for a long one, in a box of a fixed height: no resize fires.
#[component]
fn Growing() -> Element {
    let mut long = use_signal(|| false);
    let shortcuts = if long() {
        (1..=40)
            .map(|n| Shortcut::new(format!("mod+alt+{}", n % 10), format!("Command {n}")))
            .collect()
    } else {
        shortcuts()
    };
    rsx! {
        style { "#growing div:has(> dl) {{ height: 6rem; }}" }
        button { id: "grow", onclick: move |_| long.set(true), "Grow" }
        div { id: "growing", style: "width: 600px",
            ShortcutHelp { title: "Growing", shortcuts }
        }
    }
}

fn shortcuts() -> Vec<Shortcut> {
    vec![
        Shortcut::new("mod+b", "Bold"),
        Shortcut::new("shift+mod+k", "Insert a link"),
    ]
}

#[component]
fn Widths() -> Element {
    rsx! {
        div { id: "narrow", style: "width: 300px",
            ShortcutHelp { title: "Narrow", shortcuts: shortcuts() }
        }
        div { id: "wide", style: "width: 600px",
            ShortcutHelp { title: "Wide", shortcuts: shortcuts() }
        }
        div { id: "long", style: "width: 600px",
            ShortcutHelp {
                title: "Long",
                shortcuts: (1..=40).map(|n| Shortcut::new(format!("mod+alt+{}", n % 10), format!("Command {n}"))).collect(),
            }
        }
    }
}
