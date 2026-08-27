//! The chrome `use_field` puts around a control: the five slots, and the a11y
//! wiring that ties them to it. `TextField` carries most of the cases;
//! `Select` covers what changed when it was ported off its wrapping `<label>`.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{NumberField, NumberValue, Options, PasswordField, Select, TextField, Textarea},
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

/// The frame was built around an `<input>`; a `<textarea>` is what proves it
/// holds a control that is not one line tall.
#[test]
fn a_textarea_renders_its_rows_inside_the_frame() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Textarea { label: "Notes", rows: 5, status: "Say something." }
            }
        }
    }

    let body = body(&render(app));
    let textarea = attributes_of(&body, "textarea");
    let id = &textarea["id"];

    assert_eq!(textarea["rows"], "5");
    assert_eq!(attributes_of(&body, "label")["for"], *id);
    assert_eq!(textarea["aria-describedby"], format!("{id}-status"));
    assert_eq!(textarea["aria-invalid"], "true");

    // Inside the frame, not a sibling of the label.
    let after_label = body.split("</label>").nth(1).expect("a label");
    let frame = after_label.split("<textarea").next().expect("a textarea");
    assert!(frame.contains("<div"), "{body}");
}

/// `NumberField` is generic over the caller's number type, and every primitive
/// already implements `NumberValue` - `i32` here needs no code at all.
#[test]
fn a_number_field_is_a_spinbutton_carrying_its_range() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NumberField {
                    label: "Quantity",
                    min: 1i32,
                    max: 99i32,
                    steppers: true,
                    value: 4i32,
                    onchange: move |_| {},
                }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");

    assert_eq!(input["value"], "4");
    // `min`/`max` mean nothing on a text input, so the range is ARIA's.
    assert_eq!(input["role"], "spinbutton");
    assert_eq!(input["aria-valuenow"], "4");
    assert_eq!(input["aria-valuemin"], "1");
    assert_eq!(input["aria-valuemax"], "99");
    assert_eq!(input["inputmode"], "decimal");
    assert_eq!(attributes_of(&body, "label")["for"], input["id"]);

    // Both steppers are real buttons in the trailing slot.
    assert!(body.contains(r#"aria-label="Increase""#), "{body}");
    assert!(body.contains(r#"aria-label="Decrease""#), "{body}");
    let order = |needle: &str| body.find(needle).unwrap_or_else(|| panic!("no {needle}"));
    assert!(order("<input") < order(r#"data-slot="trailing""#));
}

/// The empty field is a state the type can hold, not an empty string the
/// caller has to special-case.
#[test]
fn a_number_field_without_a_value_renders_empty() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NumberField {
                    label: "Weight",
                    placeholder: "kg",
                    value: None::<f64>,
                    onchange: move |_: f64| {},
                }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");

    assert_eq!(input["value"], "");
    assert_eq!(input["placeholder"], "kg");
    assert!(!body.contains("aria-valuenow"), "{body}");
}

/// A custom `NumberValue` is `default_step` plus whatever genuinely differs -
/// here the display, because cents are stored whole and shown with a point.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Cents(i64);

impl std::ops::Add for Cents {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Cents(self.0 + other.0)
    }
}

impl std::ops::Sub for Cents {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Cents(self.0 - other.0)
    }
}

impl std::str::FromStr for Cents {
    type Err = std::num::ParseFloatError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Cents((text.parse::<f64>()? * 100.0).round() as i64))
    }
}

impl std::fmt::Display for Cents {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{:02}", self.0 / 100, self.0 % 100)
    }
}

impl NumberValue for Cents {
    fn default_step() -> Self {
        Cents(50)
    }
}

#[test]
fn a_custom_number_value_formats_itself() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NumberField {
                    label: "Price",
                    value: Cents(1234),
                    onchange: move |_| {},
                }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(attributes_of(&body, "input")["value"], "12.34");
}

/// The steppers are opt-in: a number is usually typed, and two buttons are the
/// most expensive thing a field can carry.
#[test]
fn a_number_field_has_no_steppers_until_asked() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NumberField { label: "Quantity", value: 1i32, onchange: move |_| {} }
            }
        }
    }

    let body = body(&render(app));

    assert!(!body.contains("<button"), "{body}");
    assert!(!body.contains("data-slot"), "{body}");
    // The keys still step it - that is what `role="spinbutton"` promises.
    assert_eq!(attributes_of(&body, "input")["role"], "spinbutton");
}
