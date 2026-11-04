use crate::components::{
    Control, Demo, DemoValues, DocPage, UNSET, a11y, gradient_controls, gradient_value, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Input, Text};
use libero::sx::sx;
use libero::theme::{Responsive, Size, responsive};
use libero::use_theme;

/// `sm` below `md`, the chosen size from it, while the `responsive` switch is on.
fn responsive_size(values: &DemoValues) -> Responsive<Size> {
    let size = Size::from(values.str("size").as_str());
    match values.str("responsive") == "true" {
        true => responsive(Size::Sm).md(size),
        false => size.into(),
    }
}

fn size_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match (values.str("responsive") == "true", values.str("size")) {
        (true, size) => vec![format!(
            "size: responsive(Size::Sm).md(Size::{:?})",
            Size::from(size.as_str())
        )],
        (false, size) if size == control.default => vec![],
        (false, size) => vec![format!("size: {size:?}")],
    }
}

/// Long enough to fill three lines of `CUT_WIDTH`.
const TEXT: &str = "The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs. How vexingly quick daft zebras jump!";

/// Narrow enough that `TEXT` needs a cut.
const CUT_WIDTH: &str = "16rem";

fn no_gradient(values: &DemoValues) -> bool {
    values.str("gradient") != "true"
}

fn is_cut(values: &DemoValues) -> bool {
    values.str("cut") != UNSET
}

/// The `cut` choice is a `truncate` side or a `line_clamp` count.
fn clamp(values: &DemoValues) -> Option<u32> {
    values.str("cut").parse().ok()
}

/// With `responsive` on, a `truncate` side below `md` and three lines from it.
fn clamp_of(values: &DemoValues) -> Input<Responsive<u32>> {
    match (
        values.str("responsive") == "true",
        values.str("cut").as_str(),
    ) {
        (true, "end" | "start") => Input::Value(responsive(0).md(3)),
        _ => clamp(values).map_or(Input::None, |lines| Input::Value(lines.into())),
    }
}

/// `truncate` or `line_clamp`, and the `sx` that narrows the preview to it.
fn cut_code(values: &DemoValues) -> Vec<String> {
    let mut lines = Vec::new();
    match values.str("cut").as_str() {
        UNSET => {}
        value @ ("end" | "start") => {
            lines.push(format!("truncate: {value:?}"));
            if values.str("responsive") == "true" {
                lines.push("line_clamp: responsive(0).md(3)".to_string());
            }
        }
        lines_count => lines.push(format!("line_clamp: {lines_count}")),
    }
    if is_cut(values) {
        lines.push(format!("sx: sx().max_width({CUT_WIDTH:?})"));
    }
    lines
}

#[component]
pub fn TextPage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(no_gradient);
    rsx! {
        DocPage {
            title: "Text",
            source: "libero/src/components/typography/text.rs",
            markdown: "/md/text.md",
            properties: vec![props("Text", vec![
                prop("size", "Responsive<Size>")
                    .default(theme.text.size.as_str())
                    .doc("Visual size, `xs` to `xxl`. Or one per breakpoint, `size: responsive(Size::Sm).md(Size::Lg)`. Breakpoints follow the window, not the parent."),
                prop("component", "HtmlTag")
                    .default("p")
                    .doc("The element to render."),
                prop("color", "ThemeAwareValue")
                    .doc("The text color: a theme color name in its text shade, or any CSS color. Unset, the text inherits. Under a gradient, its first stop."),
                prop("gradient", "Gradient")
                    .doc("Paints the glyphs with a gradient from `color` to a second stop, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`; `Gradient::default()` is the theme's. Keep it to large display text: the contrast of a literal CSS stop is yours to check, and a debug build warns when a hex stop reads under 4.5:1 on the page background. Solid in its first stop in print and in native windows, the system text colour in forced colours."),
                prop("truncate", "Truncate")
                    .doc("Cuts the text on one line with an ellipsis at its `end` or `start`. The element becomes a block, so a `span` truncates too. Set a `title` if the cut text matters; the full text stays in the accessible name. `line_clamp` wins when both are set."),
                prop("line_clamp", "Responsive<u32>")
                    .doc("Cuts the text after this many lines with an ellipsis. Over `truncate`; `0` is off. On Blitz there is no line clamp: the text is cut at that many lines, without the ellipsis. Or one per breakpoint, `line_clamp: responsive(3).md(0)`, where `0` lifts the clamp in that range, or cuts one line with `truncate` when one is set."),
                prop("children", "Element").default("required").doc("The text."),
            ])],
            accessibility: a11y()
                .handles([
                    "A large size is only styling, so it never makes a heading.",
                    "A long word breaks inside the text rather than overflowing a narrow column.",
                    "A cut text is still whole in the DOM, so a screen reader reads all of it.",
                ])
                .must([
                    "Use `component: \"span\"` for text inside a sentence.",
                    "For a heading, use `Title`.",
                    "Check the contrast of a literal CSS `color` on its background, 4.5:1 for body text: a theme color name takes its text shade, a literal is used as given.",
                    "Keep `gradient` to large display text and check the contrast of a literal CSS stop. A debug build warns when a hex stop reads under 4.5:1 on the page background.",
                    "Pass a text that may be cut as `title` too, so sighted users can read it on hover, or show it in full elsewhere.",
                ])
                .example("A price in a sentence, `Text { component: \"span\", size: \"lg\", \"$12\" }`: it stays part of the sentence, and its large size does not make it a heading.")
                .limits([
                    "`component` keeps the body-text look, so `a`, `strong`, `del` and the like lose their underline, weight or line. For a link, use `Anchor`.",
                    "On Blitz, `line_clamp` cuts the text at its last line without an ellipsis.",
                ]),
            lead: rsx! {
                Text {
                    "Body copy in a "
                    Code { source: "<p>" }
                    ", sized from the theme's text scale. "
                    Code { source: "component" }
                    " changes the element without changing the look. For headings, use "
                    Code { source: "Title" }
                    "."
                }
            },
            Demo {
                component: "Text",
                children_text: TEXT,
                controls: vec![
                    Control::sizes("size")
                        .default(theme.text.size.as_str()).code(size_code),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").code(|_, _| vec![]),
                    Control::toggle("component", ["p", "span", "div"])
                        .labels(["P", "Span", "Div"]),
                    // Unset inherits the page's ink, or under a gradient starts at the theme's first stop.
                    Control::color("color")
                        .with_unset()
                        .unset_swatch("ink")
                        .unset_swatch_when(
                            |values| !no_gradient(values),
                            format!("{}.6", theme.gradient.from.as_str()),
                        ),
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
                    // `truncate` cuts one line at its end or start, `line_clamp` after 2 or 3 lines.
                    Control::toggle("cut", [UNSET, "end", "start", "2", "3"])
                        .labels(["Unset", "End", "Start", "2 lines", "3 lines"])
                        .default("end")
                        .code(|_, values| cut_code(values)),
                ],
                render: move |values: DemoValues| rsx! {
                    Text {
                        size: responsive_size(&values),
                        component: values.str("component"),
                        truncate: match values.str("cut").as_str() {
                            side @ ("end" | "start") => Input::from(side),
                            _ => Input::None,
                        },
                        line_clamp: clamp_of(&values),
                        sx: is_cut(&values).then(|| sx().max_width(CUT_WIDTH)),
                        color: match values.str("color").as_str() {
                            UNSET => Input::None,
                            color => Input::from(color),
                        },
                        gradient: (values.str("gradient") == "true").then(|| gradient_value(&values)),
                        {TEXT}
                    }
                },
            }
        }
    }
}
