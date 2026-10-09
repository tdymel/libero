use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Chip, Code, Flex, Marquee, MarqueePart, Text},
    sx::sx,
    use_theme,
};

const CHIPS: [&str; 6] = ["Rust", "Dioxus", "WebAssembly", "Blitz", "Tokio", "Serde"];

/// The `links` switch's content: something to Tab to, so the focus behaviour shows (todo 2482).
const LINKS: [(&str, &str); 6] = [
    ("Rust", "https://www.rust-lang.org"),
    ("Dioxus", "https://dioxuslabs.com"),
    ("WebAssembly", "https://webassembly.org"),
    ("Blitz", "https://github.com/DioxusLabs/blitz"),
    ("Tokio", "https://tokio.rs"),
    ("Serde", "https://serde.rs"),
];

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

fn links(values: &DemoValues) -> bool {
    values.str("links") == "true"
}

/// The chips' lines swapped for links, at whatever indent they print.
fn with_links(source: &str) -> String {
    LINKS
        .iter()
        .zip(CHIPS)
        .fold(source.to_string(), |code, ((name, url), chip)| {
            code.replace(
                &format!("Chip {{ {chip:?} }}"),
                &format!("Anchor {{ to: {url:?}, {name:?} }}"),
            )
        })
}

fn vertical(values: &DemoValues) -> bool {
    values.str("orientation") == "vertical"
}

/// The page's own pause button, which `pause_control: false` asks for (WCAG 2.2.2).
// snippet: let mut paused = use_signal(|| false);
const OWN_PAUSE: &str =
    r#"Button { selected: paused(), onclick: move |_| paused.toggle(), "Pause the ticker" }"#;

/// Without the built-in toggle, prints the `paused` signal and the page's own button too.
fn wrap(values: &DemoValues, source: &str) -> String {
    let swapped = links(values).then(|| with_links(source));
    let source = swapped.as_deref().unwrap_or(source);
    if values.str("pause_control") == "true" {
        return source.to_string();
    }
    let mut code = String::from("let mut paused = use_signal(|| false);\n\nrsx! {\n");
    code.push_str(&indent(OWN_PAUSE));
    code.push_str(&indent(source));
    code.push('}');
    code
}

/// Owns `paused`, so the page's own button outlives a control change.
#[component]
fn MarqueePreview(values: DemoValues) -> Element {
    let mut paused = use_signal(|| false);
    let own_control = values.str("pause_control") != "true";
    rsx! {
        // The preview box is a flex row: without a width the strip's 1900px sets this column's.
        Flex {
            direction: "column",
            gap: "md",
            sx: sx().width("100%").min_width("0"),
            if own_control {
                Button {
                    selected: paused(),
                    onclick: move |_| paused.toggle(),
                    "Pause the ticker"
                }
            }
            Marquee {
                orientation: values.str("orientation"),
                sx: vertical(&values).then(|| sx().height(VERTICAL_HEIGHT)),
                duration: values.str("duration").parse::<u32>().ok(),
                gap: values.str("gap"),
                repeat: values.str("repeat").parse::<u8>().ok(),
                reverse: values.str("reverse") == "true",
                pause_on_hover: values.str("pause_on_hover") == "true",
                pause_control: !own_control,
                paused: own_control.then(&*paused),
                fade_edges: values.str("fade_edges") == "true",
                if links(&values) {
                    for (name, url) in LINKS {
                        Anchor { to: url, "{name}" }
                    }
                } else {
                    for chip in CHIPS {
                        Chip { "{chip}" }
                    }
                }
            }
        }
    }
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
                    .doc("What scrolls, rendered once per copy. Interactive children work only in the first copy, and an `id` repeats in every copy."),
                prop("orientation", "Orientation")
                    .default("horizontal")
                    .doc("The axis it scrolls along. A vertical marquee needs a height from `sx`, or it is as tall as all its copies."),
                prop("reverse", "bool")
                    .default("false")
                    .doc("Scrolls towards the end instead of the start."),
                prop("duration", "u32")
                    .default("40000")
                    .doc("Milliseconds per full cycle. The same number moves a longer strip faster."),
                prop("gap", "ThemeAwareValue")
                    .default("md")
                    .doc("Between copies, and between the last and the first, or any CSS, e.g. `gap: \"0\"`."),
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
                prop("parts", "Parts<MarqueePart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`."),
            ])
            .parts("MarqueePart", vec![
                (MarqueePart::Track, "The moving row of copies."),
                (MarqueePart::Group, "One copy of the children."),
                (MarqueePart::Pause, "The pause toggle."),
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
                    "Give the children no `id`: they render once per copy, so an `id` would repeat and `label for`, `aria-labelledby` or `#id` links would find only the first.",
                    "Don't rely on `pause_on_hover` alone: a keyboard or a touch screen cannot hover.",
                    "Turn the toggle off only when the page offers its own control through `paused` and `onpausechange`. `paused` is controlled when set: the toggle then only reports through `onpausechange`, and without the handler it does nothing.",
                ])
                .example("A logo strip in a `Marquee`: a screen reader reads the logos once, and a keyboard user Tabs to the toggle named \"Pause\" and stops the motion with Enter."),
            lead: rsx! {
                Text {
                    "Content that scrolls on its own in an endless loop, such as a logo strip "
                    "or a ticker. The children render "
                    Code { source: "repeat" }
                    " times in a row. "
                    Code { source: "duration" }
                    " is one full cycle, so adding an item makes the strip move faster. With "
                    Code { source: "pause_control" }
                    " off, the demo puts the page's own pause button above it, through "
                    Code { source: "paused" }
                    "."
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
                    Control::slider("gap", ["0", "xs", "sm", "md", "lg", "xl"])
                        .default(defaults.gap.as_str()),
                    Control::slider("repeat", ["2", "3", "4", "5", "6"])
                        .default(defaults.repeat.to_string())
                        .code(integer),
                    Control::switch("reverse"),
                    Control::switch("pause_on_hover"),
                    Control::switch("pause_control").default("true").code(|_, values| {
                        match values.str("pause_control").as_str() {
                            "true" => vec![],
                            _ => vec!["pause_control: false".to_string(), "paused: paused()".to_string()],
                        }
                    }),
                    Control::switch("fade_edges"),
                    Control::switch("links").code(|_, _| vec![]),
                ],
                wrap: Wrap(wrap),
                render: move |values: DemoValues| rsx! {
                    MarqueePreview { values }
                },
            }

            DocSection {
                title: "Reduced motion",
                Text {
                    "Under "
                    Code { source: "prefers-reduced-motion: reduce" }
                    " it does not move. It shows one copy in a strip the reader scrolls, "
                    "without the fade or the toggle. To see it, switch the setting in your "
                    "system or your browser's dev tools."
                }
            }
        }
    }
}
