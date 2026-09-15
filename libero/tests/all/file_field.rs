use std::collections::BTreeMap;

use crate::common::{attributes_of, body, classes_of, fake_files, has_rule_for, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::FileField};

/// Every attribute of the first element whose tag holds `needle`.
fn attributes_with(body: &str, needle: &str) -> BTreeMap<String, String> {
    let at = body
        .find(needle)
        .unwrap_or_else(|| panic!("no {needle}:\n{body}"));
    let start = body[..at].rfind('<').expect("its tag");
    let tag = body[start + 1..]
        .split(|c: char| c.is_whitespace() || c == '>')
        .next()
        .unwrap();
    attributes_of(&body[start..], tag)
}

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

    // Todo 529: the label names a group, which `for` cannot name, and the
    // Browse button inside it, after the label, by its own text.
    let label = attributes_of(&body, "label");
    assert!(!label.contains_key("for"), "{label:?}");
    let id = label["id"].trim_end_matches("-label").to_string();
    let group = attributes_with(&body, r#"role="group""#);
    assert_eq!(group["aria-labelledby"], format!("{id}-label"), "{body}");
    let button = attributes_of(&body, "button");
    assert_eq!(button["type"], "button", "{button:?}");
    assert_eq!(button["id"], id, "{button:?}");
    assert_eq!(
        button["aria-labelledby"],
        format!("{id}-label {id}"),
        "{button:?}"
    );
    assert!(body.contains("Browse files"), "{body}");
    assert!(!body.contains(r#"role="button""#), "{body}");

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
    // The surface is the group; the prompt is the Browse button's text.
    assert!(body.contains(r#"role="group""#), "{body}");
    let button = attributes_with(&body, r#"data-slot="browse""#);
    assert_eq!(button["type"], "button", "{button:?}");
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

    // The control's classes: the one element with `role="group"`.
    fn control_classes(body: &str) -> Vec<String> {
        let at = body.find(r#"role="group""#).expect("a control");
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
/// identifier. The lookup selects by attribute (todo 248); the focus move
/// needs a renderer, so this pins the field id the rows build on.
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

/// Todo 250: a `multiple` field draws the chip `MultiSelect` and `TagsField`
/// draw - the label in its `label` slot, and an x named after the file that
/// is out of the tab order.
#[test]
fn a_multiple_file_field_draws_the_shared_removable_chip() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachments",
                    multiple: true,
                    value: fake_files(&["a.txt", "b.txt"]),
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    assert!(
        body.contains(r#"<span data-slot="label">a.txt</span>"#),
        "{body}"
    );
    let named = body.find(r#""Remove b.txt""#).expect("the second chip's x");
    let x = attributes_of(&body[body[..named].rfind("<button").unwrap()..], "button");
    assert_eq!(x["aria-label"], "Remove b.txt", "{x:?}");
    assert_eq!(x["tabindex"], "-1", "{x:?}");
}

/// The dropzone's cards are a `list-style: none` list, which Safari with
/// VoiceOver stops announcing as a list without an explicit role.
#[test]
fn a_dropzones_card_list_keeps_its_list_semantics() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachments",
                    variant: "dropzone",
                    multiple: true,
                    value: fake_files(&["a.txt", "b.txt"]),
                    onchange: move |_| {},
                    "Drop files here"
                }
            }
        }
    }

    let body = body(&render(app));
    assert_eq!(attributes_of(&body, "ul")["role"], "list", "{body}");
    assert_eq!(body.matches("<li").count(), 2, "{body}");
}

/// The live region mounts empty: files already held when the field first
/// renders are not news, so only a later change is announced (todo 70).
#[test]
fn the_files_held_at_mount_are_not_announced() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    label: "Attachments",
                    multiple: true,
                    value: fake_files(&["a.txt", "b.txt"]),
                    onchange: move |_| {},
                }
            }
        }
    }
    let html = body(&render(app));

    assert!(html.contains("a.txt"), "the fixture holds no file:\n{html}");
    assert!(
        html.contains(r#"role="status"></span>"#) && !html.contains("Added"),
        "the live region is missing, or spoke at mount:\n{html}"
    );
}

/// Todo 529: ARIA allows `aria-required` on neither a group nor a button. The
/// input carries the native `required`, and the Browse button a hidden
/// "Required" in its description, ahead of the status.
#[test]
fn a_required_field_says_so_without_aria_required() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    id: "contract",
                    label: "Contract",
                    required: true,
                    status: "We cannot read that file.",
                    onchange: move |_| {},
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(!body.contains("aria-required"), "{body}");
    assert!(!body.contains("aria-activedescendant"), "{body}");
    assert!(
        attributes_of(&body, "input").contains_key("required"),
        "{body}"
    );
    let button = attributes_with(&body, r#"data-slot="browse""#);
    assert_eq!(
        button["aria-describedby"], "contract-required contract-status",
        "{button:?}"
    );
    assert_eq!(button["aria-invalid"], "true", "{button:?}");
    let required = attributes_with(&body, r#"id="contract-required""#);
    assert!(required.contains_key("hidden"), "{required:?}");
    assert!(body.contains(">Required</span>"), "{body}");
}

/// Todos 529, 530: the files are a list of focusable items with one tab stop,
/// so the focus lands on a file and hears its name.
#[test]
fn the_chips_are_a_list_with_one_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    id: "files",
                    label: "Attachments",
                    multiple: true,
                    value: fake_files(&["a.txt", "b.txt"]),
                    onchange: move |_| {},
                }
            }
        }
    }

    let body = body(&render(app));
    assert_eq!(attributes_of(&body, "ul")["role"], "list", "{body}");
    let first = attributes_with(&body, r#"id="files-file-0""#);
    let second = attributes_with(&body, r#"id="files-file-1""#);
    assert_eq!(first["tabindex"], "0", "{first:?}");
    assert_eq!(second["tabindex"], "-1", "{second:?}");
    assert_eq!(body.matches("<li").count(), 2, "{body}");
}

/// Read-only keeps the Browse button and the chips reachable and refuses;
/// disabled takes both out of the order natively.
#[test]
fn read_only_refuses_in_the_order_and_disabled_leaves_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                FileField {
                    id: "held",
                    label: "Held",
                    readonly: true,
                    multiple: true,
                    value: fake_files(&["a.txt"]),
                    onchange: move |_| {},
                }
                FileField {
                    id: "off",
                    label: "Off",
                    disabled: true,
                    multiple: true,
                    value: fake_files(&["a.txt"]),
                    onchange: move |_| {},
                }
            }
        }
    }

    let body = body(&render(app));
    let held = attributes_with(&body, r#"id="held""#);
    assert_eq!(held["aria-disabled"], "true", "{held:?}");
    assert!(!held.contains_key("disabled"), "{held:?}");
    assert_eq!(
        attributes_with(&body, r#"id="held-file-0""#)["tabindex"],
        "0"
    );

    let off = attributes_with(&body, r#"id="off""#);
    assert!(off.contains_key("disabled"), "{off:?}");
    assert!(!off.contains_key("aria-disabled"), "{off:?}");
    assert!(
        !attributes_with(&body, r#"id="off-file-0""#).contains_key("tabindex"),
        "{body}"
    );
}
