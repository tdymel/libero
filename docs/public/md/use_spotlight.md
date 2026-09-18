# use_spotlight

Crate: `libero`
Import: `use libero::components::use_spotlight;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/spotlight/spotlight.rs>
Index: [index.md](index.md) lists every other page
Description: Registers a command palette and its hotkey and returns the handle that opens it.

`use_spotlight(options) -> SpotlightHandle` registers a command palette and
returns the handle that opens it. By default Ctrl K, or Cmd K on a Mac, toggles
it from anywhere on the page. [Spotlight](spotlight.md) covers actions, groups,
search and loading.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{
    Button, Flex, SpotlightAction, SpotlightOptions, Text, spotlight_filter, use_spotlight,
};

#[component]
fn Palette() -> Element {
    let mut last = use_signal(|| String::from("nothing yet"));
    let all = use_hook(|| {
        let run = move |name: &'static str| move |_| last.set(name.to_string());
        vec![
            SpotlightAction::new("New file").onclick(run("New file")),
            SpotlightAction::new("Open recent").onclick(run("Open recent")),
            SpotlightAction::new("Settings").onclick(run("Settings")),
        ]
    });
    let spotlight = use_spotlight(SpotlightOptions {
        actions: Some(Callback::new(move |query: String| spotlight_filter(&query, &all))),
        aria_label: Some("Commands".into()),
        // No hotkey: the docs page's own search owns Ctrl K.
        shortcut: None,
        ..Default::default()
    });

    rsx! {
        Flex { direction: "row", gap: "md",
            Button { variant: "outlined", onclick: move |_| spotlight.open(), "Commands" }
            Text { "Last run: {last}" }
        }
    }
}
```

`actions` is called with the live query and returns what to show.
`spotlight_filter` is the common matcher. Open it from the handler of what the user acted on,
so focus goes back there on close.

## Web and native

The hotkey is web only. No other renderer has a document-level key listener
yet, so give the palette a visible trigger as well.

## API

```rust,ignore
pub fn use_spotlight(options: SpotlightOptions) -> SpotlightHandle
```

`SpotlightHandle` has `open()`, `close()`, `toggle()` and `is_open()`, and is
`Copy`.
`SpotlightOptions` is listed on [Spotlight](spotlight.md).
