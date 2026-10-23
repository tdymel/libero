# Accessibility settings

Crate: `libero`
Import: `use libero::hooks::use_accessibility;`
Feature: `use-accessibility` (on by default; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/accessibility.rs>
Index: [index.md](index.md) lists every other page
Description: Reads the reader's accessibility settings and lets an app force reduced motion.

`use_accessibility() -> AccessibilityHandle` reads the reader's accessibility
settings: reduced motion, forced colors, contrast and reduced transparency. A
settings page can force reduced motion on or off over the system's, and the
choice is kept for the next visit (libero's local storage) until
`set_reduced_motion(None)` clears it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Switch, Text},
    hooks::use_accessibility,
};

/// The forced setting is kept across visits, so leaving the page hands the site back to the
/// system if the demo forced it.
#[component]
pub fn Settings() -> Element {
    let accessibility = use_accessibility();
    let mut forced = use_hook(|| CopyValue::new(false));
    use_drop({
        let accessibility = accessibility.clone();
        move || {
            if forced() {
                accessibility.set_reduced_motion(None);
            }
        }
    });

    rsx! {
        Flex { direction: "column", gap: "sm",
            Switch {
                label: "Reduce motion",
                checked: accessibility.reduced_motion(),
                onchange: {
                    let accessibility = accessibility.clone();
                    move |on: bool| {
                        forced.set(true);
                        accessibility.set_reduced_motion(Some(on));
                    }
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

## API

```rust,ignore
pub fn use_accessibility() -> AccessibilityHandle
```

| Method | Returns | Description |
|---|---|---|
| `get()` | `AccessibilityPreferences` | All four at once: `reduced_motion` (forced or the system's), `forced_colors`, `contrast`, `reduced_transparency`. The system's settings are the page's media queries in a browser and Android's WebView; natively they come from the Linux desktop portal (`org.freedesktop.appearance`, GNOME's and older KDE's own keys as a fallback), and other native platforms report no preference. |
| `reduced_motion()` | `bool` | Whether motion is reduced: the forced answer, else the system's. |
| `set_reduced_motion(reduced: Option<bool>)` | `()` | Forces reduced motion on or off; `None` follows the system again. It reaches libero's own CSS and motion on every platform, but a `<style>` the app adds itself still follows the system. Kept across restarts under `lsx-reduced-motion`: in a browser's `localStorage`, and in a file on Android, the desktop WebView and native where `libero::platform::set_storage_dir` (or libero's `desktop` or `native` feature) gives local storage a directory. Elsewhere it lasts for the session. |
| `forced_colors()` | `bool` | Whether the system forces its own colors. Read-only: only reduced motion can be forced. |
| `contrast()` | `Contrast` | `Contrast::NoPreference`, `More` or `Less`. Read-only. |
| `reduced_transparency()` | `bool` | Whether the system asks for less transparency. Read-only. |

Every getter is reactive. `Clone`, not `Copy`: clone it into each handler.
