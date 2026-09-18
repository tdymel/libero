# Spotlight

Crate: `libero`
Import: `use libero::components::{SpotlightAction, SpotlightHandle, SpotlightOptions, spotlight_filter, use_spotlight};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/spotlight>
Index: [index.md](index.md) lists every other page
Description: A command palette. A modal search box over your actions, with groups, arrow-key highlight and a Ctrl/Cmd+K hotkey.

A command palette, a modal search box over a list of actions. `use_spotlight`
returns a `Copy` handle, like `use_modal`. `actions` is called with the query
and returns the rows, so a fixed list and search results are the same prop.
`spotlight_filter` covers the common case, with label hits first.

Rows with a `group` are drawn under its header, groups in the order they first
appear. A `shortcut` on an action is a hint, drawn as one `Kbd` per key, and
never bound.

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

Call the hook in a component that outlives every trigger, as with `use_modal`.

A flat list with icons and custom texts.

```rust,ignore
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

Ctrl + K (Cmd + K on a Mac) toggles the palette from anywhere on the page.
Change the key with `shortcut: Some('p')`, or turn it off with `None`. The
hotkey is ignored while focus is in a text field and while a dialog or popover
is open. A key the browser already uses, such as L, T or W, warns in a debug
build. Only the web has the hotkey. Elsewhere, open the palette from a button.
Two palettes on one page should not share a key.

## Slow search

For results from a server, return a signal's contents from `actions`. Start the
search from `onquery` and set `loading` until it answers. A loader then
replaces the rows, and "Nothing found" never flashes before the answer.

```rust,ignore
let mut results = use_signal(Vec::<SpotlightAction>::new);
let mut loading = use_signal(|| false);
let spotlight = use_spotlight(SpotlightOptions {
    actions: Some(Callback::new(move |query: String| match query.trim().is_empty() {
        true => vec![],
        false => results(),
    })),
    loading: loading(),
    onquery: Some(Callback::new(move |query: String| {
        loading.set(true);
        spawn(async move {
            results.set(search_on_server(&query).await);
            loading.set(false);
        });
    })),
    ..Default::default()
});
```

## Accessibility

Focus stays in the search box. ArrowDown and ArrowUp move the highlight,
wrapping at both ends. Enter runs the highlighted action, by default the first
row. Turn `highlight_first_on_query` off for a palette whose actions change
things. Then nothing is highlighted until you press ArrowDown. Escape or a
click outside closes, and focus goes back to what opened it.

## API

```rust,ignore
pub fn use_spotlight(options: SpotlightOptions) -> SpotlightHandle;
pub fn spotlight_filter(query: &str, actions: &[SpotlightAction]) -> Vec<SpotlightAction>;
```

### SpotlightOptions

| Prop | Type | Default | Description |
|---|---|---|---|
| `actions` | `Callback<String, Vec<SpotlightAction>>` | - | Called with the query, returns the rows. Capture a `Signal`, not a `Vec`, if the list changes. Unset warns and shows nothing. |
| `placeholder` | `String` | `"Search..."` | The search box's placeholder. |
| `nothing_found` | `Element` | - | Shown and announced when a query matches nothing. Unset, the localization's text. |
| `limit` | `usize` | - | The most rows drawn, counted across groups. |
| `close_on_action` | `bool` | `true` | Closes after running an action. |
| `clear_on_close` | `bool` | `true` | Starts every opening with an empty query. |
| `aria_label` | `String` | `"Command palette"` | Names the dialog and its list. |
| `shortcut` | `Option<char>` | `Some('k')` | Ctrl (Cmd on a Mac) plus this key toggles the palette. `None` for no hotkey. Web only. A key the browser already uses, such as L, T or W, warns in a debug build. |
| `highlight_first_on_query` | `bool` | `true` | Highlights the first row after every keystroke, so Enter runs it. Off, Enter does nothing until the arrows pick a row. |
| `loading` | `bool` | `false` | The results are still coming. A loader replaces the rows, and a screen reader hears "Searching". |
| `onquery` | `Callback<String>` | - | Called with the query on every keystroke. Set `loading` and start the search here. |

### SpotlightAction

Every field has a builder method of the same name.

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `String` | required | The row's text, and the first thing `spotlight_filter` matches. |
| `description` | `String` | - | A second line, matched after the label. |
| `keywords` | `Vec<String>` | - | Matched, never drawn. |
| `group` | `String` | - | A section header. Groups keep the order they first appear in. |
| `icon` | `Element` | - | Drawn before the label. |
| `shortcut` | `String` | - | A hint, one `Kbd` per key, split on spaces and `+`. Never bound. |
| `onclick` | `Callback<()>` | - | Runs on Enter or a click. |

### SpotlightHandle

| Method | Returns | Description |
|---|---|---|
| `open()` | `()` | Opens with focus in the search box. Call it from the trigger's handler, so focus returns there. |
| `close()` | `()` | Closes. |
| `toggle()` | `()` | Opens or closes. |
| `is_open()` | `bool` | Whether it is open. |

## Theme defaults

`SpotlightDefaults` on the theme holds `width` (`"600px"`), `top_offset`
(`"80px"`), `max_list_height` (`"400px"`), `radius` (`Md`), `padding`
(`"4px"`), `search_font_size` (`"1.125rem"`), `group_color` and
`description_color` (`muted.7`). The words are `SpotlightLabels` in the
[localization](localization.md).

## CSS variables

`--lsx-spotlight-width`, `--lsx-spotlight-top-offset`,
`--lsx-spotlight-max-list-height`, `--lsx-spotlight-padding`,
`--lsx-spotlight-search-font-size`, `--lsx-spotlight-group-color` and
`--lsx-spotlight-description-color`.

## Data attributes

The highlighted row carries `data-active`.
