use crate::components::{Control, Demo, DemoValues, DocPage, UNSET, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Button, Chip, Code, Flex, Input, Kbd, Scroller, ScrollerEdges, Text, use_scroller,
    },
    sx::sx,
    use_theme,
};

/// Wide enough to overflow the preview at every width.
const TAGS: [&str; 20] = [
    "Rust",
    "Dioxus",
    "WebAssembly",
    "Accessibility",
    "Layout",
    "Theming",
    "Forms",
    "Navigation",
    "Overlays",
    "Typography",
    "Performance",
    "Testing",
    "Tooling",
    "Rendering",
    "Signals",
    "Routing",
    "Animation",
    "Icons",
    "Tables",
    "Charts",
];

/// The strip itself, printed verbatim so the code block builds the preview.
// snippet: item const TAGS: [&str; 2] = ["rust", "dioxus"];
const CHILDREN: &str = r#"Flex { direction: "row", gap: "sm", wrap: "nowrap",
    for tag in TAGS {
        Chip { key: "{tag}", "{tag}" }
    }
}"#;

/// The edge report, printed under the strip. `onedgechange` is in `fixed`;
/// this adds the signal it writes and the text that reads it. Under
/// `controls: "never"` also the handle and the caller's own two buttons. A
/// column, so the report sits below the strip rather than beside it.
fn wrap_edges(values: &DemoValues, source: &str) -> String {
    let own = values.str("controls") == "never";
    let (handle, buttons) = match own {
        true => (
            "let strip = use_scroller();\n",
            "        Flex { direction: \"row\", gap: \"sm\",\n            \
             Button { variant: \"outlined\", onclick: move |_| strip.step_back(), \"Back\" }\n            \
             Button { variant: \"outlined\", onclick: move |_| strip.step_forward(), \"Forward\" }\n        \
             }\n",
        ),
        false => ("", ""),
    };
    format!(
        "let mut edges = use_signal(|| None::<ScrollerEdges>);\n{handle}\n\
         rsx! {{\n    \
         Flex {{ direction: \"column\", gap: \"sm\", sx: sx().width(\"100%\"),\n\
         {}{buttons}        Text {{ size: \"sm\",\n            \
         match edges() {{\n                \
         Some(ScrollerEdges {{ at_start: true, at_end: true }}) => \"Everything fits\",\n                \
         Some(ScrollerEdges {{ at_end: false, .. }}) => \"More to the right\",\n                \
         Some(_) => \"End of the list\",\n                \
         None => \"\",\n            \
         }}\n        \
         }}\n    \
         }}\n}}",
        indent(&indent(source))
    )
}

fn strip_content() -> Element {
    rsx! {
        Flex { direction: "row", gap: "sm", wrap: "nowrap",
            for tag in TAGS {
                Chip { key: "{tag}", "{tag}" }
            }
        }
    }
}

/// A page-local component, so the signal is its own and not `Demo`'s.
/// **Kept in step with `wrap_edges` by hand.**
#[component]
fn ScrollerDemo(values: DemoValues) -> Element {
    let mut edges = use_signal(|| None::<ScrollerEdges>);
    let strip = use_scroller();
    let fade = values.str("fade_color");
    let own = values.str("controls") == "never";

    rsx! {
        Flex { direction: "column", gap: "sm", sx: sx().width("100%"),
            Scroller {
                aria_label: "Tags",
                scroll_amount: values.str("scroll_amount").parse::<u32>().ok(),
                controls: values.str("controls"),
                control_size: values.str("control_size"),
                fade_color: match fade.as_str() {
                    UNSET => Input::None,
                    _ => Input::from(fade),
                },
                draggable: values.str("draggable") == "true",
                onedgechange: move |next| edges.set(Some(next)),
                // Always bound, not only under `never`: an element handle is
                // attached on mount, so one passed later is never mounted.
                handle: strip,
                {strip_content()}
            }
            if own {
                Flex { direction: "row", gap: "sm",
                    // Not `disabled` at an end: a focused button that becomes
                    // disabled drops focus to the page. A step there does nothing.
                    Button {
                        variant: "outlined",
                        onclick: move |_| strip.step_back(),
                        "Back"
                    }
                    Button {
                        variant: "outlined",
                        onclick: move |_| strip.step_forward(),
                        "Forward"
                    }
                }
            }
            Text { size: "sm",
                match edges() {
                    Some(ScrollerEdges { at_start: true, at_end: true }) => "Everything fits",
                    Some(ScrollerEdges { at_end: false, .. }) => "More to the right",
                    Some(_) => "End of the list",
                    None => "",
                }
            }
        }
    }
}

#[component]
pub fn ScrollerPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Scroller",
            source: "libero/src/components/layout/scroller.rs",
            markdown: "/md/scroller.md",
            properties: vec![
                props("Scroller", vec![
                    prop("aria_label", "String")
                        .doc("Required. Names the scrollable region, which is a tab stop."),
                    prop("scroll_amount", "u32")
                        .default(theme.scroller.scroll_amount.to_string())
                        .doc("Pixels one control press scrolls."),
                    prop("controls", "ScrollerControls")
                        .default(theme.scroller.controls.as_str())
                        .doc("auto shows each control while there is content that way; always keeps both, dimmed at their end; never renders neither."),
                    prop("control_size", "Size")
                        .default(theme.scroller.control_size.as_str())
                        .doc("Width of each control strip and its glyph."),
                    prop("fade_color", "ThemeAwareValue")
                        .default("paper background")
                        .doc("What the gradient under a control fades from. Set it to the surface the strip sits on."),
                    prop("draggable", "bool")
                        .default(theme.scroller.draggable.to_string())
                        .doc("Mouse drag-to-pan. Touch and trackpad scroll natively either way."),
                    prop("onedgechange", "EventHandler<ScrollerEdges>")
                        .doc("Fires when either edge state flips, including the first measurement."),
                    prop("handle", "ScrollerHandle")
                        .doc("From use_scroller(). Its step_forward() and step_back() move the strip as the controls do, for buttons of your own."),
                    prop("children", "Element").doc("The strip."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A horizontal strip with its scrollbar hidden and a step control over each "
                    "end, which shows while there is more content that way. A press scrolls by "
                    Code { source: "scroll_amount" }
                    " pixels. The scrolling itself is the browser's, so touch, trackpad and the "
                    "arrow keys on the focused strip work untouched. Each control's background "
                    "is a gradient from the surface colour, so the content fades out beneath it."
                }
                Text {
                    "The strip is a tab stop, so "
                    Code { source: "aria_label" }
                    " is required. On the focused strip "
                    Kbd { "←" } " and " Kbd { "→" }
                    " scroll it natively. A control at its own end leaves the tab order, but it "
                    "keeps focus if it had it."
                }
                Text {
                    Code { source: "onedgechange" }
                    " reports whether the strip rests against either end, once it is first "
                    "measured and again whenever that changes; the line under the strip "
                    "prints it. With "
                    Code { source: "controls: \"never\"" }
                    " it is the whole affordance, for a strip that should say so in its own words. "
                    "To move it from buttons of your own, pass a "
                    Code { source: "use_scroller()" }
                    " handle and call its "
                    Code { source: "step_forward()" }
                    " and "
                    Code { source: "step_back()" }
                    "; pick never in the preview to see it."
                }
            },
            Demo {
                component: "Scroller",
                children_text: "",
                children_code: CHILDREN.to_string(),
                fixed: vec![
                    r#"aria_label: "Tags""#.to_string(),
                    "onedgechange: move |next| edges.set(Some(next))".to_string(),
                ],
                controls: vec![
                    Control::slider("scroll_amount", ["120", "200", "320", "480"])
                        .default(theme.scroller.scroll_amount.to_string())
                        .code(|control, values| {
                            let value = values.str(control.name);
                            match value == control.default {
                                true => vec![],
                                false => vec![format!("scroll_amount: {value}")],
                            }
                        }),
                    // `never` hands the stepping to the caller's own buttons,
                    // so it also prints the handle they drive.
                    Control::toggle("controls", ["auto", "always", "never"])
                        .default(theme.scroller.controls.as_str())
                        .code(|control, values| {
                            let value = values.str(control.name);
                            let mut lines = match value == control.default {
                                true => vec![],
                                false => vec![format!(r#"controls: "{value}""#)],
                            };
                            if value == "never" {
                                lines.push("handle: strip".to_string());
                            }
                            lines
                        }),
                    Control::slider("control_size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.scroller.control_size.as_str()),
                    Control::color_shades("fade_color", [UNSET, "grey.1", "primary.1", "warning.1"]),
                    Control::switch("draggable"),
                ],
                wrap: Wrap(wrap_edges),
                render: move |values: DemoValues| rsx! { ScrollerDemo { values } },
            }
        }
    }
}
