use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Box, Button, Code, CodeBlock, Flex, Input, Skeleton, Text};
use libero::sx::sx;
use libero::use_theme;

/// What the wrapper covers. A button in it, so the preview shows that covered
/// content is out of the tab order, not only out of sight.
const CONTENT: &str = r#"Flex { direction: "row", gap: "sm", align: "center",
    Text { "Ada Lovelace" }
    Button { size: "xs", "Follow" }
}"#;

/// Todo 107: the grace before a placeholder shows is the caller's timing, so
/// it is a recipe over `timer()` rather than a prop.
// snippet: item #[derive(Clone, PartialEq, Default)] struct Profile;
// snippet: item async fn load_profile() -> Profile { Profile }
// snippet: item #[component] fn ProfileCard(profile: Profile) -> Element { rsx! {} }
const GRACE_EXAMPLE: &str = r#"/// How long a fetch may take before its placeholder shows.
const GRACE: Duration = Duration::from_millis(200);

#[component]
fn Card() -> Element {
    let profile = use_resource(load_profile);
    let loading = profile.read().is_none();
    let mut slow = use_signal(|| false);
    // Dropping the timer cancels it, so an unmounted card never writes `slow`.
    let mut grace = use_signal(|| {
        timer().map(|timer| timer.after(GRACE, Box::new(move || slow.set(true))))
    });
    use_drop(move || grace.set(None));

    rsx! {
        div {
            "aria-busy": loading,
            // `opacity`, not `visibility`, which would not hide the grey.
            opacity: if loading && !slow() { "0" } else { "1" },
            Skeleton { visible: loading,
                ProfileCard { profile: profile.read().clone().unwrap_or_default() }
            }
        }
    }
}"#;

fn busy_region(values: &DemoValues) -> bool {
    values.str("busy_region") == "true"
}

/// The region is the caller's: it carries `aria-busy` while the skeleton
/// covers, the same rule as the `Loader` page.
fn wrap_region(values: &DemoValues, code: &str) -> String {
    match busy_region(values) {
        true => format!(
            "Box {{ \"aria-busy\": \"{}\", sx: sx().width(\"100%\"),\n{}}}",
            values.str("visible"),
            indent(code)
        ),
        false => code.to_string(),
    }
}

fn with_content(values: &DemoValues) -> bool {
    values.str("children") == "true"
}

fn content_code(values: &DemoValues) -> String {
    match with_content(values) {
        true => CONTENT.to_string(),
        false => String::new(),
    }
}

/// The wrapper's `height`: unset at `auto`.
fn height_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    match value == control.default {
        true => vec![],
        false => vec![format!("height: {value:?}")],
    }
}

/// A shape's `height` is never unset, so it always prints.
fn shape_height_code(_: &Control, values: &DemoValues) -> Vec<String> {
    vec![format!("height: {:?}", values.str("shape_height"))]
}

/// `"auto"` is the prop left unset.
fn length(value: String, unset: &str) -> Input<libero::sx::ThemeAwareValue> {
    match value == unset {
        true => Input::None,
        false => Input::from(value),
    }
}

#[component]
pub fn SkeletonPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Skeleton",
            source: "libero/src/components/feedback/skeleton.rs",
            markdown: "/md/skeleton.md",
            properties: vec![props("Skeleton", vec![
                prop("visible", "bool")
                    .default("true")
                    .doc("Hides the children behind the placeholder, or draws the standalone shape. `false` shows the children."),
                prop("height", "ThemeAwareValue")
                    .doc("A CSS length. Unset, the children's height."),
                prop("width", "ThemeAwareValue")
                    .default("100%")
                    .doc("A CSS length. Ignored with `circle`."),
                prop("circle", "bool")
                    .default("false")
                    .doc("A circle as wide as `height`. Without `height`, as wide as the children."),
                prop("radius", "Size")
                    .default(theme.skeleton.radius.as_str())
                    .doc("Corner radius. Ignored with `circle`."),
                prop("animate", "bool")
                    .default(theme.skeleton.animate.to_string())
                    .doc("Runs the pulse. With reduced motion it stops half-way."),
                prop("children", "Element")
                    .doc("The real content, when the skeleton wraps it."),
            ])],
            accessibility: a11y()
                .handles([
                    "A skeleton says nothing to a screen reader.",
                    "Content it hides is not announced and not reachable with Tab.",
                    "With reduced motion the pulse stops half-way.",
                ])
                .must([
                    "Mark the region you are filling `aria-busy` while it waits, as on `Loader`. The demo's `Busy region` switch shows it.",
                    "Avoid a descendant that sets `visibility: visible` on itself under a visible skeleton: it shows through.",
                ]),
            lead: rsx! {
                Text {
                    "A placeholder for content that is still loading. Without children it is "
                    "a grey shape, and a few of them stand in for a layout. Wrapped around "
                    "the real content, it hides that content while "
                    Code { source: "visible" }
                    " is set, so the placeholder has exactly its size. Hidden content is not "
                    "announced and not reachable with Tab."
                }
                Text {
                    "A descendant that sets "
                    Code { source: "visibility: visible" }
                    " on itself shows through, so avoid one under a visible skeleton."
                }
                Text {
                    "A fetch that answers in 50 ms should not flash a placeholder. Keep the "
                    "region transparent until a grace period ends, and end it with "
                    Code { source: "timer()" }
                    ". Use "
                    Code { source: "opacity" }
                    " for that, since "
                    Code { source: "visibility" }
                    " would not hide the grey."
                }
                CodeBlock { source: GRACE_EXAMPLE, language: "rust" }
            },
            Demo {
                component: "Skeleton",
                children_text: "",
                code_child: Child(content_code),
                controls: vec![
                    // Not a prop: it picks between the two usages, and the
                    // code block prints the children through `code_child`.
                    Control::switch("children").default("true").code(|_, _| vec![]),
                    Control::switch("visible").default("true"),
                    Control::switch("animate").default("true"),
                    Control::switch("circle"),
                    // Not a prop: the caller's region around the skeleton. On by default, so
                    // the first code block is the safe setup.
                    Control::switch("busy_region").default("true").code(|_, _| vec![]),
                    Control::sizes("radius")
                        .default("sm")
                        .hidden_when(|values| values.str("circle") == "true"),
                    // Two controls for one prop: a shape with no content needs a height.
                    Control::toggle("height", ["auto", "12px", "40px"])
                        .labels(["Auto", "12px", "40px"])
                        .code(height_code)
                        .hidden_when(|values| !with_content(values)),
                    Control::toggle("shape_height", ["12px", "40px", "80px"])
                        .code(shape_height_code)
                        .hidden_when(with_content),
                    Control::toggle("width", ["100%", "180px", "60%"])
                        .hidden_when(|values| values.str("circle") == "true"),
                ],
                wrap: Wrap(wrap_region),
                render: move |values: DemoValues| {
                    let content = with_content(&values);
                    let height = match content {
                        true => length(values.str("height"), "auto"),
                        false => length(values.str("shape_height"), ""),
                    };
                    let circle = values.str("circle") == "true";
                    let visible = values.str("visible") == "true";
                    let skeleton = rsx! {
                        Skeleton {
                            visible,
                            animate: values.str("animate") == "true",
                            circle,
                            radius: values.str("radius"),
                            height,
                            width: length(values.str("width"), "100%"),
                            if content {
                                Flex { direction: "row", gap: "sm", align: "center",
                                    Text { "Ada Lovelace" }
                                    Button { size: "xs", "Follow" }
                                }
                            }
                        }
                    };
                    match busy_region(&values) {
                        true => rsx! {
                            // Full width, or the `100%` skeleton shrinks with the region.
                            Box {
                                "aria-busy": if visible { "true" } else { "false" },
                                sx: sx().width("100%"),
                                {skeleton}
                            }
                        },
                        false => skeleton,
                    }
                },
            }
        }
    }
}
