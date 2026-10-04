# Accessibility settings

Crate: `libero`
Import: `use libero::hooks::use_accessibility;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/accessibility.rs>
Index: [index.md](index.md) lists every other page
Description: Reads the reader's accessibility settings and lets an app force reduced motion.

`use_accessibility() -> AccessibilityHandle` reads the reader's accessibility
settings: reduced motion, forced colors, contrast and reduced transparency. A
settings page can force reduced motion on or off over the system's, and the
choice is kept for the next visit (the web's localStorage, Android) until
`set_reduced_motion(None)` clears it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Switch, Text},
    hooks::use_accessibility,
};

#[component]
fn Settings() -> Element {
    let accessibility = use_accessibility();

    rsx! {
        Flex { direction: "column", gap: "sm",
            Switch {
                label: "Reduce motion",
                checked: accessibility.reduced_motion(),
                onchange: {
                    let accessibility = accessibility.clone();
                    move |on: bool| accessibility.set_reduced_motion(Some(on))
                },
            }
            Button {
                variant: "outlined",
                onclick: {
                    let accessibility = accessibility.clone();
                    move |_| accessibility.set_reduced_motion(None)
                },
                "Follow the system"
            }
            Text {
                "Contrast: {accessibility.contrast().as_str()}, forced colors: "
                "{accessibility.forced_colors()}, reduced transparency: "
                "{accessibility.reduced_transparency()}"
            }
        }
    }
}
```

In a browser and Android's WebView the settings are the page's media queries.
Natively they come from the desktop portal on Linux (`org.freedesktop.appearance`,
GNOME's and older KDE's own keys as a fallback); other native platforms report no preference.

A forced reduced motion is kept across restarts in a browser (`localStorage`) and on
Android; on desktop and native it lasts for the session.

## Forced settings

A forced reduced motion reaches libero's own CSS and motion on every platform.
A `<style>` the app adds itself still follows the system. The other settings
are read-only.

## API

```rust,ignore
pub fn use_accessibility() -> AccessibilityHandle
```

| Method | Returns | Description |
|---|---|---|
| `get()` | `AccessibilityPreferences` | All four at once: `reduced_motion` (forced or the system's), `forced_colors`, `contrast`, `reduced_transparency`. |
| `reduced_motion()` | `bool` | Whether motion is reduced: the forced answer, else the system's. |
| `set_reduced_motion(reduced: Option<bool>)` | `()` | Forces reduced motion on or off, kept where the platform can; `None` follows the system again. |
| `forced_colors()` | `bool` | Whether the system forces its own colors. |
| `contrast()` | `Contrast` | `Contrast::NoPreference`, `More` or `Less`. |
| `reduced_transparency()` | `bool` | Whether the system asks for less transparency. |

Every getter is reactive. `Clone`, not `Copy`: clone it into each handler.
