# Spotlight

Crate: `libero`
Import: `use libero::components::{SpotlightAction, SpotlightHandle, SpotlightOptions, spotlight_filter, use_spotlight};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/spotlight>
Index: [index.md](index.md) - every other component's markdown page
Description: A command palette - `use_spotlight` opens a modal search box over caller-supplied actions, with grouped rows, arrow-key highlight and a Ctrl/Cmd+K hotkey.

A command palette: a modal search box over a list of actions. `use_spotlight`
is a hook, like `use_modal` - it returns a `Copy` handle with `open`, `close`,
`toggle` and `is_open`. What it lists is yours: `actions` is called with the
live query and returns the rows, so a static list, a filtered one and search
results are the same prop. `spotlight_filter` is the common case: label hits
first, then description and keyword hits.

Rows with a `group` are drawn under its header, groups in the order they first
appear, and `limit` counts rows through the groups. A `shortcut` on an action is
a hint, drawn as a `Kbd`, and never bound.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{
    Button, Flex, SpotlightAction, SpotlightOptions, Text, spotlight_filter, use_spotlight,
};

fn actions(mut last: Signal<String>) -> Vec<SpotlightAction> {
    let run = move |name: &'static str| move |_| last.set(name.to_string());
    vec![
        SpotlightAction::new("Home").group("Pages").description("The start page").onclick(run("Home")),
        SpotlightAction::new("Changelog").group("Pages").keywords(["releases"]).onclick(run("Changelog")),
        SpotlightAction::new("New file").group("Commands").shortcut("Ctrl N").onclick(run("New file")),
    ]
}

#[component]
fn Demo() -> Element {
    let last = use_signal(|| String::from("nothing yet"));
    let all = use_hook(|| actions(last));
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| spotlight_filter(&query, &all))),
        ..Default::default()
    });

    rsx! {
        Flex { direction: "row", gap: "md",
            Button { variant: "outlined", onclick: move |_| spotlight.open(), "Open the palette" }
            Text { size: "sm", "Last run: {last()}" }
        }
    }
}
```

Call the hook in a component that outlives every trigger: the palette is
portaled from there, like a `use_modal` dialog.

The docs page's preview has three palettes: Commands (groups, descriptions,
keywords and shortcut hints), Files (a flat list with icons, a custom
`placeholder` and `nothing_found`) and 200 issues (a long list, for `limit`).
The controls set `shortcut`, `limit`, `close_on_action` and `clear_on_close`,
and the code block prints the palette opened last. The page binds J or P, not
K, because the docs site's own search owns Ctrl+K.

A flat list with icons and custom texts:

```rust
let files: Vec<SpotlightAction> = ["src", "src/main.rs", "Cargo.toml"]
    .into_iter()
    .map(|path| SpotlightAction::new(path).icon(rsx! { FileIcon {} }))
    .collect();
let spotlight = use_spotlight(SpotlightOptions {
    actions: Some(Callback::new(move |query: String| spotlight_filter(&query, &files))),
    placeholder: Some("Go to file...".into()),
    nothing_found: Some(rsx! { "No file by that name." }),
    shortcut: Some('p'),
    ..Default::default()
});
```

## Opening it

Ctrl + K (Cmd + K on a Mac) toggles the palette from anywhere on the page. Change
the key with `shortcut: Some('p')`, or turn it off with `None`. The chord is
ignored while focus is in another text field and while a dialog or popover is
already open, and a held chord toggles once. It needs a document-level key
listener, which only the web has today; elsewhere, open the palette from a
button. Two palettes on one page should not share a key.

Results from a search index are the same prop: return a signal's contents from
`actions` instead of calling `spotlight_filter`, and the palette redraws when
the signal fills.

## Keyboard and accessibility

- The search box is `role="combobox"` (`aria-expanded`, `aria-controls`,
  `aria-haspopup="listbox"`) over a `role="listbox"`. Focus stays in it the
  whole time.
- ArrowDown / ArrowUp move a highlight named by `aria-activedescendant`,
  wrapping at both ends. Home and End are left to the caret.
- Enter runs the highlighted action. Typing clears the highlight, so Enter
  never runs a row the user did not look at.
- Escape or a click outside closes; focus returns to what opened it.
- Groups are `role="group"`, named by their visible header. No row is ever
  `aria-selected`.
- "Nothing found" is announced through a `role="status"` region that is present
  from the start.

## API

```rust
pub fn use_spotlight(options: SpotlightOptions) -> SpotlightHandle;
pub fn spotlight_filter(query: &str, actions: &[SpotlightAction]) -> Vec<SpotlightAction>;
```

| `SpotlightOptions` field | Type | Default | Description |
|---|---|---|---|
| `actions` | `Option<Callback<String, Vec<SpotlightAction>>>` | `None` (warns) | Called with the live query, returns the rows. Capture a `Signal` if the list changes. |
| `placeholder` | `Option<String>` | theme (`"Search..."`) | The search box placeholder. |
| `nothing_found` | `Option<Element>` | theme text | Shown when a non-empty query matches nothing. |
| `limit` | `Option<usize>` | `None` | Cap on rows, counted through groups. |
| `close_on_action` | `bool` | `true` | Close after running an action. |
| `clear_on_close` | `bool` | `true` | Start each opening with an empty query. |
| `aria_label` | `Option<String>` | theme (`"Command palette"`) | Names the dialog. |
| `shortcut` | `Option<char>` | `Some('k')` | Ctrl/Cmd + this key toggles the palette. |

| `SpotlightHandle` method | Returns | Description |
|---|---|---|
| `open()` | `()` | Opens with focus in the search box. Call it from the trigger's handler so focus returns there. |
| `close()` | `()` | Closes. |
| `toggle()` | `()` | Opens or closes. |
| `is_open()` | `bool` | Whether it is open. |

`SpotlightAction` fields (all `pub`, with builder methods of the same names):
`label: String`, `description: Option<String>`, `keywords: Vec<String>`
(matched, never drawn), `group: Option<String>`, `icon: Option<Element>`,
`shortcut: Option<String>` (a hint), `onclick: Option<Callback<()>>`.

## Theme defaults

`theme.spotlight: SpotlightDefaults` - `width` (`"600px"`), `top_offset`
(`"80px"`), `max_list_height` (`"400px"`), `radius` (`Md`), `padding` (`"4px"`),
`search_font_size` (`"1.125rem"`), `group_color` and `description_color`
(`grey.7`), `labels: SpotlightLabels` (`label`, `placeholder`, `nothing_found`;
`SpotlightLabels::ENGLISH`).

## CSS variables

`--lsx-spotlight-width`, `--lsx-spotlight-top-offset`,
`--lsx-spotlight-max-list-height`, `--lsx-spotlight-padding`,
`--lsx-spotlight-search-font-size`, `--lsx-spotlight-group-color`,
`--lsx-spotlight-description-color`.

## Data attributes

The highlighted row carries `data-active`.
