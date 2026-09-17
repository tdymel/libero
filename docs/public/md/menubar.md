# Menubar

Crate: `libero`
Import: `use libero::components::{MenuEntry, MenuItem, Menubar, MenubarMenu};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/menubar.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A row of menus. Each menu is a `Menu`, and the bar is a single tab stop with one menu open at most.

A row of menus, like a desktop app's File, Edit and View. Each menu is a
[`Menu`](menu.md) with the same `MenuEntry` items. The bar keeps one menu open
at most and is a single tab stop. Click a trigger to open its menu. While one
is open, hovering another trigger switches to it. Nothing opens on hover alone.

Use it in an app such as an editor. For page navigation, use links. For one set
of actions, use a single `Menu`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{MenuEntry, MenuItem, Menubar, MenubarMenu};

#[component]
fn Demo() -> Element {
    let item = |name: &str| -> MenuEntry { MenuItem::new(name).onselect(|_| {}).into() };
    let mut wrap = use_signal(|| true);
    let menus = vec![
        MenubarMenu::new("File", vec![
            item("New"),
            MenuItem::new("Open recent").submenu(vec![item("notes.md")]).into(),
            MenuEntry::Separator,
            MenuItem::new("Save").shortcut("Control+S").onselect(|_| {}).into(),
        ]),
        MenubarMenu::new("Edit", vec![
            item("Undo"),
            item("Redo"),
            MenuEntry::Separator,
            MenuItem::new("Word wrap")
                .checkbox(wrap())
                .onselect(move |_| wrap.toggle())
                .into(),
        ]),
        MenubarMenu::new("View", vec![item("Zoom in")]).disabled(true),
    ];
    rsx! { Menubar { aria_label: "Main", menus } }
}
```

## Accessibility

`aria_label` names the bar and is required.

On a trigger, ArrowLeft and ArrowRight move along the bar, wrapping unless
`loop_focus` is off. If a menu is open, the next one opens. A disabled trigger
takes focus and opens nothing. Home and End go to the first and last trigger.
Enter, Space and ArrowDown open the menu on its first item, ArrowUp on its last.
Typing jumps to a trigger by its label.

In an open menu, ArrowRight on an item without a submenu and ArrowLeft on the
top level move to the next menu. ArrowRight on a submenu item opens the
submenu, and ArrowLeft closes it. Escape closes the menu and returns focus to
its trigger. Tab closes it and leaves the bar. The rest works as in `Menu`,
including `MenuItem`'s `shortcut` and `checkbox`.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `menus` | `Vec<MenubarMenu>` | required | The top-level menus, in order. |
| `aria_label` | `String` | required | The bar's accessible name. |
| `loop_focus` | `bool` | `true` | Whether the arrow keys wrap at the ends, along the bar and down each menu. |
| `side` | `Side` | `Bottom` | Which side of its trigger every menu opens on. It flips when that side has no room. `Start`/`End` are logical: `Start` is the left under `dir="ltr"`, the right under `rtl`. |
| `align` | `Align` | `Start` | Where each menu lines up along that side. |
| `size` | `Size` | `md` | The triggers' font and padding, and each menu's item size. |
| `radius` | `Size` | `sm` | The triggers' and the menus' corner radius. |

Like every component, `Menubar` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the bar.

### `MenubarMenu`

`MenubarMenu::new(label, items)`, plus `.disabled(bool)`.

| Field | Type | Default | Description |
|---|---|---|---|
| `label` | `String` | required | The trigger's text, which typeahead on the bar matches. |
| `items` | `Vec<MenuEntry>` | required | `Menu`'s items. |
| `disabled` | `bool` | `false` | The trigger stays in view and in the arrow order, and opens nothing. |

## Theme defaults

`MenubarDefaults` on the theme. The menus read `MenuDefaults`, but take the
bar's `loop_focus`.

| Field | Description |
|---|---|
| `size` | Default `size`, `md`. |
| `radius` | Default `radius`, `sm`. |
| `gap` | Space between triggers, `2px`. |
| `sizes` | Per size, the trigger's `font_size`, `padding_x` and `padding_y`. |
| `loop_focus` | Default `loop_focus`, `true`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-menubar-gap` | Space between triggers. |
| `--lsx-menubar-font-size-<size>` | Trigger font size per size step. |
| `--lsx-menubar-padding-x-<size>`, `--lsx-menubar-padding-y-<size>` | Trigger padding per size step. |
| `--lsx-menubar-trigger-font`, `--lsx-menubar-trigger-pad-x`, `--lsx-menubar-trigger-pad-y`, `--lsx-menubar-trigger-radius` | The values in effect, on the bar. |

## Data attributes

State tokens on the bar's `data-state`: `size-<size>` and `radius-<size>`. Each
trigger carries `data-menubar-index`.
