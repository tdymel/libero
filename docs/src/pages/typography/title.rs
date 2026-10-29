use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, a11y, gradient_controls, gradient_value, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, HtmlTag, Input, Text, Title};
use libero::sx::sx;
use libero::use_theme;

/// Long enough to fill three lines of `CUT_WIDTH` at the default size.
const TEXT: &str = "The quick brown fox jumps over the lazy dog and keeps running";

/// Narrow enough that `TEXT` needs a cut.
const CUT_WIDTH: &str = "20rem";

fn no_gradient(values: &DemoValues) -> bool {
    values.str("gradient") != "true"
}

fn is_cut(values: &DemoValues) -> bool {
    values.str("truncate") != UNSET || values.str("line_clamp") != UNSET
}

/// `truncate` and the `sx` that narrows the preview to it.
fn cut_code(values: &DemoValues) -> Vec<String> {
    let mut lines = Vec::new();
    if let value @ ("end" | "start") = values.str("truncate").as_str() {
        lines.push(format!("truncate: {value:?}"));
    }
    if is_cut(values) {
        lines.push(format!("title: {TEXT:?}"));
        lines.push(format!("sx: sx().max_width({CUT_WIDTH:?})"));
    }
    lines
}

#[component]
pub fn TitlePage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(no_gradient);
    rsx! {
        DocPage {
            title: "Title",
            source: "libero/src/components/typography/title.rs",
            markdown: "/md/title.md",
            properties: vec![props("Title", vec![
                prop("size", "Size")
                    .default(theme.title.size.as_str())
                    .doc("Visual size, `xs` to `xxl`. Also picks the tag unless `component` is set."),
                prop("component", "HtmlTag")
                    .default("follows size")
                    .doc("The heading tag. The size's look stays. With `size` unset it is `h1`, whatever the theme's size. A debug build warns for a tag other than `h1` to `h6`."),
                prop("gradient", "Gradient")
                    .doc("Paints the glyphs with a gradient from the theme's first stop to a second, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`; `Gradient::default()` is the theme's. The contrast of a literal CSS stop is yours to check, and a debug build warns when a hex stop reads under 4.5:1 on the page background. Solid in its first stop in print and in native windows, the system text color in forced colors."),
                prop("truncate", "Truncate")
                    .doc("Cuts the title on one line with an ellipsis at its `end` or `start`. `line_clamp` wins when both are set."),
                prop("line_clamp", "u32")
                    .doc("Cuts the title after this many lines with an ellipsis. Over `truncate`; `0` is off. On Blitz there is no line clamp: the title is cut at that many lines, without the ellipsis."),
                prop("children", "Element").default("required").doc("The heading text."),
            ])],
            accessibility: a11y()
                .handles([
                    "`size` picks the heading tag, `xxl` as `h1` down to `xs` as `h6`, unless `component` is set.",
                    "A cut heading is still whole in the DOM, so a screen reader and the outline read all of it.",
                ])
                .must([
                    "Pass a heading that may be cut as `title` too, so a mouse user can read it on hover. A hover title does not reach touch or keyboard users: on narrow screens prefer `line_clamp` or let the heading wrap, and show the full text elsewhere where it matters.",
                    "Keep one `h1` per page and skip no levels.",
                    "A `lg` heading in a section under the page's `h1` needs `component: \"h2\"`, or the document jumps from `h1` to `h3`.",
                    "Check the contrast of a literal CSS stop in `gradient`. A debug build warns when a hex stop reads under 4.5:1 on the page background.",
                ])
                .example("A section heading under the page's `h1`, `Title { size: \"lg\", component: \"h2\", \"Pricing\" }`: a screen reader lists \"Pricing\" as a level 2 heading, whatever its size.")
                .limits(["On Blitz, `line_clamp` cuts the title at its last line without an ellipsis."]),
            lead: rsx! {
                Text {
                    "A heading, "
                    Code { source: "h1" }
                    " to "
                    Code { source: "h6" }
                    ". "
                    Code { source: "size" }
                    " sets the look and the tag: "
                    Code { source: "xxl" }
                    " is "
                    Code { source: "h1" }
                    ", "
                    Code { source: "xl" }
                    " is "
                    Code { source: "h2" }
                    ", down to "
                    Code { source: "xs" }
                    " as "
                    Code { source: "h6" }
                    ". Set "
                    Code { source: "component" }
                    " when the look and the level disagree."
                }
            },
            Demo {
                    component: "Title",
                    children_text: TEXT,
                    controls: vec![
                        Control::sizes("size")
                            .default(theme.title.size.as_str()),
                        // An h2: the page's own title is its one h1 (todo 4850).
                        Control::slider(
                            "component",
                            ["auto", "h1", "h2", "h3", "h4", "h5", "h6"],
                        )
                        .default("h2")
                        .code(|_, values| match values.str("component").as_str() {
                            "auto" => vec![],
                            tag => vec![format!("component: {tag:?}")],
                        }),
                        // The theme's own second stop and angle print as `Gradient::default()`.
                        Control::switch("gradient").code(|_, values| {
                            let default = values.str("gradient_to") == "secondary"
                                && values.str("gradient_deg") == "45";
                            match values.str("gradient").as_str() {
                                "true" if default => vec!["gradient: Gradient::default()".to_string()],
                                _ => vec![],
                            }
                        }),
                        gradient_to,
                        gradient_deg,
                        // On, so the ellipsis shows without another switch.
                        Control::toggle("truncate", [UNSET, "end", "start"])
                            .labels(["Unset", "End", "Start"])
                            .default("end")
                            .code(|_, values| cut_code(values)),
                        Control::toggle("line_clamp", [UNSET, "1", "2", "3"])
                            .labels(["Unset", "1", "2", "3"])
                            .default(UNSET)
                            .code(|_, values| match values.str("line_clamp").as_str() {
                                UNSET => vec![],
                                lines => vec![format!("line_clamp: {lines}")],
                            }),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Title {
                            size: values.str("size"),
                            truncate: match values.str("truncate").as_str() {
                                UNSET => Input::None,
                                end => Input::from(end),
                            },
                            line_clamp: values.str("line_clamp").parse::<u32>().ok(),
                            // The cut text on hover, as the Must asks.
                            title: is_cut(&values).then_some(TEXT),
                            sx: is_cut(&values).then(|| sx().max_width(CUT_WIDTH)),
                            component: match values.str("component").as_str() {
                                // The default follows `size`, so leave the prop unset.
                                "auto" => Input::None,
                                tag => Input::Value(HtmlTag::from(tag)),
                            },
                            gradient: (values.str("gradient") == "true").then(|| gradient_value(&values)),
                            {TEXT}
                        }
                },
            }
        }
    }
}
