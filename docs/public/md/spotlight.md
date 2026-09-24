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

For results from a server, return a signal's contents from `actions`, start the
search from `onquery` and set `loading` until it answers.

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
| `shortcut` | `Option<char>` | `Some('k')` | Ctrl (Cmd on a Mac) plus this key toggles the palette. `None` for no hotkey. Web only. A key the browser already uses, such as L, T or W, warns in a debug build. Elsewhere, open the palette from a button. Two palettes on one page should not share a key. |
| `highlight_first_on_query` | `bool` | `true` | Highlights the first row after every keystroke, so Enter runs it. Off, Enter does nothing until the arrows pick a row. |
| `loading` | `bool` | `false` | The results are still coming. A loader replaces the rows, and a screen reader hears "Searching". |
| `onquery` | `Callback<String>` | - | Called with the query on every keystroke. Set `loading` and start the search here. |
| `sx` | `Sx` | - | Styles the dialog box. |
| `parts` | `Parts<SpotlightPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

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

## Style API

Style a part with `SpotlightOptions::parts`, or address it as `[data-slot='…']`
in your own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `SpotlightPart::Body` | `body` | Holds the search box, the list and the status line. |
| `SpotlightPart::Search` | `search` | The search box. |
| `SpotlightPart::List` | `list` | The `listbox`, a `ScrollArea`. |
| `SpotlightPart::Group` | `group` | A named group of rows. |
| `SpotlightPart::GroupLabel` | `group-label` | A group's visible name. |
| `SpotlightPart::Option` | `option` | One action row. The highlighted one has `data-active`. |
| `SpotlightPart::Icon` | `icon` | A row's icon. |
| `SpotlightPart::Text` | `text` | The column holding the label and the description. |
| `SpotlightPart::Label` | `label` | A row's label. |
| `SpotlightPart::Description` | `description` | A row's second line. |
| `SpotlightPart::Shortcut` | `shortcut` | A row's key hint, one `Kbd` per key. |
| `SpotlightPart::Status` | `status` | The status line: "nothing found" or the loader. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Down` or `Up` | Moves the highlight, wrapping at both ends. |
| `Enter` | Runs the highlighted action, by default the first row. |
| `Escape` | Closes, as a click outside does. Focus goes back to what opened it. |

### Libero handles

- Focus stays in the search box.
- The hotkey is ignored while you type in another text field, and while a
  dialog or popover is open.

### You must

- Turn `highlight_first_on_query` off for a palette whose actions change
  things. Then nothing is highlighted until you press Down.
- Call `open()` from the trigger's handler, so focus returns there.

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

The highlighted row carries `data-active`, and every part its `data-slot`.
