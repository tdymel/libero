//! The chrome `use_field` puts around a control: the five slots, and the a11y
//! wiring that ties them to it. `TextField` carries most of the cases;
//! `Select` covers what changed when it was ported off its wrapping `<label>`.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Options, PasswordField, Select, TextField},
};

#[test]
fn every_slot_renders_and_the_control_names_all_three_descriptions() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField {
                    label: "Email",
                    description: "The address we send the invoice to.",
                    helper: "Work addresses only.",
                    status: "Not a valid address.",
                    required: true,
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let input = attributes_of(&body, "input");
    let id = &input["id"];

    assert_eq!(attributes_of(&body, "label")["for"], *id);
    assert_eq!(
        input["aria-describedby"],
        format!("{id}-description {id}-helper {id}-status")
    );
    assert_eq!(input["aria-invalid"], "true");
    assert_eq!(input["aria-required"], "true");

    // In DOM order: label, description, control, helper, status.
    let order = |needle: &str| body.find(needle).unwrap_or_else(|| panic!("no {needle}"));
    assert!(order("<label") < order(r#"data-slot="description""#));
    assert!(order(r#"data-slot="description""#) < order("<input"));
    assert!(order("<input") < order(r#"data-slot="helper""#));
    assert!(order(r#"data-slot="helper""#) < order(r#"data-slot="status""#));

    // `aria-required` carries it to AT, so the asterisk is hidden from it.
    assert!(body.contains(r#"data-slot="required""#));
    assert!(body.contains(r#"aria-hidden="true""#));
}

#[test]
fn a_bare_field_carries_no_description_and_no_invalid_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name" }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");

    assert!(!input.contains_key("aria-describedby"), "{input:?}");
    assert!(!input.contains_key("aria-invalid"), "{input:?}");
    assert!(!input.contains_key("aria-required"), "{input:?}");
    assert!(!body.contains("data-slot"), "{body}");
}

#[test]
fn a_warning_is_described_but_not_invalid() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField {
                    label: "Name",
                    status: libero::components::FieldStatus::Warning("Unusual.".to_string()),
                }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");

    assert!(input["aria-describedby"].ends_with("-status"));
    // `aria-invalid` on a warning would announce the field as broken.
    assert!(!input.contains_key("aria-invalid"), "{input:?}");
    assert!(body.contains("Unusual."));
}

#[test]
fn the_callers_own_described_by_wins_outright() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField {
                    label: "Name",
                    helper: "Ours would name this.",
                    "aria-describedby": "elsewhere",
                }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");

    assert_eq!(input["aria-describedby"], "elsewhere");
    // Still rendered - the caption props stay visual when the caller takes
    // over the wiring.
    assert!(body.contains("Ours would name this."));
}

#[test]
fn a_callers_id_is_what_the_label_points_at() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name", helper: "Hint.", id: "chosen" }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(attributes_of(&body, "input")["id"], "chosen");
    assert_eq!(attributes_of(&body, "label")["for"], "chosen");
    assert_eq!(
        attributes_of(&body, "input")["aria-describedby"],
        "chosen-helper"
    );
}

#[test]
fn markup_captions_render_but_name_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField {
                    label: "Name",
                    helper: rsx! { strong { "bold hint" } },
                }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");

    assert!(body.contains("<strong>bold hint</strong>"), "{body}");
    // A caller who passes markup owns its a11y, so it joins no id list.
    assert!(!input.contains_key("aria-describedby"), "{input:?}");
}

#[test]
fn omitting_value_leaves_the_input_uncontrolled() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name" }
            }
        }
    }

    let body = body(&render(app));

    assert!(
        !attributes_of(&body, "input").contains_key("value"),
        "an omitted `value` must not render one - that would freeze the field"
    );
}

#[test]
fn a_controlled_value_renders() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name", value: "Ada" }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(attributes_of(&body, "input")["value"], "Ada");
}

#[test]
fn the_frame_wraps_the_control_with_its_leading_and_trailing_slots() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField {
                    label: "Amount",
                    leading: rsx! { span { "before" } },
                    trailing: rsx! { span { "after" } },
                }
            }
        }
    }

    let body = body(&render(app));
    let order = |needle: &str| body.find(needle).unwrap_or_else(|| panic!("no {needle}"));

    assert!(order(r#"data-slot="leading""#) < order("<input"));
    assert!(order("<input") < order(r#"data-slot="trailing""#));
    assert!(body.contains("before") && body.contains("after"));

    // The label still names the control, not the frame around it.
    assert_eq!(
        attributes_of(&body, "label")["for"],
        attributes_of(&body, "input")["id"]
    );
}

#[test]
fn a_field_with_no_slots_renders_an_empty_frame() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name" }
            }
        }
    }

    let body = body(&render(app));

    // The frame is always there - it carries the border every field shares -
    // but it costs no node for a slot nothing filled.
    assert!(!body.contains("data-slot"), "{body}");
    // The control is inside the frame, not a sibling of the label.
    assert!(body.contains("</label><div"), "{body}");
}

#[test]
fn the_frame_draws_the_focus_ring_and_the_control_does_not() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name" }
            }
        }
    }

    let html = render(app);
    let frame_class = attributes_of(&body(&html), "div")["class"].clone();
    let control_class = attributes_of(&body(&html), "input")["class"].clone();

    // The ring lives on the frame, keyed off a descendant's `:focus-visible` -
    // there is no `:focus-visible-within`.
    assert!(
        html.contains(":has(:focus-visible)"),
        "the frame must draw the ring"
    );
    // Both would draw one, at two different offsets, if the control kept the
    // shared ring class.
    let shared: Vec<&str> = frame_class
        .split_whitespace()
        .filter(|class| {
            control_class
                .split_whitespace()
                .any(|other| other == *class)
        })
        .collect();
    assert!(
        shared.is_empty(),
        "the control still shares {shared:?} with an ancestor that rings"
    );
}

#[derive(Clone, PartialEq, Options)]
enum Pick {
    First,
    Second,
}

/// `Select`'s root was a wrapping `<label>` before R5. A wrapping label
/// swallows clicks on anything nested in the frame, which is why every field
/// uses a `<label for>`/`id` pair instead.
#[test]
fn a_select_names_its_control_through_for_rather_than_wrapping_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Select {
                    label: "Plan",
                    description: "Billed monthly.",
                    status: "Pick one.",
                    required: true,
                    value: Pick::Second,
                    onchange: move |_| {},
                }
            }
        }
    }

    let body = body(&render(app));
    let select = attributes_of(&body, "select");
    let id = &select["id"];

    assert_eq!(attributes_of(&body, "label")["for"], *id);
    assert_eq!(
        select["aria-describedby"],
        format!("{id}-description {id}-status")
    );
    assert_eq!(select["aria-invalid"], "true");
    assert_eq!(select["aria-required"], "true");
    // The control sits in the frame, and the label is its sibling - not its
    // parent.
    let after_label = body.split("</label>").nth(1).expect("a label");
    let frame = after_label.split("<select").next().expect("a select");
    assert!(frame.contains("<div"), "{body}");
}

/// Both fields read one `FieldDefaults` scale now, so a `Select` beside a
/// `TextField` lines up by construction.
#[test]
fn a_select_and_a_text_field_share_one_size_scale() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { size: "lg" }
                Select { size: "lg", value: Pick::First, onchange: move |_| {} }
            }
        }
    }

    let html = render(app);

    assert!(html.contains("--lsx-field-height-lg"), "{html}");
    assert!(!html.contains("--lsx-select-height"), "{html}");
}

/// `PasswordField` is a `TextField` with a narrower contract - the first use of
/// the specialization path the field foundation exists for.
#[test]
fn a_password_field_hides_its_value_and_offers_a_reveal_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PasswordField { label: "Password", helper: "At least 8 characters." }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");
    let id = &input["id"];

    assert_eq!(input["type"], "password");
    assert_eq!(attributes_of(&body, "label")["for"], *id);
    assert_eq!(input["aria-describedby"], format!("{id}-helper"));

    // The toggle is a real button in the trailing slot, named for what the
    // click does rather than for the current state.
    let button = attributes_of(&body, "button");
    assert_eq!(button["aria-label"], "Show password");
    let order = |needle: &str| body.find(needle).unwrap_or_else(|| panic!("no {needle}"));
    assert!(order("<input") < order(r#"data-slot="trailing""#));
}
