use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Filters;

#[component]
pub fn UseFocusReturnPage() -> Element {
    rsx! {
        DocPage {
            title: "Focus return",
            source: "libero/src/hooks/focus_return.rs",
            markdown: "/md/use_focus_return.md",
            accessibility: a11y()
                .handles([
                    "`restore()` puts focus back on the remembered element. In the demo, Tab into the panel, then press Apply or Escape, and focus lands on Filters again. `Collapse` shows the same return on an animated panel.",
                ])
                .example("A filters panel of your own: the Filters button calls `remember_active()` as it opens the panel, and Apply calls `restore()`, so a keyboard user lands back on Filters instead of the top of the page.")
                .limits([
                    "Some browsers do not focus a button on a mouse click, so a panel opened with the mouse may remember the body. That only matters to a keyboard user, and for them the trigger has focus.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_focus_return() -> FocusReturn" }
                    " puts focus back where it came from once a panel or popup closes. "
                    "Without it, focus inside a closing panel drops to the document body "
                    "and a keyboard user loses their place. The overlays in libero do "
                    "this already; use the hook for a panel of your own. Call "
                    Code { source: "remember_active()" }
                    " in the handler that opens, on every open, and "
                    Code { source: "restore()" }
                    " wherever it closes. A webview, unlike the web and Blitz, has no "
                    "focused element to read, so name the trigger with "
                    Code { source: "remember(event)" }
                    " there."
                }
            },

            Demo {
                component: "Filters",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Filters {} },
                file: DemoFile(include_str!("use_focus_return/demo.rs")),
            }
        }
    }
}
