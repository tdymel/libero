//! `RichTextEditor`, controlled through a parent that echoes each change a task later;
//! `#out` shows the doc as Markdown, `#changes` counts `onchange` calls.

use dioxus::prelude::*;
use libero::components::RichTextEditor;
use libero::components::rich_text::Doc;

use crate::Routes;

pub const ROUTES: Routes = &[("/rich-text-editor", || rsx! { RichTextEditorPage {} })];

#[component]
fn RichTextEditorPage() -> Element {
    let mut doc = use_signal(Doc::new);
    let mut changes = use_signal(|| 0u32);

    rsx! {
        RichTextEditor {
            id: "editor-field",
            label: "Notes",
            placeholder: "Write something",
            value: doc(),
            onchange: move |next: Doc| {
                changes += 1;
                // A late echo, as an async parent would send it.
                spawn(async move { doc.set(next) });
            },
        }
        pre { id: "out", {doc.read().to_markdown()} }
        output { id: "changes", "{changes}" }
        button { id: "replace", onclick: move |_| doc.set(Doc::new()), "Replace" }
    }
}
