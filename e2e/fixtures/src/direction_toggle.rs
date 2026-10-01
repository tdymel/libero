//! `DirectionToggle` beside text, so a turn has something to reorder, and a
//! button that clears the choice.

use dioxus::prelude::*;
use libero::components::{Button, DirectionToggle, Flex, Text};
use libero::hooks::use_direction;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/direction-toggle", || rsx! { DirectionTogglePage {} }),
    (
        "/direction-toggle/shielded",
        || rsx! { DirectionToggleShieldedPage {} },
    ),
];

/// Test pages share one browser profile, so a stored `rtl` would leak into later fixtures.
const SHIELD_STORAGE: &str = r#"const items = new Map();
Object.defineProperty(window, 'localStorage', { configurable: true, value: {
    getItem: k => items.has(k) ? items.get(k) : null,
    setItem: (k, v) => { items.set(k, String(v)); },
    removeItem: k => { items.delete(k); },
}});
return true;"#;

/// The same page behind a stand-in `localStorage`, for the scenarios every backend runs
/// (1496). `#shielded` appears once the script ran, or failed where there is none.
#[component]
fn DirectionToggleShieldedPage() -> Element {
    let shield = use_resource(|| async { document::eval(SHIELD_STORAGE).await.is_ok() });
    rsx! {
        DirectionTogglePage {}
        if shield.read().is_some() {
            Text { id: "shielded", "shielded" }
        }
    }
}

#[component]
fn DirectionTogglePage() -> Element {
    let direction = use_direction();
    let kept = direction.kept().map_or("none", |kept| kept.as_str());
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            DirectionToggle { id: "direction" }
            Text { id: "page-text", "Page text" }
            Button { id: "clear", onclick: move |_| direction.clear(), "Clear" }
            Text { id: "kept", "{kept}" }
        }
    }
}
