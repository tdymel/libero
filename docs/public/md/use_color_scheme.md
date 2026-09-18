# use_color_scheme

Crate: `libero`
Import: `use libero::hooks::use_color_scheme;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/hooks/color_scheme.rs>
Index: [index.md](index.md) lists every other page
Description: Reads and sets light or dark, or hands the choice back to the platform.

`use_color_scheme() -> ColorSchemeHandle` reads and sets light or dark.
`setting()` is what the app asked for, `resolved()` is the scheme on screen.
[ColorSchemeButton](color_scheme_button.md) is the ready-made switch built on
it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{SegmentedControl, Text},
    hooks::use_color_scheme,
    theme::ColorSchemeSetting,
};

const SETTINGS: [(&str, ColorSchemeSetting); 3] = [
    ("System", ColorSchemeSetting::System),
    ("Light", ColorSchemeSetting::Light),
    ("Dark", ColorSchemeSetting::Dark),
];

#[component]
fn SchemePicker() -> Element {
    let scheme = use_color_scheme();
    let resolved = scheme.resolved().as_str();
    let current = SETTINGS
        .iter()
        .find(|(_, setting)| *setting == scheme.setting())
        .map_or("System", |(label, _)| *label);

    rsx! {
        SegmentedControl {
            "aria-label": "Colour scheme",
            value: current.to_string(),
            options: SETTINGS.map(|(label, _)| label.to_string()).to_vec(),
            onchange: move |next: String| {
                if let Some((_, setting)) = SETTINGS.iter().find(|(label, _)| *label == next) {
                    scheme.set(*setting);
                }
            },
        }
        Text { "On screen: {resolved}" }
    }
}
```

`toggle()` flips to the other scheme and `cycle()` steps through all three
settings. `System` hands the choice back to the platform, and the component
re-renders when the platform changes its mind.

## Web and native

On the web the choice is kept in `localStorage`, so a reload comes back to it.
[Theming](theming.md) shows the script that restores it before the first
paint.

## API

```rust,ignore
pub fn use_color_scheme() -> ColorSchemeHandle
```

| Method | Returns | Description |
|---|---|---|
| `setting()` | `ColorSchemeSetting` | `System`, `Light` or `Dark`: what the app asked for. |
| `resolved()` | `ColorScheme` | `Light` or `Dark`: what is on screen. |
| `set(setting: impl Into<ColorSchemeSetting>)` | `()` | Pins a scheme, or `System` to follow the platform. |
| `toggle()` | `()` | Flips to the other scheme. Flipping back to the platform's hands the choice back to it. |
| `cycle()` | `()` | From following the platform, to the other scheme, to the platform's own pinned, and back. |
| `next_in_cycle()` | `ColorSchemeSetting` | The setting `cycle()` moves to next. |
| `set_theme(name: &'static str)` | `()` | Selects a theme beyond the pair, by the name given to `ThemeSet::named`. |

`Clone`, not `Copy`.
