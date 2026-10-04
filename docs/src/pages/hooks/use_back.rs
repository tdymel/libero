use crate::components::{Demo, DemoValues, DocPage, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Text},
    hooks::use_back,
};

/// The hook in one component, as `Steps` renders it.
// snippet: mirrors Steps
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut step = use_signal(|| 1);
// On Android, Back goes one step back while there is one; at the first, it leaves the app.
use_back(step() > 1, Callback::new(move |()| step -= 1));

rsx! {
    Flex { direction: "column", gap: "sm", align: "flex-start",
        Text { "Step {step} of 3" }
        Flex { gap: "sm",
            Button { variant: "outlined", disabled: step() == 1, onclick: move |_| step -= 1, "Previous" }
            Button { disabled: step() == 3, onclick: move |_| step += 1, "Next" }
        }
    }
}"#
    .to_string()
}

#[component]
fn Steps() -> Element {
    let mut step = use_signal(|| 1);
    use_back(step() > 1, Callback::new(move |()| step -= 1));
    rsx! {
        Flex { direction: "column", gap: "sm", align: "flex-start",
            Text { "Step {step} of 3" }
            Flex { gap: "sm",
                Button {
                    variant: "outlined",
                    disabled: step() == 1,
                    onclick: move |_| step -= 1,
                    "Previous"
                }
                Button { disabled: step() == 3, onclick: move |_| step += 1, "Next" }
            }
        }
    }
}

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
                .limits([
                    "Android only. Elsewhere the hook does nothing.",
                    "Activated without a tap (on mount, from a timer), the first Back may still leave the app: Android's WebView skips a history entry pushed without a user gesture.",
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
                component: "use_back",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Steps {} },
                wrap: Wrap(code),
            }
        }
    }
}
