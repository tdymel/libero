use crate::common::{attributes_of, body, classes_of, has_rule_for, render};

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

/// The two variants prepare different boxes, so their hooks must not share one
/// scope's slots: each draws in a scope of its own, and a `variant` switch
/// remounts it. This flips the variant on every pass. Each pass must draw the
/// variant it names, and the `Input` control's rule must go when the dropzone
/// takes over - with shared slots the `Input` arm's second box stayed
/// registered under the dropzone, its drop never run.
#[test]
fn switching_variant_at_runtime_swaps_the_control_and_its_styles() {
    #[component]
    fn Switching() -> Element {
        let mut step = use_signal(|| 0u8);
        use_effect(move || {
            if step() < 3 {
                step += 1;
            }
        });
        let variant = match step() % 2 {
            0 => "input",
            _ => "dropzone",
        };
        rsx! {
            div { "step {step}" }
            FileField { label: "Attachment", variant, onchange: move |_| {} }
        }
    }

    fn app() -> Element {
        rsx! { LiberoProvider { Switching {} } }
    }

    // The control's classes: the one element with `role="button"`.
    fn control_classes(body: &str) -> Vec<String> {
        let at = body.find(r#"role="button""#).expect("a control");
        let start = body[..at].rfind('<').expect("its tag");
        classes_of(&body[start..], "div")
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let input_classes = control_classes(&body(&dioxus_ssr::render(&dom)));
    let mut seen = Vec::new();
    for _ in 0..4 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        let body = body(&html);
        let step = body
            .split("step ")
            .nth(1)
            .and_then(|rest| rest.chars().next());
        let dropzone = body.contains("Drop a file here, or click to pick");
        let input_rules = input_classes.iter().any(|class| has_rule_for(&html, class));
        seen.push((step, dropzone, input_rules));
    }

    assert_eq!(seen.last(), Some(&(Some('3'), true, false)), "{seen:?}");
    for (step, dropzone, input_rules) in &seen {
        let odd = step
            .and_then(|step| step.to_digit(10))
            .is_some_and(|step| step % 2 == 1);
        assert_eq!((*dropzone, *input_rules), (odd, !odd), "{seen:?}");
    }
}

/// A removed row hands focus to the next row's button, found by
/// `{id}-remove-{n}` on the field's id, and a caller's id need not be a CSS
/// identifier. The lookup selects by attribute (todo 248); rows need picked
/// files, which SSR cannot supply, so this pins the field id they build on.
#[test]
fn a_caller_id_that_is_no_css_identifier_is_the_field_id() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField { id: "1-faq", label: "Attachment", onchange: move |_| {} }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("id=\"1-faq-label\""), "{body}");
}
