# Menubar

Crate: `libero`
Import: `use libero::components::{MenuEntry, MenuItem, Menubar, MenubarMenu};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/menubar.rs>
Index: [index.md](index.md) lists every other page
Description: A row of menus. Each menu is a `Menu`, and the bar is a single tab stop with one menu open at most.

A row of menus, like a desktop app's File, Edit and View. Each menu is a
[`Menu`](menu.md) with the same `MenuEntry` items. The bar keeps one menu open
at most and is a single tab stop. Click a trigger to open its menu. While one
is open, hovering another trigger switches to it. Nothing opens on hover alone.

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

## When to use it

Use it in an app such as an editor. For page navigation, use links. For one set
of actions, use a single `Menu`.

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
| `menu_parts` | `Parts<MenuPart>` | - | Every menu's `parts`, the `Menu` page's Style API table. The menus open in a portal, out of the bar's `sx` and `parts`. |
| `parts` | `Parts<MenubarPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. The menus take `menu_parts`. |

Like every component, `Menubar` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the bar.

### `MenubarMenu`

`MenubarMenu::new(label, items)`, plus `.disabled(bool)`.

| Field | Type | Default | Description |
|---|---|---|---|
| `new(label, items)` | `String, Vec<MenuEntry>` | required | The trigger's text, which typeahead on the bar matches, and `Menu`'s items. |
| `disabled` | `bool` | `false` | The trigger stays in view and in the arrow order, and opens nothing. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work. The menus open in a portal, outside the bar: style them with
`menu_parts`, which takes the [`Menu`](menu.md) parts.

| Part | `data-slot` | Description |
|---|---|---|
| `MenubarPart::Trigger` | `trigger` | A menu's trigger button. `aria-expanded` is `true` while its menu is open. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Enters the bar, one tab stop. In an open menu: closes it and leaves the bar. |
| `Left` or `Right` | On a trigger: moves along the bar, disabled triggers included, wrapping unless `loop_focus` is off. If a menu is open, the next one opens. |
| `Home` or `End` | On a trigger: goes to the first or last trigger. |
| `Enter`, `Space` or `Down` | On a trigger: opens its menu on the first item. |
| `Up` | On a trigger: opens its menu on the last item. |
| `Right` | In an open menu: opens a submenu item's submenu, or else moves to the next menu. |
| `Left` | In an open menu: closes a submenu, or on the top level moves to the previous menu. |
| `Escape` | Closes the menu and returns focus to its trigger. |

### Libero handles

- On a trigger, typing jumps to a trigger by its label.
- A disabled trigger takes focus and opens nothing.
- The rest works as in [`Menu`](menu.md), including `MenuItem`'s `shortcut`
  and `checkbox`.

### You must

- Name the bar with `aria_label`. It is required.

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
trigger carries `data-slot="trigger"` and `data-menubar-index`.
