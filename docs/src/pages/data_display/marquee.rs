use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Chip, Code, Marquee, Text},
    sx::sx,
    use_theme,
};

const CHIPS: [&str; 6] = ["Rust", "Dioxus", "WebAssembly", "Blitz", "Tokio", "Serde"];

/// A vertical marquee is as tall as all its copies unless it is given a
/// height, so the demo gives it one and prints it.
const VERTICAL_HEIGHT: &str = "160px";

fn children_code() -> String {
    CHIPS
        .iter()
        .map(|chip| format!("Chip {{ {chip:?} }}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// An integer prop: printed bare, and only when it moved off the default.
fn integer(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    match value == control.default {
        true => vec![],
        false => vec![format!("{}: {value}", control.name)],
    }
}

fn vertical(values: &DemoValues) -> bool {
    values.str("orientation") == "vertical"
}

#[component]
pub fn MarqueePage() -> Element {
    let theme = use_theme();
    let defaults = theme.marquee;

    rsx! {
        DocPage {
            title: "Marquee",
            source: "libero/src/components/data_display/marquee.rs",
            markdown: "/md/marquee.md",
            properties: vec![props("Marquee", vec![
                prop("children", "Element")
                    .default("required")
                    .doc("What scrolls, rendered once per copy. Interactive children work only in the first copy."),
                prop("orientation", "Orientation")
                    .default("horizontal")
                    .doc("The axis it scrolls along. A vertical marquee needs a height from `sx`, or it is as tall as all its copies."),
                prop("reverse", "bool")
                    .default("false")
                    .doc("Scrolls towards the end instead of the start."),
                prop("duration", "u32")
                    .default("40000")
                    .doc("Milliseconds per full cycle. The same number moves a longer strip faster."),
                prop("gap", "Size")
                    .default("md")
                    .doc("Between copies, and between the last and the first."),
                prop("repeat", "u8")
                    .default("4")
                    .doc("Copies in a row. Raise it when a gap crosses the view. Anything below 2 renders 2."),
                prop("pause_on_hover", "bool")
                    .default("false")
                    .doc("Pauses under the pointer. Not enough on its own, since a keyboard or a touch screen cannot hover."),
                prop("paused", "Option<bool>")
                    .default("None")
                    .doc("Controlled when set, so pair it with `onpausechange`. `None` leaves the state to the built-in toggle."),
                prop("onpausechange", "EventHandler<bool>")
                    .default("None")
                    .doc("The built-in toggle was pressed, with the state it asks for."),
                prop("pause_control", "bool")
                    .default("true")
                    .doc("Renders the pause toggle. Turn it off only when the page offers its own control through `paused`."),
                prop("fade_edges", "bool")
                    .default("false")
                    .doc("Fades both ends into the surface color. Only right on a surface of that color."),
            ])],
            accessibility: a11y()
                .key(["Enter", "Space"], "On the pause toggle, a tab stop: pauses or resumes the motion.")
                .handles([
                    "Every copy after the first is `aria-hidden` and `inert`, so the content is read and tabbed through once.",
                    "The pause toggle meets WCAG 2.2.2, which asks for a way to stop motion that runs longer than five seconds. It is named \"Pause\", and `aria-pressed` says whether it is paused.",
                    "While focus is on the content the toggle turns transparent, so it never covers a focused link.",
                ])
                .must([
                    "Put interactive children in the content knowing they work only in the first copy.",
                    "Don't rely on `pause_on_hover` alone: a keyboard or a touch screen cannot hover.",
                    "Turn the toggle off only when the page offers its own control through `paused` and `onpausechange`. `paused` is controlled when set: the toggle then only reports through `onpausechange`, and without the handler it does nothing.",
                ]),
            lead: rsx! {
                Text {
                    "Content that scrolls on its own in an endless loop, such as a logo strip "
                    "or a ticker. The children render "
                    Code { source: "repeat" }
                    " times in a row. "
                    Code { source: "duration" }
                    " is one full cycle, so adding an item makes the strip move faster."
                }
                Text {
                    "Under "
                    Code { source: "prefers-reduced-motion: reduce" }
                    " it does not move. It shows one copy in a strip the reader scrolls, "
                    "without the fade or the toggle. To see it, switch the setting in your "
                    "system or your browser's dev tools."
                }
            },
            Demo {
                component: "Marquee",
                children_text: "",
                children_code: children_code(),
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .code(|_, values| match vertical(values) {
                            true => vec![
                                "orientation: \"vertical\"".to_string(),
                                format!("sx: sx().height({VERTICAL_HEIGHT:?})"),
                            ],
                            false => vec![],
                        }),
                    Control::slider("duration", ["10000", "25000", "40000", "80000"])
                        .default(defaults.duration.to_string())
                        .code(integer),
                    Control::slider("gap", ["xs", "sm", "md", "lg", "xl"])
                        .default(defaults.gap.as_str()),
                    Control::slider("repeat", ["2", "3", "4", "5", "6"])
                        .default(defaults.repeat.to_string())
                        .code(integer),
                    Control::switch("reverse"),
                    Control::switch("pause_on_hover"),
                    Control::switch("pause_control").default("true"),
                    Control::switch("fade_edges"),
                ],
                render: move |values: DemoValues| rsx! {
                    Marquee {
                        orientation: values.str("orientation"),
                        sx: vertical(&values).then(|| sx().height(VERTICAL_HEIGHT)),
                        duration: values.str("duration").parse::<u32>().ok(),
                        gap: values.str("gap"),
                        repeat: values.str("repeat").parse::<u8>().ok(),
                        reverse: values.str("reverse") == "true",
                        pause_on_hover: values.str("pause_on_hover") == "true",
                        pause_control: values.str("pause_control") == "true",
                        fade_edges: values.str("fade_edges") == "true",
                        for chip in CHIPS {
                            Chip { "{chip}" }
                        }
                    }
                },
            }
        }
    }
}
