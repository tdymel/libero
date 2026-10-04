use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Text},
    hooks::{use_element, use_fullscreen},
};

/// The hook in one component, as `Panel` renders it.
// snippet: mirrors Panel
fn code(_: &DemoValues, _: &str) -> String {
    r#"let panel = use_element();
let fullscreen = use_fullscreen(panel);
// Drawn where the platform refuses native fullscreen: a fixed box over the page.
let drawn = if fullscreen.is_drawn() { "position: fixed; inset: 0; z-index: 1000;" } else { "" };

rsx! {
    div {
        onmounted: panel.mount(),
        style: "{drawn} display: flex; flex-direction: column; gap: 8px; padding: 16px; background: Canvas; color: CanvasText; border: 1px solid GrayText; border-radius: 8px;",
        ..fullscreen.attributes(),
        Text { "A chart, a map or a slide deck." }
        Button {
            variant: "outlined",
            onclick: move |_| fullscreen.toggle(),
            if fullscreen.is_fullscreen() { "Exit fullscreen" } else { "Fullscreen" }
        }
    }
}"#
    .to_string()
}

#[component]
fn Panel() -> Element {
    let panel = use_element();
    let fullscreen = use_fullscreen(panel);
    let drawn = if fullscreen.is_drawn() {
        "position: fixed; inset: 0; z-index: 1000;"
    } else {
        ""
    };
    rsx! {
        div {
            onmounted: panel.mount(),
            style: "{drawn} display: flex; flex-direction: column; gap: 8px; padding: 16px; background: Canvas; color: CanvasText; border: 1px solid GrayText; border-radius: 8px;",
            ..fullscreen.attributes(),
            Text { "A chart, a map or a slide deck." }
            Button {
                variant: "outlined",
                onclick: move |_| fullscreen.toggle(),
                if fullscreen.is_fullscreen() { "Exit fullscreen" } else { "Fullscreen" }
            }
        }
    }
}

#[component]
pub fn UseFullscreenPage() -> Element {
    rsx! {
        DocPage {
            title: "Fullscreen",
            source: "libero/src/hooks/fullscreen.rs",
            markdown: "/md/use_fullscreen.md",
            accessibility: a11y()
                .handles([
                    "Escape leaves either fullscreen: the browser leaves its native one, the handle the drawn one and an Android WebView's native one.",
                    "Focus leaving the element leaves the drawn fullscreen, so the covered page is never focused unseen.",
                    "Android's Back button leaves either fullscreen rather than the app.",
                ])
                .must([
                    "Give the toggle a name that says what it does now, such as Fullscreen or Exit fullscreen.",
                    "Keep a visible way out inside the element: a touch screen has no Escape.",
                ])
                .example("A video panel with a button whose text follows `is_fullscreen()`, Fullscreen or Exit fullscreen: Escape leaves fullscreen, and the button stays inside the panel for a touch screen.")
                .limits([
                    "Blitz and a server render have no Fullscreen API: the handle always draws it.",
                    "The drawn fullscreen keeps the browser's bars and the device's status bar.",
                    "On Android, a fullscreen entered without a tap (from a timer, say) may let Back close the app.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_fullscreen(element) -> FullscreenHandle" }
                    " puts one of your elements in fullscreen. Spread its "
                    Code { source: "attributes()" }
                    " on the element and mount it with "
                    Code { source: "use_element" }
                    ". Where the page refuses the Fullscreen API (a headless browser, an "
                    "iframe without "
                    Code { source: "allowfullscreen" }
                    "), the handle draws it: it sets "
                    Code { source: "data-fullscreen=\"drawn\"" }
                    " and your CSS turns the element into a fixed box over the page. "
                    Code { source: "Video" }
                    " is built on it."
                }
            },

            Demo {
                component: "use_fullscreen",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Panel {} },
                wrap: Wrap(code),
            }
        }
    }
}
