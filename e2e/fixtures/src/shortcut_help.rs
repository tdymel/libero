//! `ShortcutHelp` inline, in a narrow and a wide column, and one with a long keymap.

use dioxus::prelude::*;
use libero::components::{Shortcut, ShortcutHelp};

use crate::Routes;

pub const ROUTES: Routes = &[("/shortcut-help", || rsx! { Widths {} })];

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
