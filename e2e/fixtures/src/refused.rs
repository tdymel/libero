//! Presses and Tab stops the web refuses: disabled buttons and an `inert` subtree.

use dioxus::prelude::*;
use libero::components::Button;

use crate::Routes;

pub const ROUTES: Routes = &[("/refused", || rsx! { RefusedPage {} })];

#[component]
fn RefusedPage() -> Element {
    let mut clicks = use_signal(|| 0);
    rsx! {
        Button { id: "first", "First" }
        Button { id: "off", disabled: true, onclick: move |_| clicks += 1,
            span { id: "off-label", "Off" }
        }
        div { id: "card", tabindex: "-1", padding: "8px",
            Button { id: "off-in-card", disabled: true, onclick: move |_| clicks += 1, "Off in a card" }
        }
        div { id: "asleep", inert: true,
            Button { id: "asleep-button", onclick: move |_| clicks += 1, "Asleep" }
        }
        Button { id: "last", "Last" }
        span { id: "clicks", "{clicks}" }
    }
}
