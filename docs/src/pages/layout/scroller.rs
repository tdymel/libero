use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, UNSET, Wrap, a11y, indent, prop,
    props,
};
use dioxus::prelude::*;
use libero::{
    components::{
        Button, Chip, Code, Flex, Input, Scroller, ScrollerEdges, ScrollerPart, Text, use_scroller,
    },
    sx::sx,
    use_theme,
};

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("scroller.rs"));

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

/// Adds the signal `onedgechange` writes and the report under the strip; under
/// `controls: "never"` also the handle and the caller's own two buttons.
fn wrap_edges(values: &DemoValues, source: &str) -> String {
    let own = values.str("controls") == "never";
    let (handle, buttons) = match own {
        true => (
            "let strip = use_scroller();\n",
            "        Flex { direction: \"row\", gap: \"sm\",\n            \
             Button { variant: \"outlined\", onclick: move |_| strip.step_back(), \"Scroll tags back\" }\n            \
             Button { variant: \"outlined\", onclick: move |_| strip.step_forward(), \"Scroll tags forward\" }\n        \
             }\n",
        ),
        false => ("", ""),
    };
    format!(
        "let mut edges = use_signal(|| None::<ScrollerEdges>);\n{handle}\n\
         rsx! {{\n    \
         Flex {{ direction: \"column\", gap: \"sm\", sx: sx().width(\"100%\"),\n\
         {}{buttons}        Text {{ size: \"sm\", role: \"status\",\n            \
         match edges() {{\n                \
         Some(ScrollerEdges {{ at_start: true, at_end: true }}) => \"Everything fits\",\n                \
         Some(ScrollerEdges {{ at_end: false, .. }}) => \"More after this\",\n                \
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
        // demo-code: children start
        Flex { direction: "row", gap: "sm", wrap: "nowrap",
            for tag in TAGS {
                Chip { key: "{tag}", "{tag}" }
            }
        }
        // demo-code: children end
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
                        "Scroll tags back"
                    }
                    Button {
                        variant: "outlined",
                        onclick: move |_| strip.step_forward(),
                        "Scroll tags forward"
                    }
                }
            }
            Text { size: "sm", role: "status",
                match edges() {
                    Some(ScrollerEdges { at_start: true, at_end: true }) => "Everything fits",
                    Some(ScrollerEdges { at_end: false, .. }) => "More after this",
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
                        .default("required")
                        .doc("Names the strip, a tab stop while it overflows with nothing focusable inside."),
                    prop("scroll_amount", "u32")
                        .default(theme.scroller.scroll_amount.to_string())
                        .doc("Pixels one control press scrolls."),
                    prop("controls", "ScrollerControls")
                        .default(theme.scroller.controls.as_str())
                        .doc("`auto` shows each control while there is content that way. `always` keeps both, dimmed at their end. `never` shows neither."),
                    prop("control_size", "Size")
                        .default(theme.scroller.control_size.as_str())
                        .doc("Width of each control and its glyph."),
                    prop("fade_color", "ThemeAwareValue")
                        .default("paper background")
                        .doc("The colour the content fades into under a control. Set it to the surface the strip sits on."),
                    prop("draggable", "bool")
                        .default(theme.scroller.draggable.to_string())
                        .doc("Lets the mouse drag the strip. Touch and trackpad scroll it either way."),
                    prop("onedgechange", "EventHandler<ScrollerEdges>")
                        .doc("Fires when the strip reaches or leaves an end, and once when it is first measured."),
                    prop("handle", "ScrollerHandle")
                        .doc("From `use_scroller()`. Its `step_forward()` and `step_back()` move the strip as the controls do, for buttons of your own."),
                    prop("children", "Element").doc("The strip."),
                    prop("parts", "Parts<ScrollerPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                ])
                .parts("ScrollerPart", vec![
                    (ScrollerPart::Viewport, "The `ScrollArea` that scrolls, the named region."),
                    (ScrollerPart::Content, "The strip around the children."),
                    (ScrollerPart::Control, "Both step buttons. `data-state` holds `start` or `end`."),
                ]),
            ],
            accessibility: a11y()
                .key(["Left", "Right"], "Scrolls the focused strip.")
                .handles([
                    "The strip is a tab stop while it overflows with nothing focusable inside. Focusable children take the focus themselves, and the strip scrolls each into view.",
                    "A control at its own end leaves the tab order, but keeps focus if it had it.",
                ])
                .must(["Name the strip with `aria_label`. It is required, as the strip is a named region and can be a tab stop."])
                .example("A row of category chips, `Scroller { aria_label: \"Categories\", .. }`: Tab moves from chip to chip, and the strip scrolls each one into view."),
            lead: rsx! {
                Text {
                    "A horizontal strip with a hidden scrollbar and a step control over "
                    "each end, shown while there is more content that way. A press scrolls "
                    "by "
                    Code { source: "scroll_amount" }
                    " pixels. Touch, trackpad and the arrow keys scroll it as usual. The "
                    "content fades out under each control, so set "
                    Code { source: "fade_color" }
                    " to the surface the strip sits on."
                }
            },
            // snippet: item const TAGS: [&str; 2] = ["rust", "dioxus"];
            Demo {
                component: "Scroller",
                children_text: "",
                children_code: FILE.section("children").to_string(),
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
                        .labels(["Auto", "Always", "Never"])
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
                    Control::sizes("control_size")
                        .default(theme.scroller.control_size.as_str()),
                    Control::color_shades("fade_color", [UNSET, "muted.1", "primary.1", "warning.1"]),
                    Control::switch("draggable"),
                ],
                wrap: Wrap(wrap_edges),
                render: move |values: DemoValues| rsx! { ScrollerDemo { values } },
            }

            DocSection {
                title: "Edges",
                Text {
                    Code { source: "onedgechange" }
                    " reports whether the strip rests against an end, printed under the "
                    "strip. With "
                    Code { source: "controls: \"never\"" }
                    ", move the strip from your own buttons through a "
                    Code { source: "use_scroller()" }
                    " handle."
                }
            }
        }
    }
}
