# use_accessibility

Crate: `libero`
Import: `use libero::hooks::use_accessibility;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/accessibility.rs>
Index: [index.md](index.md) lists every other page
Description: Reads the reader's accessibility preferences and lets an app answer them over the system.

`use_accessibility() -> AccessibilityHandle` reads the reader's accessibility
preferences: reduced motion, forced colors, contrast and reduced transparency.
An app's settings page can answer them over the system with overrides.

The overrides act on native renderers only. In a browser the media queries
decide: `get()` returns what the browser answers, and `set_overrides` changes
nothing on screen. `LiberoProvider` takes the same overrides at mount as
`accessibility`.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Switch},
    hooks::use_accessibility,
    theme::{AccessibilityOverrides, Contrast},
};

#[component]
fn Settings() -> Element {
    let accessibility = use_accessibility();
    let now = accessibility.get();

    rsx! {
        Flex { direction: "column", gap: "sm",
            Switch {
                label: "Reduce motion",
                checked: now.reduced_motion,
                onchange: {
                    let accessibility = accessibility.clone();
                    move |on: bool| accessibility.set_overrides(AccessibilityOverrides {
                        reduced_motion: Some(on),
                        ..accessibility.overrides()
                    })
                },
            }
            Switch {
                label: "More contrast",
                checked: now.contrast == Contrast::More,
                onchange: {
                    let accessibility = accessibility.clone();
                    move |on: bool| accessibility.set_overrides(AccessibilityOverrides {
                        contrast: Some(if on { Contrast::More } else { Contrast::NoPreference }),
                        ..accessibility.overrides()
                    })
                },
            }
            Button {
                variant: "outlined",
                onclick: move |_| accessibility.set_overrides(AccessibilityOverrides::default()),
                "Follow the system"
            }
        }
    }
}
```

At mount:

```rust
use dioxus::prelude::*;
use libero::{LiberoProvider, theme::AccessibilityOverrides};

fn App() -> Element {
    rsx! {
        LiberoProvider {
            accessibility: AccessibilityOverrides { reduced_motion: Some(true), ..Default::default() },
            "..."
        }
    }
}
```

Natively the system's settings come from the desktop portal on Linux
(`org.freedesktop.appearance`, GNOME's own keys as a fallback). Other
platforms report no preference; the overrides still apply.

## API

```rust,ignore
pub fn use_accessibility() -> AccessibilityHandle
```

| Method | Returns | Description |
|---|---|---|
| `get()` | `AccessibilityPreferences` | What the page answers: the system with the overrides on top. On the web, the browser's answers. |
| `system()` | `AccessibilityPreferences` | The platform's settings, without the overrides. |
| `overrides()` | `AccessibilityOverrides` | The app's answers. |
| `set_overrides(overrides: AccessibilityOverrides)` | `()` | Replaces them; a `None` field follows the system again. Native only, for the session. |

`AccessibilityPreferences` has `reduced_motion`, `forced_colors`,
`reduced_transparency` (`bool`) and `contrast` (`Contrast::NoPreference`,
`More`, `Less`). `AccessibilityOverrides` has the same fields as `Option`s.

`Clone`, not `Copy`: clone it into each handler.
