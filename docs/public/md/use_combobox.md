# use_combobox

Crate: `libero`
Import: `use libero::components::use_combobox;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/combobox/state.rs>
Index: [index.md](index.md) lists every other page
Description: Keeps a Combobox's open state and active option in your scope, for a trigger of your own.

`use_combobox() -> ComboboxState` keeps a `Combobox`'s open state and active
option in your scope, so your own trigger opens, closes and wires it. Pass it
as the combobox's `state`. [Combobox](combobox.md) documents the component and
its keyboard.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{
    Button, Combobox, ComboboxOption, ComboboxOptionArgs, Options, use_combobox,
};

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

#[component]
fn FruitPicker() -> Element {
    let fruit = use_combobox();
    let mut picked = use_signal(|| None::<Fruit>);

    rsx! {
        Combobox {
            state: fruit,
            options: Fruit::options().to_vec(),
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    selected: picked() == Some(o.value),
                    onpick: move |_| {
                        picked.set(Some(o.value));
                        fruit.close();
                    },
                    "{o.value.label()}"
                }
            },
            Button {
                variant: "outlined",
                attributes: fruit.a11y_attributes(),
                onclick: move |_| fruit.toggle(),
                match picked() {
                    Some(fruit) => rsx! { "{fruit.label()}" },
                    None => rsx! { "Pick a fruit" },
                }
            }
        }
    }
}
```

## Accessibility

`a11y_attributes()` gives the trigger its role, `aria-haspopup` and
`aria-expanded`. While the list is open and has rows, it adds `aria-controls`
and `aria-activedescendant`. Spread them on whatever control sits inside the
combobox.

## API

```rust,ignore
pub fn use_combobox() -> ComboboxState
```

| Method | Returns | Description |
|---|---|---|
| `is_open()` | `bool` | Whether the list is open. |
| `open()`, `close()`, `toggle()`, `set_open(bool)` | `()` | Opens or closes it. |
| `active()` | `Option<usize>` | The row the arrow keys are on. |
| `set_active(Option<usize>)` | `()` | Moves the highlight. `None` is none. |
| `id()` | `String` | The id the wiring is built from. |
| `a11y_attributes()` | `Vec<Attribute>` | The trigger's aria wiring. |

`Copy`.
