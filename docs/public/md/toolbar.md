# Toolbar

Crate: `libero`
Import: `use libero::components::{Toolbar, ToolbarGroup, ToolbarSeparator};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/toolbar.rs>
Index: [index.md](index.md) lists every other page
Description: A row of buttons, action icons and selects that is one tab stop, moved through with the arrow keys.

A row of controls that is one tab stop: the arrow keys move between them, Home
and End jump to the ends. An editor's formatting bar is the typical use.

Put `Button`s, `ActionIcon`s, `ButtonGroup`s and `Select`s inside, sectioned by
`ToolbarGroup` and `ToolbarSeparator`. For buttons that stay separate tab stops,
use `ButtonGroup` alone.

## Usage

`BoldIcon` and its siblings stand for any component of yours that renders an `svg`.

```rust
use dioxus::prelude::*;
use libero::components::{ActionIcon, Toolbar, ToolbarGroup, ToolbarSeparator};

#[component]
fn Demo() -> Element {
    rsx! {
        Toolbar { "aria-label": "Formatting",
            ToolbarGroup { "aria-label": "Style",
                ActionIcon { aria_label: "Bold", BoldIcon {} }
                ActionIcon { aria_label: "Italic", ItalicIcon {} }
            }
            ToolbarSeparator {}
            ToolbarGroup { "aria-label": "History",
                ActionIcon { aria_label: "Undo", UndoIcon {} }
                ActionIcon { aria_label: "Redo", disabled: true, RedoIcon {} }
            }
        }
    }
}
#
# #[component] fn BoldIcon() -> Element { rsx! {} }
# #[component] fn ItalicIcon() -> Element { rsx! {} }
# #[component] fn UndoIcon() -> Element { rsx! {} }
# #[component] fn RedoIcon() -> Element { rsx! {} }
```

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ToolbarPart::Group` | `group` | A `ToolbarGroup`, laid out along the bar. |
| `ToolbarPart::Separator` | `separator` | A `ToolbarSeparator`, a 1px line across the bar. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Enters the bar on the control focused last, the first one at the start, and leaves it. |
| `Left` or `Right` | Moves to the previous or next control, disabled ones included. Swapped under right-to-left text. Up and Down in a vertical bar. |
| `Home` or `End` | Goes to the first or last control. |
| `Alt` + `F10` | With `focus_from`, moves from that element to the bar's tab stop. |
| `Escape` | After Alt+F10, hands focus back to where it was, until focus leaves the bar; a menu or dialog the bar opened does not count. |

### Libero handles

- The root is a `role="toolbar"`, with `aria-orientation="vertical"` when
  vertical.
- The bar is one tab stop: the controls inside get a roving `tabindex`.
- A disabled `Button`, `ActionIcon` or `Select` inside stays focusable with
  `aria-disabled`, so a keyboard user finds it; `focusable_when_disabled: false`
  opts out. A disabled field stays focusable too, read-only with
  `aria-disabled`.
- A key a control uses itself stays its own: `Select` opens on Home, End and Up,
  a slider keeps its arrows.
- `Checkbox`, `Switch`, `SegmentedControl`, `TextField` and `NumberField` join
  the arrow order. A `SegmentedControl` moves through its segments and passes
  the arrow on past its last one.
- A text field keeps Left and Right until the caret sits at its start or end
  with nothing selected; then the arrow moves on. Home and End stay the
  field's.
- `NumberField`'s Up and Down stay its stepper, in a vertical bar too; its
  stepper buttons stay out of the arrow order.
- `ToolbarGroup` is a `role="group"`; `ToolbarSeparator` a `role="separator"`
  across the bar.

### You must

- Name the bar with `aria-label`, or `aria-labelledby` on a visible heading.
  Without either it warns in debug builds.
- Name each `ToolbarGroup` with `aria-label`. Without it the group warns in
  debug builds too.

### Limits

- Only `Button`, `ActionIcon`, `Select`, `Checkbox`, `Switch`,
  `SegmentedControl`, `TextField` and `NumberField` (and what is built on them)
  join the arrow order. Another focusable element inside stays its own tab
  stop.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `horizontal` | `"vertical"` stacks the controls; Up and Down move instead of Left and Right. |
| `loop_focus` | `bool` | `true` | Whether the arrow keys wrap at the ends. |
| `focus_from` | `Option<ElementHandle>` | - | The element the bar serves, such as an editor: Alt+F10 inside it moves focus to the bar, Escape in the bar hands it back. Spread its `attributes()`. |
| `parts` | `Parts<ToolbarPart>` | - | Styles for the groups and separators, under `sx`. |
| `children` | `Element` | `required` | `Button`s, `ActionIcon`s, `Select`s, `ButtonGroup`s, fields, `ToolbarGroup`s and `ToolbarSeparator`s. |

Like every component, `Toolbar` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

### `ToolbarGroup`

| Prop | Type | Default | Description |
|---|---|---|---|
| `children` | `Element` | `required` | The section's controls. They stay in the bar's arrow order. |

`ToolbarGroup` and `ToolbarSeparator` take the shared props too.

## CSS variables

None of its own; the controls inside keep theirs.

## Data attributes

`horizontal` or `vertical` on the root's `data-state`. Each control in the
arrow order carries `data-toolbar-item`.
