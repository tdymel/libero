# use_menu

Crate: `libero`
Import: `use libero::components::use_menu;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/menu/state.rs>
Index: [index.md](index.md) lists every other page
Description: Keeps a Menu's open state in your scope, so your own trigger opens it.

`use_menu() -> MenuState` keeps a `Menu`'s open state in your scope, so your
own trigger opens it and carries its aria wiring. Pass it as the menu's
`state`. [Menu](menu.md) covers groups, checkboxes, submenus and the keyboard.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Flex, Menu, MenuEntry, MenuItem, Text, use_menu};

#[component]
fn FileActions() -> Element {
    let menu = use_menu();
    let mut last = use_signal(|| String::from("nothing yet"));
    let pick = move |name: &'static str| move |_| last.set(name.to_string());

    let items: Vec<MenuEntry> = vec![
        MenuItem::new("Rename").onselect(pick("Rename")).into(),
        MenuItem::new("Duplicate").onselect(pick("Duplicate")).into(),
        MenuItem::new("Delete").onselect(pick("Delete")).into(),
    ];

    rsx! {
        Flex { direction: "row", gap: "md",
            Menu { state: menu, items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "File" }
            }
            Text { "Last chosen: {last}" }
        }
    }
}
```

## Accessibility

Spread `a11y_attributes()` on the trigger. It carries the trigger's id,
`aria-haspopup` and `aria-expanded`, plus `aria-controls` while the menu is
open. The `Menu` around the trigger handles the click and the keys that open
it.

## API

```rust,ignore
pub fn use_menu() -> MenuState
```

| Method | Returns | Description |
|---|---|---|
| `is_open()` | `bool` | Whether the menu is open. |
| `open()` | `()` | Opens it with the first item focused. |
| `close()`, `toggle()` | `()` | Closes or flips it. |
| `id()` | `String` | The id the wiring is built from. |
| `a11y_attributes()` | `Vec<Attribute>` | The trigger's aria wiring. |

`Copy`.
