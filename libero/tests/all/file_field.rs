use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::FileField};

#[test]
fn a_file_field_draws_a_hidden_input_and_names_its_control() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachment",
                    placeholder: "No file picked",
                    accept: "image/*",
                    name: "avatar",
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // The control is named by the label, not by `for` - it is a div with a
    // role, which `for` cannot name.
    let label = attributes_of(&body, "label");
    assert!(!label.contains_key("for"), "{label:?}");
    let id = label["id"].trim_end_matches("-label").to_string();
    assert!(
        body.contains(&format!(r#"aria-labelledby="{id}-label""#)),
        "{body}"
    );
    assert!(body.contains(r#"role="button""#), "{body}");

    // The real input carries everything a form needs, and nothing else does.
    let input = attributes_of(&body, "input");
    assert_eq!(input["type"], "file");
    assert_eq!(input["name"], "avatar");
    assert_eq!(input["accept"], "image/*");
    assert_eq!(input["aria-hidden"], "true");

    assert!(body.contains("No file picked"), "{body}");
}

#[test]
fn a_dropzone_file_field_renders_its_prompt_instead_of_a_frame() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachments",
                    variant: "dropzone",
                    accept: "image/*",
                    multiple: true,
                    onchange: move |_| {},
                    "Drop files here"
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("Drop files here"), "{body}");
    assert!(body.contains(r#"role="button""#), "{body}");
    // A `false` bool is never pushed, so the attribute's presence is the test.
    assert!(body.contains("multiple=true"), "{body}");
    assert!(body.contains(r#"type="file""#), "{body}");
    // The prompt says what the attribute enforces, read off the attribute.
    assert!(body.contains(r#"data-slot="hint""#), "{body}");
    assert!(body.contains(">images<"), "{body}");
}
