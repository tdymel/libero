# Menubar

Crate: `libero`
Import: `use libero::components::{MenuEntry, MenuItem, Menubar, MenubarMenu};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/navigation/menubar.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A row of menus - the WAI-ARIA menubar. Each menu is a `Menu`; the bar owns which one is open and the single tab stop.

A row of menus - the WAI-ARIA menubar, a desktop application's File, Edit and
View. Each menu is a `Menu` with the same `MenuEntry` items; the bar owns which
one is open (at most one) and which trigger is its tab stop, so the caller
passes data and nothing else. Click a trigger to open its menu; while one is
open, the pointer on another trigger switches to it with no delay. Nothing
opens on hover alone.

It belongs in an application - an editor, an IDE. For a page's navigation use
links; for one set of actions, a single `Menu`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{MenuEntry, MenuItem, Menubar, MenubarMenu};

#[component]
fn Demo() -> Element {
    let item = |name: &str| -> MenuEntry { MenuItem::new(name).on_select(|_| {}).into() };
    let menus = vec![
        MenubarMenu::new("File", vec![
            item("New"),
            MenuItem::new("Open recent").submenu(vec![item("notes.md")]).into(),
            MenuEntry::Separator,
            item("Save"),
        ]),
        MenubarMenu::new("Edit", vec![item("Undo"), item("Redo")]),
        MenubarMenu::new("View", vec![item("Zoom in")]).disabled(true),
    ];
    rsx! { Menubar { aria_label: "Main", menus } }
}
```

## Keyboard

On a trigger: ArrowLeft/ArrowRight move along the bar, wrapping unless
`loop_focus` is off; if a menu is open the new one opens. A disabled trigger
takes focus like the others but opens nothing, so moving onto it closes the
open menu. Home/End go to the first/last trigger. Enter, Space and
ArrowDown open the menu on its first item, ArrowUp on its last. Typing jumps to
a trigger by its label.

In an open menu: ArrowRight on an item without a submenu, and ArrowLeft on the
top level, close the menu and open the neighbouring one on its first item.
ArrowRight on a submenu item opens the submenu; ArrowLeft in a submenu closes
it. Escape closes the menu and returns focus to its trigger; Tab closes it and
leaves the bar. Everything else is `Menu`'s.

## Accessibility

`aria_label` is required: it names the bar.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `menus` | `Vec<MenubarMenu>` | required | The top-level menus, in order. |
| `aria_label` | `String` | required | The bar's accessible name. |
| `loop_focus` | `bool` | `true` | Arrows wrap - along the bar and down each menu. |
| `side` | `Side` | `Bottom` | Opening side of every menu; flips when it has no room. |
| `align` | `Align` | `Start` | Alignment along that side. |
| `size` | `Size` | `md` | Trigger font and padding, and each menu's item size. |
| `radius` | `Size` | `sm` | Trigger and menu corner radius. |

`sx`, `class`, `states` and `attributes` land on the bar.

`MenubarMenu { label, items, disabled }`, or `MenubarMenu::new(label, items)`
and `.disabled(bool)`. `items` is `Menu`'s `Vec<MenuEntry>`.

## Theme defaults

`theme.menubar: MenubarDefaults` - `size` (`Md`), `radius` (`Sm`), `gap`
(`"2px"`), `sizes: Sizes<MenubarSizeLevel { font_size, padding_x, padding_y }>`.
The menus read `theme.menu`.

## CSS variables

`--lsx-menubar-gap`, `--lsx-menubar-font-size-{size}`,
`--lsx-menubar-padding-x-{size}`, `--lsx-menubar-padding-y-{size}`; resolved on
the bar as `--lsx-menubar-trigger-font`, `--lsx-menubar-trigger-pad-x`,
`--lsx-menubar-trigger-pad-y`, `--lsx-menubar-trigger-radius`.

## Data attributes

On the bar: `size-{size}`, `radius-{size}`. Triggers: `data-menubar-index`.
