# Combobox

Crate: `libero`
Import: `use libero::components::{Combobox, ComboboxTarget};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/inputs/combobox>
Index: [index.md](index.md) - every other component's markdown page
Description: A listbox over an enum with its own search field inside the dropdown, and a caller-supplied control.

A listbox over an enum, with its own search field inside the dropdown. The
control is the caller's, through `target` - so the selected option is displayed
exactly as the caller draws it, and `Combobox` never owns a field's styling.
Rows are virtualized, so a list of thousands costs the same as a list of ten.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Combobox, ComboboxTarget, Options};

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

#[component]
fn Demo() -> Element {
    let mut fruit = use_signal(|| None::<Fruit>);

    rsx! {
        Combobox {
            value: fruit(),
            onchange: move |next| fruit.set(next),
            search_placeholder: "Search fruit",
            target: move |t: ComboboxTarget| rsx! {
                Button {
                    variant: "outlined",
                    full_width: true,
                    onclick: move |event| t.onclick.call(event),
                    onkeydown: move |event| t.onkeydown.call(event),
                    attributes: t.aria.clone(),
                    match t.labels.first() {
                        Some(label) => label.render(),
                        None => rsx! { "Pick a fruit" },
                    }
                }
            },
        }
    }
}
```

`target` is required and has no default. That is deliberate: a default would
drag `label`, `placeholder`, `size` and `radius` props back in to configure a
field `Combobox` does not render. The target can be a `Button`, a `TextField`,
or anything else - it just has to spread `t.aria` and forward `t.onclick` and
`t.onkeydown`.

## Search

The search field lives *inside* the dropdown, above the options, rather than
being the control itself. That is what lets the control always show the exact
selected option: there is no single `<input>` doing double duty as both the
value display and the query box.

`filter` decides what survives a query, and it is a function rather than a mode
because the thing to search by is not always the thing on screen - a `Person`
shown by name might be searched by email:

```rust
filter: move |f: ComboboxFilterArgs<Person>| f.value.email.contains(&f.query),
```

The default is a case-insensitive contains on `f.name` - the label
`option_label` already resolved, so a translated option is searched by what it
actually reads as.

## Rich options

`option_label` returns an `OptionLabel`, not a `String`, so a row can hold an
icon or a badge. `OptionLabel::rich(name, rsx! { .. })` takes the accessible
name and the drawing apart, and the same rich label is what the target receives
in `t.labels` - a selected option is displayed the same way it was listed.

A row's height is a `min-height`, so a taller custom row grows rather than being
clipped. Virtualization cannot measure it though - it is told the row pitch
rather than probing for it, which is what keeps a short list from flickering on
open - so a custom row that outgrows the themed height passes its real height
through `option_height`:

```rust
option_label: move |fruit: Fruit| OptionLabel::rich(fruit.label(), rsx! { .. }),
option_height: 56.0,
```

## What it does not do yet

Single selection only; `value` is an `Option<T>`. There is no creatable mode, no
option groups, no async/`loading` options, and no per-option disabling. The
dropdown is positioned with plain absolute placement below the control - it is
not portaled and it does not flip when it runs out of room below.

With `searchable: false` there is nothing to take focus inside the dropdown, so
clicking away does not close it; press Escape or click the control again.

## Accessibility

The target carries `role`-adjacent wiring through `t.aria`: the `id`,
`aria-haspopup="listbox"`, `aria-expanded`, and `aria-controls`. Opening moves
focus into the search field, which is the `role="combobox"` and owns
`aria-activedescendant` - so typing never fights the arrow keys for focus.
Closing puts focus back on the target.

Rows are `role="option"` with `aria-selected`, and they cancel `mousedown` so a
click does not blur the search field out from under itself.

Keyboard: Enter, Space and ArrowDown open from the target. Once open, the arrows
move the active row, Home and End jump to the ends, Enter picks, Escape closes,
and Tab closes without committing.

## Props

### `Combobox`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Option<T>` | - | The selected option; strictly controlled. |
| `onchange` | `EventHandler<Option<T>>` | - | Called with what should be selected next. `None` clears the selection. |
| `target` | `Callback<ComboboxTarget, Element>` | - | The whole control. Required. |
| `options` | `Vec<T>` | `T::options()` | The options to show. |
| `option_label` | `Callback<T, OptionLabel>` | `T::label()` | Overrides `Options::label`. |
| `searchable` | `bool` | `true` | A search field above the options. |
| `search_placeholder` | `String` | - | Placeholder for the search field. |
| `filter` | `Callback<ComboboxFilterArgs<T>, bool>` | contains on the label | Whether an option survives the query. |
| `empty` | `Element` | - | Shown in place of the list when nothing matches. |
| `size` | `Size` | `md` | Rows, the search field, and the row height virtualization assumes. |
| `radius` | `Size` | `sm` | The dropdown's corner radius. |
| `option_height` | `f64` | themed row height | A custom row's real height in px, for virtualization. |
| `max_dropdown_height` | `ThemeAwareValue` | `260px` | Height past which the option list scrolls. |
| `disabled` | `bool` | `false` | Blocks opening. |

`sx`, `class`, `states` and any extra HTML attributes land on the **dropdown**,
not on a wrapper: the element the dropdown hangs off is positioning scaffolding
rather than something to style.

### `ComboboxTarget`

| Field | Type | Description |
|---|---|---|
| `labels` | `Vec<OptionLabel>` | The selected options, richly. Empty when nothing is selected. |
| `opened` | `bool` | Whether the dropdown is open - for rotating a chevron. |
| `disabled` | `bool` | Mirrors the prop. |
| `aria` | `Vec<Attribute>` | Spread with `attributes: t.aria`. |
| `onclick` | `EventHandler<MouseEvent>` | Opens and closes. |
| `onkeydown` | `EventHandler<KeyboardEvent>` | Enter, Space and ArrowDown open. |
| `onclear` | `EventHandler<MouseEvent>` | Selects nothing - for a clear button. |

`labels` is a `Vec` rather than an `Option` so that multi-selection, when it
lands, changes nothing about the target's contract.

## Theme defaults

`ComboboxDefaults` on the theme; per-size values live in its `sizes` scale,
seeded with the numbers `SelectDefaults` uses.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted; `md`. |
| `radius` | `Size` | Default `radius` when the prop is omitted; `sm`. |
| `max_dropdown_height` | `&'static str` | Height past which the option list scrolls; `260px`. |
| `sizes` | `Sizes<ComboboxSizeLevel>` | `font_size`, `row_height`, `padding_x` per size. |

`row_height` is a number, not a CSS string, because it is also the `item_size`
handed to `Virtualize` - rows never have to probe their own height, so a short
list does not flicker on open. It is applied as a `min-height`, so a taller
custom row grows; tell virtualization about that height with `option_height`.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-combobox-font-size-<size>` | A row's `font-size` for that size step. |
| `--lsx-combobox-row-height-<size>` | A row's `height` for that size step. |
| `--lsx-combobox-padding-x-<size>` | A row's horizontal padding for that size step. |

`radius` reads the shared `--lsx-radius-<size>` scale rather than one of its own.

## Data attributes

| Token | Condition | Element |
|---|---|---|
| `size-<size>` | The `size` in effect. | Dropdown, rows |
| `radius-<size>` | The `radius` in effect. | Dropdown |
| `disabled` | `disabled` is set. | Dropdown |
| `active` | The row the arrows are on. | Rows |
| `selected` | The row is the selected option. | Rows |
