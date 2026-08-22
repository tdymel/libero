use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, or_unset};
use dioxus::prelude::*;
use libero::{
    components::{Box, Flex, Text},
    sx::sx,
};

// `divider` splits children apart, which only the fork can do - so on main the
// control, the prop and the section it documents are all absent.
#[cfg(feature = "dioxus-fork")]
use libero::components::Divider;

/// The three children the demo lays out - a subtree, so the code block prints
/// them verbatim rather than as a quoted child.
const CHILDREN: &str = r#"Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
Box { sx: sx().padding("8px 16px").background("primary.1"), "Three" }"#;

/// `align` and `justify` distribute *spare* space, so the flex needs a box
/// bigger than its children - its own, not a wrapper's, or the children only
/// ever fill it.
const BOX_SX: &str =
    r#"sx: sx().width("400px").height("200px").padding("8px").background("grey.1")"#;

/// The divider a row needs is vertical; a column's is the default rule.
#[cfg(feature = "dioxus-fork")]
fn divider_rsx(values: &DemoValues) -> &'static str {
    match values.str("direction").as_str() {
        "row" => "Divider { orientation: \"vertical\" }",
        _ => "Divider {}",
    }
}

#[cfg(feature = "dioxus-fork")]
fn divider_element(values: &DemoValues) -> Option<Element> {
    (values.str("divider") == "true").then(|| match values.str("direction").as_str() {
        "row" => rsx! { Divider { orientation: "vertical" } },
        _ => rsx! { Divider {} },
    })
}

#[cfg(not(feature = "dioxus-fork"))]
fn divider_element(_values: &DemoValues) -> Option<Element> {
    None
}

fn controls() -> Vec<Control> {
    #[allow(unused_mut)]
    let mut controls = vec![
        Control::toggle("direction", ["column", "row"]),
        Control::slider("gap", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
        Control::select(
            "align",
            ["auto", "flex-start", "center", "flex-end", "stretch"],
        ),
        Control::select(
            "justify",
            [
                "auto",
                "flex-start",
                "center",
                "flex-end",
                "space-between",
                "space-around",
            ],
        ),
        Control::select("wrap", ["auto", "nowrap", "wrap", "wrap-reverse"]),
    ];

    // A rule between row children has to be the other way round, so the
    // divider follows `direction`.
    #[cfg(feature = "dioxus-fork")]
    controls.push(Control::switch("divider").code(
        |_, values| match values.str("divider").as_str() {
            "true" => vec![format!("divider: rsx! {{ {} }}", divider_rsx(values))],
            _ => vec![],
        },
    ));

    controls
}

#[component]
pub fn FlexPage() -> Element {
    rsx! {
        DocPage {
            title: "Flex",
            lead: rsx! {
                Text { "A flexbox container - direction, gap, align, justify and wrap, all theme-aware." }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Flex",
                    children_text: "",
                    children_code: CHILDREN,
                    fixed: vec![BOX_SX.to_string()],
                    controls: controls(),
                    render: move |values: DemoValues| rsx! {
                        Flex {
                            sx: sx()
                                .width("400px")
                                .height("200px")
                                .padding("8px")
                                .background("grey.1"),
                            direction: values.str("direction"),
                            gap: or_unset(values.str("gap")),
                            align: or_unset(values.str("align")),
                            justify: or_unset(values.str("justify")),
                            wrap: or_unset(values.str("wrap")),
                            divider: divider_element(&values),
                            Box {
                                sx: sx().padding("8px 16px").background("primary.1"),
                                "One"
                            }
                            Box {
                                sx: sx().padding("8px 16px").background("primary.1"),
                                "Two"
                            }
                            Box {
                                sx: sx().padding("8px 16px").background("primary.1"),
                                "Three"
                            }
                        }
                    },
                }
            }
        }
    }
}
