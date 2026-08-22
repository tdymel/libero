use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Text},
    sx::sx,
};

const HREF: &str = "https://dioxuslabs.com";
const SX: &str = r#"sx: sx().padding("16px").background("grey.1").border_radius("md")"#;

/// The escape-hatch attributes only exist on some tags, so they follow
/// `component` rather than standing as controls of their own.
fn tag_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let tag = values.str("component");
    let mut set = match tag == control.default {
        true => vec![],
        false => vec![format!("component: {tag:?}")],
    };
    match tag.as_str() {
        "a" => set.extend([format!("href: {HREF:?}"), "target: \"_blank\"".to_string()]),
        "button" => set.push("r#type: \"button\"".to_string()),
        _ => {}
    }
    set
}

#[component]
pub fn BoxPage() -> Element {
    rsx! {
        DocPage {
            title: "Box",
            lead: rsx! {
                Text {
                    "The polymorphic primitive every other component is built on - renders "
                    "as any tag via "
                    Code { source: "component" }
                    ", plus "
                    Code { source: "sx" }
                    "/"
                    Code { source: "states" }
                    " styling and escape-hatch attributes like "
                    Code { source: "href" }
                    "/"
                    Code { source: "src" }
                    ", which follow whichever tag you picked."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Box",
                    children_text: "Styled entirely via sx",
                    fixed: vec![SX.to_string()],
                    controls: vec![
                        Control::toggle("component", ["div", "section", "a", "button"])
                            .code(tag_code),
                    ],
                    render: move |values: DemoValues| {
                        let tag = values.str("component");
                        rsx! {
                            Box {
                                component: tag.clone(),
                                href: (tag == "a").then(|| HREF.to_string()),
                                target: (tag == "a").then(|| "_blank".to_string()),
                                r#type: (tag == "button").then(|| "button".to_string()),
                                sx: sx().padding("16px").background("grey.1").border_radius("md"),
                                "Styled entirely via sx"
                            }
                        }
                    },
                }
            }
        }
    }
}
