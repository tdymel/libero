# Menu

Crate: `libero`
Import: `use libero::components::{Menu, MenuEntry, MenuItem, use_menu};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/menu>
Index: [index.md](index.md) lists every other page
Description: A list of commands that drops from a trigger, with groups, separators and submenus.

A list of commands that drops from a trigger. The items are data, not
children. `use_menu()` keeps the open state in your scope, and the trigger is
your own `Button`, wired by `menu.a11y_attributes()`. The menu is portaled, so
no `overflow: hidden` ancestor clips it.

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
                MenuItem::new("Paste")
                    .disabled(true)
                    .description("The clipboard is empty")
                    .onselect(pick("Paste"))
                    .into(),
            ],
        },
        MenuEntry::Separator,
        MenuItem::new("Show hidden")
            .checkbox(hidden())
            .onselect(move |_| hidden.toggle())
            .into(),
        MenuItem::new("Save").onselect(pick("Save")).into(),
        MenuItem::new("Zoom in").keep_open().onselect(pick("Zoom in")).into(),
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
            Text { size: "sm", role: "status", "Last chosen: {last()}" }
        }
    }
}
```

## Submenus

A submenu opens beside its item on ArrowRight, a click, or when the pointer
rests on the item for 150ms. Moving to a sibling waits as long, so the pointer
can cross one on its way into the submenu.

## Links

An item can be a link: `MenuItem::new("Docs").href(url)` renders an `<a>` that
opens `url` in a new tab. Space activates it like Enter.

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
| `parts` | `Parts<MenuPart>` | - | Styles for the inner parts in the Style API tab, on every menu level, submenus too, as `sx` does: `Parts::new().part(MenuPart::Label, sx().font_weight("500"))`. |

Like every component, `Menu` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the menu; `sx`
also on every submenu.

### MenuItem

| Method | Type | Default | Description |
|---|---|---|---|
| `new(label)` | `String` | required | The visible text, the accessible name, and what typeahead matches. |
| `onselect` | `FnMut(())` | - | Runs when the item is chosen by a click, Enter or Space. |
| `submenu` | `Vec<MenuEntry>` | - | Opens a second menu beside the item instead. An item runs a command or opens a submenu, and the later call wins. |
| `href` | `String` | - | Makes the item a link: an `<a>` that opens the URL in a new tab, so middle-click and the context menu work. Replaces `onselect` and `submenu`. A disabled link has no `href`. The label ends in a new-tab icon and a hidden "(opens in a new tab)", as on `Anchor`. |
| `new_tab_hint` | `bool` | `true` | With `href`: `false` drops the new-tab icon and the hidden hint. |
| `keep_open` | `()` | - | Choosing the item leaves the menu open, and focus stays on it, whatever the menu's `close_on_select` says. `close_on_select(bool)` overrides it either way. |
| `leading` | `Element` | - | Before the label, such as an icon. Nothing interactive, since it sits inside the item's button. |
| `trailing` | `Element` | - | At the far end, such as a badge. It joins the accessible name. Nothing interactive. |
| `shortcut` | `&str` | - | The key that runs the item outside the menu, in `aria-keyshortcuts` syntax (`"Control+X"`). Drawn as a hint ("Ctrl+X") and kept out of the name. `Control`, `Shift`, `Alt` and `Meta` are drawn in the localization's `menu` words ("Strg+Umschalt+S" in German). You bind the key yourself. |
| `radio` | `bool` | - | Makes the item one choice of several, with a check while `true`. Put the choices in one `Group` and keep one checked. A menu opens on its checked item. |
| `checkbox` | `bool` | - | Makes the item an on/off setting, with a check while `true`. Flip it in `onselect`. An item is `radio` or `checkbox`, and the later call wins. |
| `disabled` | `bool` | `false` | Stays in the arrow-key order but cannot be chosen, and typeahead skips it. |
| `description` | `String` | - | Read after the label through `aria-describedby`, never shown. Say why a `disabled` item cannot be chosen. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `MenuPart::Item` | `item` | A row: `menuitem`, `menuitemradio` or `menuitemcheckbox`. |
| `MenuPart::GroupLabel` | `group-label` | A group's visible name. |
| `MenuPart::Check` | `check` | The check column, on every row of a level with a checkable item. |
| `MenuPart::Leading` | `leading` | The item's `leading` content. |
| `MenuPart::Label` | `label` | The item's text. |
| `MenuPart::Trailing` | `trailing` | The item's `trailing` content. |
| `MenuPart::Shortcut` | `shortcut` | The key hint, hidden from screen readers. |
| `MenuPart::Chevron` | `chevron` | A submenu item's arrow. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Enter`, `Space` or `Down` | On the trigger: opens the menu on its first item. |
| `Up` | On the trigger: opens the menu on its last item. |
| `Down` or `Up` | In the menu: moves an item, wrapping unless `loop_focus` is off. |
| `Home` or `End` | In the menu: goes to the first or last item. |
| `Enter` or `Space` | In the menu: chooses the item. |
| `Right` | Opens a submenu. |
| `Left` | Closes a submenu again. |
| `Escape` | Closes only the menu it is pressed in and returns focus to what opened it. |
| `Tab` | Closes every level and moves on from the trigger. |

### Libero handles

- Typing jumps to an item, "s" to the next one starting with S and "sav" to
  Save. A pause of half a second starts over.
- `menu.a11y_attributes()` wires your trigger.
- Android's Back button closes the menu as Escape does, rather than the app.

### You must

- Put a shortcut hint in `shortcut`, not `trailing`. A screen reader then
  hears it as `aria-keyshortcuts`, not as part of the item's name.

### Limits

- On Android, a menu opened without a tap (on mount or from a timer) may let
  Back close the app.

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
