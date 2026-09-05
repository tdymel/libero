# Menu

Crate: `libero`
Import: `use libero::components::{Menu, MenuEntry, MenuItem, use_menu};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/menu>
Index: [index.md](index.md) - every other component's markdown page
Description: A list of commands that drops from a caller-supplied trigger - the WAI-ARIA menu button, with groups, separators and submenus.

A list of commands that drops from a trigger - the WAI-ARIA menu button. The
items are data, not children, so the menu owns their order: the arrow keys and
typeahead index a `Vec` rather than asking the page. `use_menu()` keeps the
open state in your scope, and the trigger is your own `Button`, wired by
`menu.a11y_attributes()`. The menu is a surface from the theme's `paper`
defaults, portaled so no `overflow: hidden` ancestor clips it.

Focus moves onto the items - one of them is tabbable at a time - so a screen
reader follows it. A submenu opens beside its item on ArrowRight, a click, or
the pointer resting on the item for `theme.menu.submenu_delay` (150ms). Moving
on to a sibling waits the same delay, so the pointer can cross one on its way
into the submenu. There is no "safe triangle".

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Flex, Kbd, Menu, MenuEntry, MenuItem, Text, use_menu};

#[component]
fn Demo() -> Element {
    let menu = use_menu();
    let mut last = use_signal(|| String::from("nothing yet"));
    let pick = move |name: &'static str| move |_| last.set(name.to_string());

    let items = vec![
        MenuEntry::Group {
            label: "Edit".into(),
            items: vec![
                MenuItem::new("Cut")
                    .trailing(rsx! { Kbd { "Ctrl X" } })
                    .on_select(pick("Cut"))
                    .into(),
                MenuItem::new("Paste").disabled(true).on_select(pick("Paste")).into(),
            ],
        },
        MenuEntry::Separator,
        MenuItem::new("Save").on_select(pick("Save")).into(),
        MenuItem::new("Share")
            .submenu(vec![MenuItem::new("Email").on_select(pick("Email")).into()])
            .into(),
    ];

    rsx! {
        Flex {
            direction: "row",
            gap: "md",
            Menu {
                state: menu,
                items,
                Button { variant: "outlined", attributes: menu.a11y_attributes(), "Actions" }
            }
            Text { size: "sm", "Last chosen: {last()}" }
        }
    }
}
```

## Accessibility

On the trigger, Enter, Space and ArrowDown open the menu on its first item,
ArrowUp on its last. In the menu, ArrowDown/ArrowUp move an item (wrapping
unless `loop_focus` is off), Home/End go to the ends, Enter and Space choose,
ArrowRight opens a submenu and ArrowLeft closes it. Escape closes only the menu
it is pressed in and hands focus back to whatever opened it; Tab closes every
level and moves on from the trigger. Typing jumps to an item: "s" to the next
one starting with S, "sav" to Save; a half-second pause starts over. A disabled
item stays in the arrow order but cannot be chosen.

- Spread `menu.a11y_attributes()` on the trigger; it ties the trigger to the
  menu.
- `leading` and `trailing` sit inside the item's button: never put anything
  interactive there.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `state` | `MenuState` | required | From `use_menu()`. |
| `items` | `Vec<MenuEntry>` | required | `Item`, `Group { label, items }`, `Separator`. |
| `children` | `Element` | required | The trigger, carrying `menu.a11y_attributes()`. |
| `side` | `Side` | `Bottom` | Opening side; flips when it has no room. |
| `align` | `Align` | `Start` | Alignment along that side. |
| `close_on_select` | `bool` | `true` | Choosing an item closes the menu. |
| `loop_focus` | `bool` | `true` | Arrows wrap. |
| `size` | `Size` | `md` | Item height and font size. |
| `radius` | `Size` | `sm` | Menu corner radius; items nest with it minus the padding. |
| `disabled` | `bool` | `false` | The trigger opens nothing. |
| `on_edge` | `Option<Callback<MenuEdge>>` | `None` | Hears ArrowLeft (top level) and ArrowRight (an item without a submenu) when no submenu answers them - `MenuEdge::Previous` / `Next`. `Menubar` uses it. |

`sx`, `class`, `states` and `attributes` land on the root menu box.

`MenuItem` builder: `new(label)`, `on_select(FnMut(()))`, `submenu(Vec<MenuEntry>)`
(an item does one or the other; the later call wins), `leading(Element)`,
`trailing(Element)`, `disabled(bool)`.

## Theme defaults

`theme.menu: MenuDefaults` - `size` (`Md`), `radius` (`Sm`), `max_height`
(`"340px"`), `sizes: Sizes<MenuSizeLevel { font_size, item_height, padding_x,
label_font_size }>`, `submenu_delay` (`150` ms).

## CSS variables

`--lsx-menu-max-height`, `--lsx-menu-font-size-{size}`,
`--lsx-menu-item-height-{size}`, `--lsx-menu-padding-x-{size}`,
`--lsx-menu-label-font-size-{size}`; resolved on the box as
`--lsx-menu-item-font`, `--lsx-menu-item-min-height`, `--lsx-menu-item-pad-x`,
`--lsx-menu-label-font`, `--lsx-menu-item-radius`.

## Data attributes

On the menu box: `size-{size}`, `radius-{size}`, `bordered`.
