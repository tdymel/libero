use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Button, Code, Flex, Paper, Text, Transition, TransitionKind};

const KINDS: [&str; 9] = [
    "Fade",
    "FadeUp",
    "FadeDown",
    "Scale",
    "SlideUp",
    "SlideDown",
    "SlideLeft",
    "SlideRight",
    "Pop",
];

fn kind_of(name: &str) -> TransitionKind {
    match name {
        "FadeUp" => TransitionKind::FadeUp,
        "FadeDown" => TransitionKind::FadeDown,
        "Scale" => TransitionKind::Scale,
        "SlideUp" => TransitionKind::SlideUp,
        "SlideDown" => TransitionKind::SlideDown,
        "SlideLeft" => TransitionKind::SlideLeft,
        "SlideRight" => TransitionKind::SlideRight,
        "Pop" => TransitionKind::Pop,
        _ => TransitionKind::Fade,
    }
}

/// Prints the trigger and its `show` signal: `open` is the caller's, and the paste needs it to compile.
fn wrap_page(_values: &DemoValues, source: &str) -> String {
    let transition = indent(&indent(source));
    format!(
        "let mut show = use_signal(|| false);\n\n\
         rsx! {{\n    \
         Flex {{\n        \
         direction: \"column\",\n        \
         align: \"flex-start\",\n        \
         gap: \"sm\",\n        \
         Button {{\n            \
         onclick: move |_| show.toggle(),\n            \
         \"Toggle\"\n        \
         }}\n\
         {transition}    }}\n}}"
    )
}

/// Its own component: a hook in `render` would land in `Demo`'s scope.
/// **Kept in step with `wrap_page` by hand.**
#[component]
fn TransitionDemo(kind: TransitionKind, duration: Option<u32>) -> Element {
    let mut show = use_signal(|| false);

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Button { onclick: move |_| show.toggle(), "Toggle" }
            Transition { kind, duration, open: show(),
                Paper { Text { "Hello" } }
            }
        }
    }
}

#[component]
pub fn TransitionPage() -> Element {
    rsx! {
        DocPage {
            title: "Transition",
            source: "libero/src/components/layout/transition.rs",
            markdown: "/md/transition.md",
            properties: vec![props("Transition", vec![
                prop("kind", "TransitionKind")
                    .default("Fade")
                    .doc("How the children move in and out: `Fade`, `FadeUp`, `FadeDown`, `Scale`, `SlideUp`, `SlideDown`, `SlideLeft`, `SlideRight` or `Pop`. All of them also fade. The slides travel the children's own size, in physical directions."),
                prop("open", "bool")
                    .default("true")
                    .doc("Omitted, the children animate in once on mount and never exit. Passed, they enter and exit as it flips; the first value does not animate. You own this state."),
                prop("duration", "u32")
                    .default("200")
                    .doc("Animation length in milliseconds. `0` turns the animation off."),
                prop("children", "Element").doc("The content that animates. It is unmounted once the exit ends."),
            ])],
            accessibility: a11y()
                .handles([
                    "Closed children are hidden from the focus order and screen readers once the exit ends, and removed from the DOM.",
                    "Under reduced motion the children switch instantly.",
                ])
                .must([
                    "Move focus yourself when focused content exits: use `use_focus_return`.",
                    "Do not hide content that must be announced behind an omitted `open` on the server: it renders in its from-state until the page hydrates.",
                ]),
            lead: rsx! {
                Text {
                    "Animates its children in on mount, and out when "
                    Code { source: "open" }
                    " turns false. Leave "
                    Code { source: "open" }
                    " out for a one-time entrance. It renders a wrapper "
                    Code { source: "div" }
                    "; the motion is a CSS transition, so a native renderer without transitions swaps instantly."
                }
            },
            Demo {
                component: "Transition",
                children_text: "",
                code_child: Child(|_| "Paper { Text { \"Hello\" } }".to_string()),
                fixed: vec!["open: show()".to_string()],
                controls: vec![
                    Control::select("kind", KINDS).code(|control, values| {
                        let value = values.str("kind");
                        match value == control.default {
                            true => vec![],
                            false => vec![format!("kind: TransitionKind::{value}")],
                        }
                    }),
                    Control::slider("duration", ["0", "100", "200", "600", "1200"])
                        .default("200")
                        .labels(["0ms", "100ms", "200ms", "600ms", "1200ms"])
                        .code(|control, values| {
                            let value = values.str("duration");
                            match value == control.default {
                                true => vec![],
                                false => vec![format!("duration: {value}")],
                            }
                        }),
                ],
                wrap: Wrap(wrap_page),
                render: move |values: DemoValues| {
                    let duration = match values.str("duration").as_str() {
                        "200" => None,
                        value => value.parse::<u32>().ok(),
                    };

                    rsx! {
                        TransitionDemo { kind: kind_of(&values.str("kind")), duration }
                    }
                },
            }
        }
    }
}
