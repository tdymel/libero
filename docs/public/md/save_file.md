# Save file

Crate: `libero`
Import: `use libero::platform::{SaveOutcome, save_file};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/platform/save_file.rs>
Index: [index.md](index.md) lists every other page
Description: Saves bytes the app made as a file: a download on the web, a save dialog on the desktop and Blitz, the share sheet on Android.

`save_file(name, mime, bytes).await` saves bytes your app made, an export or a
report, as a file the user keeps. The web downloads it, the desktop and Blitz
open a save dialog, and Android opens the share sheet for text. It returns a
`SaveOutcome`: `Saved`, `Shared`, `Cancelled` or `Failed(reason)`.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, Text},
    platform::{SaveOutcome, save_file},
};

#[component]
fn SaveDemo() -> Element {
    let mut outcome = use_signal(|| None::<SaveOutcome>);

    rsx! {
        Flex { gap: "sm", align: "center",
            Button {
                onclick: move |_| async move {
                    let csv = "name,role\nAda,Engineer\n";
                    let saved = save_file("team.csv", "text/csv", csv.into()).await;
                    outcome.set(Some(saved));
                },
                "Save team.csv"
            }
            if let Some(outcome) = outcome() {
                Text { size: "sm", "{outcome:?}" }
            }
        }
    }
}
```

`TableExportButton`'s `onexport` hands over the CSV to save the same way; see
[Table](table.md).

## API

```rust,ignore
pub async fn save_file(name: &str, mime: &str, bytes: Vec<u8>) -> SaveOutcome

pub enum SaveOutcome {
    Saved,
    Shared,
    Cancelled,
    Failed(String),
}
```

| Platform | Support |
|---|---|
| Web | A download named `name`, typed `mime`; `Saved` once the browser has it. Checked by the e2e suite. |
| Desktop (WebView), Blitz | The system save dialog, `name` filled in; `Saved`, or `Cancelled` when closed. |
| Android (WebView) | The share sheet with the bytes as text, `name` as its title; `Shared`. Bytes that are not UTF-8, or over about 500 KB, fail. |
| Liveview, fullstack | The browser downloads it through the page. |
| Server render, no page | `Failed("not supported on this platform")`. |

## Accessibility

### Libero handles

- The save dialog and the share sheet are the system's own, with its keyboard
  and screen reader support.

### You must

- Say what happened: the browser's download shows no dialog, so name the file
  in your button or confirm the save in text, as `TableExportButton` announces
  its rows.

### Example

An "Export report.csv" button that calls the save: the file name is in the
button's text, and a status line says "Saved report.csv" once it reports
`Saved`, since a browser download shows no dialog.

### Limits

- Android shares text only, under about 500 KB; other bytes come back `Failed`.
- The web cannot tell a finished download from a blocked one: it reports
  `Saved` once the browser has the file.
