# Menu

Crate: `libero`
Import: `use libero::components::{Menu, MenuEntry, MenuItem, use_menu};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/menu>
Index: [index.md](index.md) - every other component's markdown page
Description: A list of commands that drops from a trigger, with groups, separators and submenus.

A list of commands that drops from a trigger. The items are data, not
children. `use_menu()` keeps the open state in your scope, and the trigger is
your own `Button`, wired by `menu.a11y_attributes()`. The menu is portaled, so
no `overflow: hidden` ancestor clips it.

A submenu opens beside its item on ArrowRight, a click, or when the pointer
rests on the item for 150ms. Moving to a sibling waits as long, so the pointer
can cross one on its way into the submenu.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Flex, Menu, MenuEntry, MenuItem, Text, use_menu};

#[component]
fn Demo() -> Element {
    let menu = use_menu();
    let mut last = use_signal(|| String::from("nothing yet"));
    let pick = move |name: &'static str| move |_| last.set(name.to_string());
    let mut hidden = use_signal(|| false);

    let items = vec![
        MenuEntry::Group {
            label: "Edit".into(),
            items: vec![
                MenuItem::new("Cut")
                    .shortcut("Control+X")
                    .onselect(pick("Cut"))
                    .into(),
                MenuItem::new("Paste").disabled(true).onselect(pick("Paste")).into(),
            ],
        },
        MenuEntry::Separator,
        MenuItem::new("Show hidden")
            .checkbox(hidden())
            .onselect(move |_| hidden.toggle())
            .into(),
        MenuItem::new("Save").onselect(pick("Save")).into(),
        MenuItem::new("Share")
            .submenu(vec![MenuItem::new("Email").onselect(pick("Email")).into()])
            .into(),
    ];

    rsx! {
        Flex {
            direction: "row",
            align: "center",
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
ArrowUp on its last.

In the menu, ArrowDown and ArrowUp move an item, wrapping unless `loop_focus`
is off, and Home and End go to the ends. Enter and Space choose. ArrowRight
opens a submenu and ArrowLeft closes it again. Escape closes only the menu it
is pressed in and returns focus to what opened it. Tab closes every level and
moves on from the trigger. Typing jumps to an item, "s" to the next one
starting with S and "sav" to Save. A pause of half a second starts over.

Put a shortcut hint in `shortcut`, not `trailing`. A screen reader then hears
it as `aria-keyshortcuts`, not as part of the item's name.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `state` | `MenuState` | required | From `use_menu()`. Holds the open state and wires the trigger to the menu. |
| `items` | `Vec<MenuEntry>` | required | The menu, in order. `MenuEntry::Item`, `MenuEntry::Group { label, items }` for a named section, and `MenuEntry::Separator`. |
| `children` | `Element` | required | The trigger, carrying `menu.a11y_attributes()`. It needs no click or key handler of its own. |
| `side` | `Side` | `Bottom` | Which side of the trigger the menu opens on. It flips when that side has no room. `Start`/`End` are logical: `Start` is the left under `dir="ltr"`, the right under `rtl`. |
| `align` | `Align` | `Start` | Where the menu lines up along that side. |
| `close_on_select` | `bool` | `true` | Whether choosing an item closes the menu. |
| `loop_focus` | `bool` | `true` | Whether the arrow keys wrap from the last item to the first. |
| `size` | `Size` | `md` | Item height and font size. |
| `radius` | `Size` | `sm` | The menu's corner radius. The items' corners follow it. |
| `disabled` | `bool` | `false` | The trigger opens nothing, and an open menu closes. Disable the trigger too, so it looks disabled. |
| `onedge` | `Callback<MenuEdge>` | - | Called with ArrowLeft on the top level, or ArrowRight on an item without a submenu. `Menubar` uses it to move to the next menu. |

Like every component, `Menu` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the menu.

### MenuItem

| Method | Type | Default | Description |
|---|---|---|---|
| `new(label)` | `String` | required | The visible text, the accessible name, and what typeahead matches. |
| `onselect` | `FnMut(())` | - | Runs when the item is chosen by a click, Enter or Space. |
| `submenu` | `Vec<MenuEntry>` | - | Opens a second menu beside the item instead. An item runs a command or opens a submenu, and the later call wins. |
| `leading` | `Element` | - | Before the label, such as an icon. Nothing interactive, since it sits inside the item's button. |
| `trailing` | `Element` | - | At the far end, such as a badge. It joins the accessible name. Nothing interactive. |
| `shortcut` | `&str` | - | The key that runs the item outside the menu, in `aria-keyshortcuts` syntax (`"Control+X"`). Drawn as a hint ("Ctrl+X") and kept out of the name. `Control`, `Shift`, `Alt` and `Meta` are drawn in the localization's `menu` words ("Strg+Umschalt+S" in German). You bind the key yourself. |
| `radio` | `bool` | - | Makes the item one choice of several, with a check while `true`. Put the choices in one `Group` and keep one checked. A menu opens on its checked item. |
| `checkbox` | `bool` | - | Makes the item an on/off setting, with a check while `true`. Flip it in `onselect`. An item is `radio` or `checkbox`, and the later call wins. |
| `disabled` | `bool` | `false` | Stays in the arrow-key order but cannot be chosen, and typeahead skips it. |

## Theme defaults

`MenuDefaults` on the theme holds `size`, `radius`, `max_height` (`"340px"`),
the per-size `sizes`, `submenu_delay` (150ms), and the defaults of
`close_on_select` and `loop_focus`. The surface is the theme's `paper`.

## CSS variables

Per size: `--lsx-menu-font-size-{size}`, `--lsx-menu-item-height-{size}`,
`--lsx-menu-padding-x-{size}` and `--lsx-menu-label-font-size-{size}`. Also
`--lsx-menu-max-height`.

## Data attributes

On the menu: `size-{size}`, `radius-{size}` and `bordered`.
