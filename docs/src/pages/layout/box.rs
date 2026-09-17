use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Text},
    sx::sx,
};

const HREF: &str = "https://dioxuslabs.com";
// snippet: in Box { .., "Styled entirely via sx" }
const SX: &str = r#"sx: sx().padding("16px").background("muted.1").border_radius("md")"#;

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
            source: "libero/src/components/layout/box.rs",
            markdown: "/md/box.md",
            properties: vec![props("Box", vec![
                prop("variables", "Variables")
                    .doc("Custom properties set on the element's `style`, so `sx` can use a changing value without a new class per value."),
                prop("component", "HtmlTag")
                    .default("div")
                    .doc("The element to render, any of the 111 HTML5 element names. The 28 outside the default set (document metadata, embedded and media content, `template`, `slot`, bidi and ruby) need the `full-polymorphism` feature and render as a `div` without it."),
                prop("framework_sx", "&'static StaticSx")
                    .doc("Base styles for a component built on `Box`. They sit on a CSS layer below `sx`, so a caller's `sx` still wins."),
                prop("style", "String")
                    .doc("Raw `style` declarations, applied after `variables`."),
                prop("alt", "String")
                    .doc("The `img` alt text."),
                prop("r#type", "String")
                    .doc("The `button` type, such as `\"submit\"`."),
                prop("children", "Element").doc("The element's content."),
            ]).extends("img, a and button")],
            lead: rsx! {
                Text {
                    "The primitive every other component is built on. "
                    Code { source: "component" }
                    " picks the tag, and attributes like "
                    Code { source: "href" }
                    " or "
                    Code { source: "src" }
                    " pass through to it. "
                    Code { source: "Box" }
                    " has no look of its own. Everything visible comes from "
                    Code { source: "sx" }
                    " and "
                    Code { source: "states" }
                    "."
                }
            },
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
                            sx: sx().padding("16px").background("muted.1").border_radius("md"),
                            "Styled entirely via sx"
                        }
                    }
                },
            }
        }
    }
}
