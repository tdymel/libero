use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Steps;

#[component]
pub fn UseBackPage() -> Element {
    rsx! {
        DocPage {
            title: "Back button",
            source: "libero/src/hooks/dismiss.rs",
            markdown: "/md/use_back.md",
            accessibility: a11y()
                .handles([
                    "A `Modal`, `Drawer`, menu, popover, open field list or fullscreen opened after your hook takes Back first, as Escape closes the newest layer first.",
                ])
                .must([
                    "Keep a visible control for the same step, such as a Previous button: web, desktop and keyboard users have no Back button.",
                ])
                .example("A three-step wizard with `use_back(step() > 0, ..)` going one step back: on Android Back returns to the previous step, and a visible Previous button does the same for everyone else.")
                .limits([
                    "Android only. Elsewhere the hook does nothing.",
                    "Activated without a tap (on mount, from a timer), the first Back may still leave the app: Android's WebView skips a history entry pushed without a user gesture.",
                    "A Back that leaves the hook active (one wizard step back) needs a tap or key press before the next Back reaches it; two Backs in a row leave the app.",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_back(active, onback)" }
                    " runs "
                    Code { source: "onback" }
                    " when Android's Back button is pressed, instead of leaving the app, "
                    "while "
                    Code { source: "active" }
                    " is true. Use it to step back through a wizard, undo, or leave an "
                    "editing mode. The newest active hook or overlay takes the press. On the "
                    "web and the desktop it does nothing: try the demo in the Android app."
                }
            },

            Demo {
                component: "Steps",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Steps {} },
                file: DemoFile(include_str!("use_back/demo.rs")),
            }
        }
    }
}
