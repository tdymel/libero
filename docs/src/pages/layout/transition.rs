use crate::components::{
    Child, Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Text, TransitionKind};

mod demo;
use demo::TransitionDemo;

const KINDS: [&str; 21] = [
    "Fade",
    "FadeUp",
    "FadeDown",
    "FadeLeft",
    "FadeRight",
    "Scale",
    "ScaleX",
    "ScaleY",
    "SlideUp",
    "SlideDown",
    "SlideLeft",
    "SlideRight",
    "Pop",
    "PopTopLeft",
    "PopTopRight",
    "PopBottomLeft",
    "PopBottomRight",
    "SkewUp",
    "SkewDown",
    "RotateLeft",
    "RotateRight",
];

fn kind_of(name: &str) -> TransitionKind {
    match name {
        "FadeUp" => TransitionKind::FadeUp,
        "FadeDown" => TransitionKind::FadeDown,
        "FadeLeft" => TransitionKind::FadeLeft,
        "FadeRight" => TransitionKind::FadeRight,
        "Scale" => TransitionKind::Scale,
        "ScaleX" => TransitionKind::ScaleX,
        "ScaleY" => TransitionKind::ScaleY,
        "SlideUp" => TransitionKind::SlideUp,
        "SlideDown" => TransitionKind::SlideDown,
        "SlideLeft" => TransitionKind::SlideLeft,
        "SlideRight" => TransitionKind::SlideRight,
        "Pop" => TransitionKind::Pop,
        "PopTopLeft" => TransitionKind::PopTopLeft,
        "PopTopRight" => TransitionKind::PopTopRight,
        "PopBottomLeft" => TransitionKind::PopBottomLeft,
        "PopBottomRight" => TransitionKind::PopBottomRight,
        "SkewUp" => TransitionKind::SkewUp,
        "SkewDown" => TransitionKind::SkewDown,
        "RotateLeft" => TransitionKind::RotateLeft,
        "RotateRight" => TransitionKind::RotateRight,
        _ => TransitionKind::Fade,
    }
}

/// The `filter` of the demo's `from` state; `none` passes no `from`.
fn from_filter(values: &DemoValues) -> Option<String> {
    match values.str("from").as_str() {
        "blur" => Some("blur(4px)".to_string()),
        "grayscale" => Some("grayscale(1)".to_string()),
        _ => None,
    }
}

/// The live demo; its `show` signal prints above the `rsx!`.
const FILE: DemoFile = DemoFile(include_str!("transition/demo.rs"));

/// Prints the trigger and its `show` signal: `open` is the caller's, and the paste needs it to compile.
fn wrap_page(_values: &DemoValues, source: &str) -> String {
    let transition = indent(&indent(source));
    format!(
        "{}\n\n\
         rsx! {{\n    \
         Flex {{\n        \
         direction: \"column\",\n        \
         align: \"flex-start\",\n        \
         gap: \"sm\",\n        \
         Button {{\n            \
         aria_expanded: show(),\n            \
         onclick: move |_| show.toggle(),\n            \
         \"Toggle\"\n        \
         }}\n\
         {transition}    }}\n}}",
        FILE.section("state")
    )
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
                    .doc("How the children move in and out. Travel: `Fade`, `FadeUp`, `FadeDown`, `FadeLeft`, `FadeRight`, `SlideUp`, `SlideDown`, `SlideLeft`, `SlideRight`. Grow: `Scale`, `ScaleX`, `ScaleY`, `Pop`, `PopTopLeft`, `PopTopRight`, `PopBottomLeft`, `PopBottomRight`. Lean or turn: `SkewUp`, `SkewDown`, `RotateLeft`, `RotateRight`. All of them also fade. The slides travel the children's own size; left and right are physical directions."),
                prop("open", "bool")
                    .default("true")
                    .doc("Omitted, the children animate in once on mount and never exit. Passed, they enter and exit as it flips; the first value does not animate. You own this state."),
                prop("duration", "u32")
                    .default("200")
                    .doc("Animation length in milliseconds. `0` turns the animation off."),
                prop("from", "Sx")
                    .doc("Extra styles of the closed state, stacked on the `kind`'s; every property it sets also animates, and a `transform` in it replaces the kind's. Needs a passed `open`: the mount entrance animates only the `kind`."),
                prop("children", "Element").doc("The content that animates. It is unmounted once the exit ends."),
            ])],
            accessibility: a11y()
                .handles([
                    "Closed children are hidden from the focus order and screen readers once the exit ends, and removed from the DOM.",
                    "Under reduced motion the children switch instantly.",
                    "With `open` omitted, the server markup is already visible: the entrance is a CSS animation that needs no JavaScript.",
                ])
                .must([
                    "Give the button that shows and hides the content `aria_expanded`, so a screen reader hears whether it is open.",
                    "Move focus yourself when focused content exits: use `use_focus_return`.",
                ])
                .example("A \"Show filters\" button with `aria_expanded` above a filter panel in a `Transition`: a screen reader hears whether the panel is open, and once it closes its fields leave the Tab order."),
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
                    Control::select("from", ["none", "blur", "grayscale"])
                        .labels(["None", "Blur", "Grayscale"])
                        .code(|_, values| match from_filter(values) {
                            Some(filter) => vec![format!("from: sx().filter({filter:?})")],
                            None => vec![],
                        }),
                ],
                wrap: Wrap(wrap_page),
                render: move |values: DemoValues| {
                    let duration = match values.str("duration").as_str() {
                        "200" => None,
                        value => value.parse::<u32>().ok(),
                    };

                    rsx! {
                        TransitionDemo {
                            kind: kind_of(&values.str("kind")),
                            duration,
                            filter: from_filter(&values),
                        }
                    }
                },
            }
        }
    }
}
