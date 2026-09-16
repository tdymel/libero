use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
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
                    .doc("What scrolls. Rendered once per copy; interactive children work only in the first copy."),
                prop("orientation", "Orientation")
                    .default("horizontal")
                    .doc("The axis it scrolls along. A vertical marquee needs a height from `sx`, or it is as tall as all its copies."),
                prop("reverse", "bool")
                    .default("false")
                    .doc("Scrolls towards the end instead of the start."),
                prop("duration", "u32")
                    .default("40000")
                    .doc("Milliseconds per full cycle. A duration, not a speed: the same number moves a longer strip faster."),
                prop("gap", "Size")
                    .default("md")
                    .doc("Between copies, and between the last and the first."),
                prop("repeat", "u8")
                    .default("4")
                    .doc("Copies laid in a row. Raise it when a gap crosses the view. Anything below 2 renders 2."),
                prop("pause_on_hover", "bool")
                    .default("false")
                    .doc("Pointer hover pauses. Not a way to stop it on its own: a keyboard or a touch screen cannot hover."),
                prop("paused", "Option<bool>")
                    .default("None")
                    .doc("Strictly controlled when set - pair it with `onpausechange`. `None` leaves the state to the built-in toggle."),
                prop("onpausechange", "EventHandler<bool>")
                    .doc("The built-in toggle was pressed, with the state it asks for."),
                prop("pause_control", "bool")
                    .default("true")
                    .doc("Renders the pause toggle. Turn it off only when the page offers its own control, through `paused`."),
                prop("fade_edges", "bool")
                    .default("false")
                    .doc("Fades both ends into the surface color, `--lsx-paper-background`. Only right on a surface of that color."),
            ])],
            lead: rsx! {
                Text {
                    "Content that scrolls on its own in an endless loop - a logo strip, a "
                    "ticker. The children are rendered "
                    Code { source: "repeat" }
                    " times in a row, and every copy after the first is "
                    Code { source: "aria-hidden" }
                    " and "
                    Code { source: "inert" }
                    ", so it is read and tabbed through once - and interactive children "
                    "work only in the first copy. "
                    Code { source: "duration" }
                    " is one full cycle, so adding an item makes the whole strip move faster."
                }
                Text {
                    "The pause toggle is there for WCAG 2.2.2, which asks for a way to stop "
                    "any motion that runs longer than five seconds. Its name stays "
                    "\"Pause\" and "
                    Code { source: "aria-pressed" }
                    " says whether it is paused. Turn it off with "
                    Code { source: "pause_control: false" }
                    " only when the page offers its own control through "
                    Code { source: "paused" }
                    " and "
                    Code { source: "onpausechange" }
                    "."
                }
                Text {
                    "Under "
                    Code { source: "prefers-reduced-motion: reduce" }
                    " it does not move at all: it shows one copy in a strip the reader "
                    "scrolls themselves, without the fade or the toggle. That is pure CSS, "
                    "so no control below can show it - switch the setting in your system "
                    "or your browser's dev tools."
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
