//! The focus ring's `--lsx-focus-contrast` fallback.

use dioxus::prelude::*;
use libero::components::{Button, Flex};

use crate::Routes;

pub const ROUTES: Routes = &[("/focus-contrast", || rsx! { FocusContrastPage {} })];

/// Todo 53, part one: what a `--lsx-focus-contrast` naming an undeclared
/// referent does to the ring.
///
/// The ring resolves as `var(--lsx-focus-contrast, var(--lsx-color-primary-6))`.
/// A CSS fallback applies only when the custom property is **not set at all**;
/// a property that *is* set to an invalid value is "invalid at computed-value
/// time", which is a different rule. So the question is whether the ring falls
/// back to primary or disappears - and that cannot be read off the emitted CSS,
/// only measured in a browser.
#[component]
fn FocusContrastPage() -> Element {
    rsx! {
        document::Style {
            ":root {{ --lsx-focus-contrast: var(--nothing-declares-this); }}"
        }
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button { id: "ring-probe", "Probe" }
        }
    }
}
