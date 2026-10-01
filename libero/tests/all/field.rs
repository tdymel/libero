//! The chrome `use_field` puts around a control: the five slots, and the a11y
//! wiring that ties them to it. `TextField` carries most of the cases;
//! `NativeSelect` covers what changed when it was ported off its wrapping `<label>`.

use crate::common::{attributes_of, body, nth_attributes, render, tag_with, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Checkbox, Fields, Fieldset, Form, MultiSelect, NativeSelect, NumberField, NumberValue,
        OptionItem, OptionList, Options, PasswordField, PinField, Radio, RadioGroup,
        SegmentedControl, Select, Slider, Switch, TextField, Textarea,
    },
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
    assert_eq!(
        tag_with(&body, r#"data-slot="required""#)["aria-hidden"],
        "true"
    );
}

/// A caption or frame slot was drawn. The label, frame and control always carry one.
fn fills_a_slot(body: &str) -> bool {
    ["description", "helper", "status", "leading", "trailing"]
        .iter()
        .any(|slot| body.contains(&format!(r#"data-slot="{slot}""#)))
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
    assert!(!fills_a_slot(&body), "{body}");
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

/// A text slot describes the input only when the caller flags it: an icon's
/// or a button's name must not (todo 496).
#[test]
fn a_flagged_slot_describes_the_input_in_reading_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField {
                    label: "Website",
                    helper: "Your own domain.",
                    leading: rsx! { "https://" },
                    describe_leading: true,
                    trailing: rsx! { "kg" },
                }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");
    let id = &input["id"];

    assert_eq!(
        input["aria-describedby"],
        format!("{id}-leading {id}-helper")
    );
    assert!(
        body.contains(&format!(r#"id="{id}-leading">https://"#)),
        "{body}"
    );
    assert!(!body.contains(&format!("{id}-trailing")), "{body}");
}

#[test]
fn an_empty_flagged_slot_describes_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Weight", describe_trailing: true }
            }
        }
    }

    let input = attributes_of(&body(&render(app)), "input");

    assert!(!input.contains_key("aria-describedby"), "{input:?}");
}

/// `aria-describedby` is a list, so the caller's ids join the field's rather
/// than replacing them - dropping ours took the validation message away from
/// AT on a field that is `aria-invalid`.
#[test]
fn the_callers_own_described_by_joins_the_fields_own() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField {
                    label: "Name",
                    helper: "Ours names this.",
                    status: "Too short.",
                    "aria-describedby": "elsewhere",
                }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");
    let id = &input["id"];

    // The caller's first: their hint is the one they asked to be read first.
    assert_eq!(
        input["aria-describedby"],
        format!("elsewhere {id}-helper {id}-status")
    );
    assert_eq!(input["aria-invalid"], "true");
    assert!(body.contains("Ours names this."));

    // One attribute, not two. This is what makes the fix renderer-independent:
    // a duplicate is raced, and SSR keeps the first while the DOM keeps the
    // last, so the same markup would be wrong differently in each. With one
    // attribute there is nothing to race.
    assert_eq!(body.matches("aria-describedby=").count(), 1, "{body}");
}

/// The same rule on the other side of the pair: `Fieldset` set only its own,
/// and a component's attribute beats the caller's, so there the caller's ids
/// were the ones that went missing.
#[test]
fn a_fieldsets_described_by_joins_the_callers_too() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Fieldset::<()> { label: "Address", helper: "Ours names this.", "aria-describedby": "elsewhere",
                    TextField { label: "Street" }
                }
            }
        }
    }

    let body = body(&render(app));
    let fieldset = attributes_of(&body, "fieldset");
    let id = &fieldset["id"];

    assert_eq!(
        fieldset["aria-describedby"],
        format!("elsewhere {id}-helper")
    );
    assert_eq!(body.matches("aria-describedby=").count(), 1, "{body}");
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
    assert!(!fills_a_slot(&body), "{body}");
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

    // The ring is the frame's, drawn by an overlay after the control and
    // keyed off the control's `:focus-visible` - there is no
    // `:focus-visible-within`, and `:has()` never matches natively.
    assert!(
        html.contains(" :focus-visible ~ [data-ring]{outline:"),
        "the frame must draw the ring: {html}"
    );
    assert!(body(&html).contains("data-ring"), "{html}");
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

/// `NativeSelect`'s root was a wrapping `<label>` before R5. A wrapping label
/// swallows clicks on anything nested in the frame, which is why every field
/// uses a `<label for>`/`id` pair instead.
#[test]
fn a_select_names_its_control_through_for_rather_than_wrapping_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect {
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

/// Both fields read one `FieldDefaults` scale now, so a `NativeSelect` beside a
/// `TextField` lines up by construction.
#[test]
fn a_select_and_a_text_field_share_one_size_scale() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { size: "lg" }
                NativeSelect { size: "lg", value: Pick::First, onchange: move |_| {} }
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
                    onchange: move |_: Option<f64>| {},
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

/// `Formats::GERMAN` writes `1,5`; `aria-valuenow` stays a machine number.
#[test]
fn a_number_field_writes_the_formats_decimal_separator() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { formats: &libero::localization::Formats::GERMAN,
                NumberField { label: "Weight", value: 1.5, onchange: move |_| {} }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");

    assert_eq!(input["value"], "1,5");
    assert_eq!(input["aria-valuenow"], "1.5");
}

/// A step at either end of an integer type stays there. Unchecked, `0u32 - 1`
/// panics in a debug build and wraps to `u32::MAX` in release.
#[test]
fn an_integer_step_saturates_at_the_ends_of_its_type() {
    assert_eq!(0u8.step_down(1), 0);
    assert_eq!(u8::MAX.step_up(1), u8::MAX);
    assert_eq!(250u8.step_up(10), u8::MAX);
    assert_eq!(0u32.step_down(1), 0);
    assert_eq!(u32::MAX.step_up(1), u32::MAX);
    assert_eq!(i32::MIN.step_down(1), i32::MIN);
    assert_eq!(i32::MAX.step_up(1), i32::MAX);
    assert_eq!((-1i32).step_up(1), 0);
}

/// A saturated step still lands inside the caller's range.
#[test]
fn a_saturated_step_is_then_clamped_to_min_and_max() {
    assert_eq!(0u8.step_down(1).clamp_between(Some(5), Some(10)), 5);
    assert_eq!(u8::MAX.step_up(1).clamp_between(Some(5), Some(10)), 10);
    assert_eq!(i32::MIN.step_down(1).clamp_between(Some(-3), None), -3);
}

/// A float step lands on the step's precision, or the value's where that is
/// finer, so binary noise never reaches the field.
#[test]
fn a_float_step_rounds_away_binary_noise() {
    assert_eq!(0.1f64.step_up(0.2).to_string(), "0.3");
    assert_eq!(0.3f64.step_down(0.1).to_string(), "0.2");
    assert_eq!(1.1f64.step_up(0.1).step_up(0.1).to_string(), "1.3");
    // The value's own digits survive a coarser step.
    assert_eq!(1.234f64.step_up(0.1).to_string(), "1.334");
    assert_eq!(2.5f64.step_down(1.0).to_string(), "1.5");
    assert_eq!(0.1f32.step_up(0.2).to_string(), "0.3");
    // Ten presses of 0.1 from zero end on 1, not 0.9999999999999999.
    let ten = (0..10).fold(0.0f64, |value, _| value.step_up(0.1));
    assert_eq!(ten.to_string(), "1");
    assert_eq!(f64::MAX.step_up(f64::MAX), f64::INFINITY);
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
    assert!(!fills_a_slot(&body), "{body}");
    // The keys still step it - that is what `role="spinbutton"` promises.
    assert_eq!(attributes_of(&body, "input")["role"], "spinbutton");
}

/// The reveal button is the one affordance this component exists for, so it is
/// on by default - but a confirmation field beside a revealed twin has nothing
/// to add.
#[test]
fn a_password_field_can_drop_its_reveal_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PasswordField { label: "Repeat password", reveal_button: false }
            }
        }
    }

    let body = body(&render(app));

    assert!(!body.contains("<button"), "{body}");
    assert!(!fills_a_slot(&body), "{body}");
    assert_eq!(attributes_of(&body, "input")["type"], "password");
}

/// A checkbox has no frame: the box sits beside the label, and the captions
/// stay under both. The order in the DOM is what the grid places.
#[test]
fn a_checkbox_puts_its_control_before_the_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox {
                    label: "Email me",
                    description: "About once a month.",
                    helper: "You can unsubscribe later.",
                    checked: true,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let input = attributes_of(&body, "input");
    let id = &input["id"];

    assert_eq!(input["type"], "checkbox");
    assert!(body.contains("checked=true"), "{body}");
    assert_eq!(attributes_of(&body, "label")["for"], *id);
    assert_eq!(
        input["aria-describedby"],
        format!("{id}-description {id}-helper")
    );

    let order = |needle: &str| body.find(needle).unwrap_or_else(|| panic!("no {needle}"));
    assert!(order("<input") < order("<label"));
    assert!(order("<label") < order(r#"data-slot="description""#));
    assert!(order(r#"data-slot="description""#) < order(r#"data-slot="helper""#));
}

/// `indeterminate` is a DOM property with no attribute, so the native input
/// can never carry it. The state is Rust's, and AT reads it from ARIA.
#[test]
fn an_indeterminate_checkbox_reads_as_mixed_and_is_not_checked() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox {
                    label: "Select all",
                    checked: true,
                    indeterminate: true,
                    onchange: move |_| {},
                }
            }
        }
    }

    let body = body(&render(app));
    let input = attributes_of(&body, "input");

    assert_eq!(input["aria-checked"], "mixed");
    assert!(!body.contains("checked=true"), "{body}");
}

/// The box is decoration - the input owns the name, the state and the
/// keyboard - so it must not reach assistive tech.
#[test]
fn the_checkbox_box_is_hidden_from_assistive_tech() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Checkbox { aria_label: "Accept", onchange: move |_| {} }
            }
        }
    }

    let body = body(&render(app));

    assert!(!body.contains("<label"), "{body}");
    assert_eq!(attributes_of(&body, "input")["aria-label"], "Accept");
    assert!(body.contains(r#"<span aria-hidden="true""#) || body.contains(r#"aria-hidden="true""#));
}

/// The switch is a checkbox with `role="switch"`, and since the port it wears
/// the same chrome every field does - the label beside the track, the captions
/// under both.
#[test]
fn a_switch_is_a_checkbox_its_label_points_at() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Switch { label: "wifi", checked: true, onchange: move |_| {} }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let input = attributes_of(&body, "input");

    assert_eq!(input["type"], "checkbox");
    assert_eq!(input["role"], "switch");
    assert!(body.contains("checked=true"), "{body}");
    assert_eq!(attributes_of(&body, "label")["for"], input["id"]);
    assert!(body.contains(">wifi<"), "{body}");

    // The control comes first, so the grid can put the track beside the label.
    let order = |needle: &str| body.find(needle).unwrap_or_else(|| panic!("no {needle}"));
    assert!(order("<input") < order("<label"));
}

#[test]
fn a_switch_without_a_label_is_named_by_its_aria_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Switch { aria_label: "Airplane mode", checked: false, onchange: move |_| {} }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(attributes_of(&body, "input")["aria-label"], "Airplane mode");
    assert!(!body.contains("<label"), "{body}");
}

/// `for` names a labelable element, and the slider's control is a span with
/// `role="slider"`. So the label is named instead, and the thumb points at it.
#[test]
fn a_slider_is_named_by_labelledby_rather_than_for() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    label: "Volume",
                    helper: "Loud enough for a room.",
                    value: 25.0,
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let label = attributes_of(&body, "label");
    let id = label["id"].trim_end_matches("-label").to_string();

    assert!(!label.contains_key("for"), "{label:?}");
    assert!(
        body.contains(&format!(r#"aria-labelledby="{id}-label""#)),
        "{body}"
    );
    // The helper, then the thumb's own value bubble (todo 309).
    assert!(
        body.contains(&format!(r#"aria-describedby="{id}-helper "#)),
        "{body}"
    );
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
    Team,
}

/// The three things a lone `Radio` cannot own: the shared `name`, the single
/// tab stop, and the grouping the question is announced through.
#[test]
fn a_radio_group_shares_one_name_and_one_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                RadioGroup {
                    label: "Plan",
                    helper: "Change it later.",
                    value: Plan::Pro,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let group = attributes_of(&body, "div");
    let label = attributes_of(&body, "label");
    let id = label["id"].trim_end_matches("-label").to_string();

    // The wrapper is the first div; the group itself is the second.
    let group_attributes = nth_attributes(&body, "div", 1);
    assert_eq!(group_attributes["role"], "radiogroup");
    assert_eq!(group_attributes["aria-labelledby"], format!("{id}-label"));
    assert_eq!(group_attributes["aria-describedby"], format!("{id}-helper"));
    assert!(!group.contains_key("role"), "{group:?}");

    let names: Vec<&str> = body
        .match_indices("name=\"")
        .map(|(at, _)| {
            let rest = &body[at + 6..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect();
    assert_eq!(names.len(), 3, "{body}");
    assert!(names.iter().all(|name| *name == names[0]), "{names:?}");

    // One tab stop: the selected option. The other two are out of the order.
    assert_eq!(body.matches(r#"tabindex="0""#).count(), 1, "{body}");
    assert_eq!(body.matches(r#"tabindex="-1""#).count(), 2, "{body}");
    assert!(body.contains(r#"data-radio-index="1""#), "{body}");

    // Todo 20: each radio posts `Options::value`, not the browser's `on`.
    let values: Vec<String> = tags_with(&body, "<input")
        .into_iter()
        .map(|input| input["value"].clone())
        .collect();
    assert_eq!(values, ["Free", "Pro", "Team"]);
}

/// Todo 371: the per-option flag rides on the option, and `options` left
/// unset is not an empty list - it still means every `Options::options()`.
#[test]
fn a_radio_group_disables_the_option_its_item_flagged() {
    fn unset() -> Element {
        rsx! {
            LiberoProvider {
                RadioGroup { label: "Plan", value: Some(Plan::Free), onchange: move |_: Plan| {} }
            }
        }
    }

    fn flagged() -> Element {
        rsx! {
            LiberoProvider {
                RadioGroup {
                    label: "Plan",
                    value: Some(Plan::Free),
                    onchange: move |_: Plan| {},
                    options: OptionList::new([
                        Plan::Free.into(),
                        Plan::Pro.into(),
                        OptionItem::new(Plan::Team).disabled(true),
                    ]),
                }
            }
        }
    }

    // Unset lists the enum's own three options, none of them off.
    let unset = body(&render(unset));
    let inputs = tags_with(&unset, "<input");
    assert_eq!(inputs.len(), 3, "{unset}");
    assert!(
        inputs.iter().all(|input| !input.contains_key("disabled")),
        "{inputs:?}"
    );

    // Only the flagged option is refused, and the selected one keeps the one
    // tab stop the group has.
    let flagged = body(&render(flagged));
    assert_eq!(flagged.matches("<input").count(), 3);
    let team = tags_with(&flagged, r#"data-radio-index="2""#);
    assert_eq!(team.len(), 1, "{flagged}");
    assert!(team[0].contains_key("disabled"), "{team:?}");
    assert_eq!(flagged.matches(r#"tabindex="0""#).count(), 1, "{flagged}");
}

/// Nothing selected is what an unanswered question looks like - but the group
/// still needs a way in, so the first option holds the tab stop.
#[test]
fn an_unanswered_radio_group_still_has_a_way_in() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                RadioGroup { label: "Plan", options: vec![Plan::Free, Plan::Pro], onchange: move |_| {} }
            }
        }
    }

    let body = body(&render(app));

    assert!(!body.contains("checked=true"), "{body}");
    assert_eq!(body.matches(r#"tabindex="0""#).count(), 1, "{body}");
    assert!(
        body.find(r#"tabindex="0""#) < body.find(r#"tabindex="-1""#),
        "{body}"
    );
}

/// A segmented control is a radio group with the field around it: the label
/// names the group, the captions describe it, and `disabled` reaches every
/// segment rather than only the ones its `options` flag.
#[test]
fn a_segmented_control_wears_the_field_around_its_radiogroup() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    label: "Plan",
                    helper: "Change it later.",
                    status: "Pick a plan.",
                    required: true,
                    disabled: true,
                    value: Plan::Pro,
                    onchange: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let label = attributes_of(&body, "label");
    let id = label["id"].trim_end_matches("-label").to_string();

    // The wrapper is the first div; the group itself is the second.
    let group = nth_attributes(&body, "div", 1);
    assert_eq!(group["role"], "radiogroup");
    assert_eq!(group["aria-labelledby"], format!("{id}-label"));
    assert!(
        group["aria-describedby"].contains(&format!("{id}-helper")),
        "{group:?}"
    );
    assert!(
        group["aria-describedby"].contains(&format!("{id}-status")),
        "{group:?}"
    );
    assert_eq!(group["aria-invalid"], "true");
    assert_eq!(group["aria-required"], "true");

    let radios = tags_with(&body, "<input");
    assert_eq!(radios.len(), 3, "{body}");
    assert!(
        radios.iter().all(|radio| radio.contains_key("disabled")),
        "{radios:?}"
    );
}

/// `full_width` fills the wrapper, so the wrapper has to fill its parent too -
/// inside a centring parent it would otherwise shrink to the strip.
#[test]
fn a_full_width_segmented_control_stretches_its_field_wrapper() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl { full_width: true, value: Plan::Pro, onchange: move |_| {} }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let wrapper = attributes_of(&body, "div");
    assert!(wrapper["data-state"].contains("full-width"), "{wrapper:?}");
    assert!(
        html.contains("[data-state~=\"full-width\"]{width:100%"),
        "{html}"
    );
}

/// The hidden radio is `position: absolute` beside its label, so the group has
/// to be its containing block. Without one the radio is laid out against the
/// viewport: it escapes every scroll container above it, makes the document
/// itself scrollable, and focusing it - which a click on a segment does -
/// scrolls the whole page instead of nothing.
#[test]
fn a_segmented_control_contains_its_hidden_radios() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl { value: Plan::Pro, onchange: move |_| {} }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let group = nth_attributes(&body, "div", 1);
    assert_eq!(group["role"], "radiogroup");

    let class = group["class"]
        .split_whitespace()
        .find(|class| html.contains(&format!(".{class} > input{{position:absolute")))
        .unwrap_or_else(|| panic!("no class positions the radio: {group:?}"));
    let rule = format!(".{class}{{");
    let base = &html[html
        .find(&rule)
        .unwrap_or_else(|| panic!("no {rule} in {html}"))..];
    let base = &base[..base.find('}').unwrap()];
    assert!(base.contains("position:relative"), "{base}");
}

#[derive(Clone, Copy, PartialEq, Default, Options)]
enum Density {
    Compact,
    #[default]
    Cosy,
    Roomy,
}

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Layout {
    density: Density,
}

/// Bound through a path, the control reads its selection out of the form's
/// value and posts under the path's name - no `value` and no `onchange`.
#[test]
fn a_bound_segmented_control_reads_its_form_value_and_posts_its_name() {
    fn app() -> Element {
        let layout = use_store(|| Layout {
            density: Density::Roomy,
        });
        rsx! {
            LiberoProvider {
                Form {
                    value: layout,
                    SegmentedControl { name: Layout::FIELDS.density() }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert_eq!(html.matches(r#"name="density""#).count(), 3, "{html}");
    let checked: Vec<bool> = html
        .match_indices("<input")
        .map(|(at, _)| html[at..at + html[at..].find('>').unwrap()].contains("checked"))
        .collect();
    assert_eq!(checked, [false, false, true], "{html}");
}

/// A radio is turned off by another being turned on, so `onselect` reports a
/// pick and never an unpick - and a standalone one carries the field chrome.
#[test]
fn a_standalone_radio_is_a_field_of_its_own() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Radio {
                    label: "Free",
                    description: "One seat.",
                    checked: true,
                    onselect: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let input = attributes_of(&body, "input");
    let id = &input["id"];

    assert_eq!(input["type"], "radio");
    assert!(body.contains("checked=true"), "{body}");
    assert_eq!(attributes_of(&body, "label")["for"], *id);
    assert_eq!(input["aria-describedby"], format!("{id}-description"));

    let order = |needle: &str| body.find(needle).unwrap_or_else(|| panic!("no {needle}"));
    assert!(order("<input") < order("<label"));
}

#[test]
fn a_pin_field_is_one_group_of_cells_that_share_the_frame() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PinField { label: "Code", length: 6usize, name: "otp", value: "12" }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // Six cells, each in its own frame, plus the hidden input the pin posts
    // with.
    assert_eq!(body.matches(r#"data-pin-index="#).count(), 6);
    assert_eq!(body.matches(r#"type="hidden""#).count(), 1);
    assert!(body.contains(r#"name="otp""#));

    // One prepared frame rendered six times: the same class, six times over,
    // and no id on any of them.
    let frame_class = body
        .split(r#"<div class=""#)
        .nth(2)
        .and_then(|rest| rest.split('"').next())
        .expect("a cell frame");
    assert_eq!(body.matches(frame_class).count(), 6);

    // The value fills the cells left to right and leaves the rest empty.
    assert_eq!(body.matches(r#"value="1""#).count(), 1);
    assert_eq!(body.matches(r#"value="2""#).count(), 1);

    // The group is named by the label, not by `for` - a `div` is not labelable.
    assert!(body.contains(r#"role="group""#));
    assert!(body.contains(r#"dir="ltr""#));
    let label_id = format!(
        "{}-label",
        attributes_of(&body, "input")["id"]
            .strip_suffix("-1")
            .expect("the first cell's id ends in -1")
    );
    assert!(body.contains(&format!(r#"aria-labelledby="{label_id}""#)));

    // Only the first cell offers the code a phone just received.
    assert_eq!(body.matches(r#"autocomplete="one-time-code""#).count(), 1);
    // `tel`, not `number`: a numeric keypad with no spinner.
    assert_eq!(body.matches(r#"type="tel""#).count(), 6);
}

/// Todos 507 and 591: each cell names its place, and the helper reaches the
/// focused cell as well as the group.
#[test]
fn every_pin_cell_is_named_and_described() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                PinField { label: "Code", length: 4usize, helper: "Sent by SMS" }
            }
        }
    }

    let body = body(&render(app));
    for n in 1..=4 {
        assert!(
            body.contains(&format!(r#"aria-label="Character {n} of 4""#)),
            "{body}"
        );
    }
    let describedby = attributes_of(&body, "input")["aria-describedby"].clone();
    assert!(!describedby.is_empty(), "{body}");
    assert_eq!(
        body.matches(&format!(r#"aria-describedby="{describedby}""#))
            .count(),
        5,
        "the group and four cells: {body}"
    );
}

#[derive(Clone, PartialEq, Options)]
enum Fruit {
    #[option(label = "Sweet apple")]
    Apple,
    Pear,
}

#[test]
fn a_select_posts_its_value_through_a_hidden_input() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Select { value: Fruit::Apple, onchange: move |_| {}, name: "fruit" }
                MultiSelect {
                    value: vec![Fruit::Apple, Fruit::Pear],
                    onchange: move |_| {},
                    name: "fruits",
                }
                Select { value: Fruit::Pear, onchange: move |_| {}, name: "off", disabled: true }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // The wire value is the variant's name, not the customised label - a form
    // must not change what it sends when a label is translated.
    assert!(body.contains(r#"<input type="hidden" name="fruit" value="Apple"/>"#));
    assert!(!body.contains(r#"value="Sweet apple"/>"#));
    // One input per value, the way a native `<select multiple>` posts.
    assert!(body.contains(r#"<input type="hidden" name="fruits" value="Apple"/>"#));
    assert!(body.contains(r#"<input type="hidden" name="fruits" value="Pear"/>"#));
    assert_eq!(body.matches(r#"type="hidden""#).count(), 4);
    // A disabled select posts nothing.
    assert!(body.contains(r#"name="off" value="Pear" disabled=true/>"#));
}

#[test]
fn a_select_without_a_name_emits_no_hidden_input() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Select { value: Fruit::Apple, onchange: move |_| {} }
            }
        }
    }

    assert!(!body(&render(app)).contains(r#"type="hidden""#));
}

/// The frame paints the surface colour through a var, which `sx` cannot infer
/// a focus contrast from, so the frame has to publish its own - or its ring
/// falls back to the primary shade. And the var it publishes must have a
/// referent, or the ring is not recoloured but erased.
#[test]
fn the_frame_reads_the_surface_and_publishes_its_focus_contrast() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { TextField { label: "Name" } }
        }
    }

    let html = render(app);
    let markup = body(&html);
    let input = markup.find("<input").expect("the control rendered");
    let frame = markup[..input].rfind("<div").expect("the frame");
    let class = attributes_of(&markup[frame..], "div")["class"]
        .split_whitespace()
        .next()
        .expect("a framework class")
        .to_string();

    let start = html
        .find(&format!(".{class}{{"))
        .expect("the frame's base rule");
    let rule = &html[start..start + html[start..].find('}').expect("a closed rule")];
    assert!(
        rule.contains("background:var(--lsx-paper-background);"),
        "{rule}"
    );
    assert!(
        rule.contains("--lsx-focus-contrast:var(--lsx-paper-contrast);"),
        "{rule}"
    );
    assert!(
        html.contains("--lsx-paper-contrast:"),
        "no referent declared"
    );
}

/// `readonly` is one prop with one meaning on every field - focusable, posted,
/// not editable - so it has to reach the control the way that control
/// expresses it. HTML's `readonly` does not apply to a checkbox, and an
/// attribute that does not apply must not be written at all, so there it is
/// `aria-readonly` instead (WAI-ARIA 1.2 supports it on the `checkbox` role).
#[test]
fn read_only_reaches_each_control_the_way_that_control_says_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                TextField { label: "Name", readonly: true }
                Textarea { label: "Notes", readonly: true }
                Checkbox { label: "Agree", readonly: true }
                Switch { label: "On", readonly: true }
            }
        }
    }

    // A boolean attribute renders unquoted, which `attributes_of` cannot
    // split - so this one reads the markup.
    let body = body(&render(app));
    let tag = |needle: &str| {
        let at = body
            .find(needle)
            .unwrap_or_else(|| panic!("no {needle}\n{body}"));
        body[at..]
            .split_once('>')
            .expect("an unterminated tag")
            .0
            .to_string()
    };

    assert!(tag("<input").contains("readonly=true"), "{body}");
    assert!(tag("<textarea").contains("readonly=true"), "{body}");

    // The two checkables say it the ARIA way, and neither writes the attribute
    // HTML says does not apply to them.
    assert_eq!(body.matches(r#"aria-readonly="true""#).count(), 2, "{body}");
    assert_eq!(body.matches("readonly=true").count(), 2, "{body}");

    // Still a tab stop and still posted - that is the whole difference from
    // `disabled`.
    assert!(!body.contains("disabled"), "{body}");
}

/// A read-only field is still in the tab order and still posts, so the one
/// thing that must change is the editing.
#[test]
fn a_read_only_select_keeps_its_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Select {
                    label: "Size",
                    name: "size",
                    value: Pick::First,
                    readonly: true,
                    onchange: move |_: Option<Pick>| {},
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains(r#"tabindex="0""#), "{body}");
    assert!(body.contains(r#"aria-readonly="true""#), "{body}");
    assert!(!body.contains(r#"aria-disabled="true""#), "{body}");

    // And still posts: the hidden input is how a control that is not a form
    // element gets into the form data at all, and `readonly` must not touch
    // it the way `disabled` does.
    assert!(body.contains(r#"name="size""#), "{body}");
    assert!(!body.contains("disabled"), "{body}");
}

/// Options come from data, and data repeats itself. Like `Tabs` with a repeated
/// value, both radio groups check the first match only: two `checked` radios
/// in one group leave the browser showing the last, while the component
/// reasons about the first. Review 6 R7 reported both as checking two.
#[test]
fn a_repeated_option_is_checked_once_on_its_first_radio() {
    fn app() -> Element {
        let options = vec!["a".to_string(), "a".to_string(), "b".to_string()];
        rsx! {
            LiberoProvider {
                RadioGroup { label: "Radio", options: options.clone(), value: "a".to_string(), onchange: move |_: String| {} }
                SegmentedControl { options, value: "a".to_string(), onchange: move |_: String| {} }
            }
        }
    }

    let body = body(&render(app));
    // Open tags only, read raw: SSR writes a boolean as `checked=true`, unquoted.
    let checked: Vec<&str> = body
        .match_indices("<input")
        .map(|(start, _)| &body[start..start + body[start..].find('>').unwrap()])
        .filter(|input| input.contains("checked"))
        .collect();
    assert_eq!(checked.len(), 2, "one per group: {body}");
    assert!(
        checked[0].contains(r#"data-radio-index="0""#),
        "{checked:?}"
    );
    assert!(checked[1].contains(r#"segment-0""#), "{checked:?}");
}

static NO_REVEAL: libero::theme::Theme = libero::theme::Theme {
    password_field: libero::theme::PasswordFieldDefaults {
        reveal_button: false,
    },
    ..libero::theme::Theme::DEFAULT
};

/// An unset `reveal_button` takes the theme's, and a call site still wins.
#[test]
fn an_unset_reveal_button_follows_the_theme() {
    fn themed() -> Element {
        rsx! {
            LiberoProvider { themes: &NO_REVEAL, PasswordField { label: "Password" } }
        }
    }
    fn overridden() -> Element {
        rsx! {
            LiberoProvider { themes: &NO_REVEAL,
                PasswordField { label: "Password", reveal_button: true }
            }
        }
    }

    let html = body(&render(themed));
    assert!(!html.contains("<button"), "{html}");
    assert!(body(&render(overridden)).contains("<button"));
}

static XS_FIELDS: libero::theme::Theme = libero::theme::Theme {
    text_field: libero::theme::TextFieldDefaults {
        size: libero::theme::Size::Xs,
        radius: libero::theme::Size::Sm,
    },
    ..libero::theme::Theme::DEFAULT
};

/// The reveal button takes the field's size, the theme's default included, so
/// it does not outgrow an `xs` frame.
#[test]
fn the_reveal_button_follows_the_field_size_from_the_theme() {
    fn themed() -> Element {
        rsx! {
            LiberoProvider { themes: &XS_FIELDS, PasswordField { label: "Password" } }
        }
    }
    fn explicit() -> Element {
        rsx! {
            LiberoProvider { themes: &XS_FIELDS, PasswordField { label: "Password", size: "xs" } }
        }
    }

    // The size is an inline override of the icon's own size.
    let style = |html: String| attributes_of(&body(&html), "button").get("style").cloned();
    let explicit = style(render(explicit));
    assert!(explicit.is_some());
    assert_eq!(style(render(themed)), explicit);
}

/// A disabled `Fieldset` disables the reveal button along with the input.
#[test]
fn a_disabled_fieldset_disables_the_reveal_button() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Fieldset::<String> { label: "Locked", disabled: true,
                    PasswordField { label: "Old password" }
                }
            }
        }
    }

    let body = body(&render(app));
    assert!(
        attributes_of(&body, "input").contains_key("disabled"),
        "{body}"
    );
    assert!(
        attributes_of(&body, "button").contains_key("disabled"),
        "{body}"
    );
}

/// The field shells skip a value change and a caller's new slot `Element`, so
/// the control and the slot must still show the new one (todo 29).
mod value_moves {
    use super::*;

    #[derive(Clone, PartialEq, Options)]
    enum Size {
        Small,
        Large,
    }

    thread_local! {
        static STEP: std::cell::Cell<Option<Signal<i32>>> = const { std::cell::Cell::new(None) };
    }

    #[test]
    fn a_new_value_reaches_every_control_while_the_shell_skips() {
        fn app() -> Element {
            let step = use_signal(|| 1);
            use_hook(|| STEP.set(Some(step)));
            let oninput = use_callback(|_: String| {});
            let onchange = use_callback(|_: Option<i32>| {});
            let onpick = use_callback(|_: Size| {});
            let size = if step() == 1 {
                Size::Small
            } else {
                Size::Large
            };
            rsx! {
                LiberoProvider {
                    TextField {
                        value: format!("text-{}", step()),
                        oninput,
                        trailing: rsx! { "slot-{step}" },
                    }
                    NumberField { value: step() * 10, onchange }
                    NativeSelect { value: size, onchange: onpick }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let html = body(&dioxus_ssr::render(&dom));
        assert!(html.contains(r#"value="text-1""#), "{html}");
        assert!(html.contains(r#"aria-valuenow="10""#), "{html}");
        assert!(html.contains(r#"value="Small" selected"#), "{html}");

        let mut step = STEP.get().expect("the app stored its signal");
        dom.in_runtime(|| step.set(2));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = body(&dioxus_ssr::render(&dom));
        assert!(html.contains(r#"value="text-2""#), "stale text:\n{html}");
        assert!(html.contains("slot-2"), "stale slot:\n{html}");
        assert!(
            html.contains(r#"aria-valuenow="20""#),
            "stale number:\n{html}"
        );
        assert!(html.contains(r#"value="20""#), "stale number text:\n{html}");
        assert!(
            html.contains(r#"value="Large" selected"#),
            "stale selection:\n{html}"
        );
        assert!(!html.contains(r#"value="Small" selected"#), "{html}");
    }
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::{attributes_of, body};
    use crate::dispatch::*;
    use dioxus::core::ElementId;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{
            Button, ColorCode, ColorField, Form, NumberField, PasswordField, PinField, Rule,
            SliderChangeEvent, TextField, not_empty, use_form,
        },
    };

    use std::rc::Rc;

    /// The reveal button flips the input's `type` and its `aria-pressed`, both ways,
    /// under one static name.
    #[test]
    fn clicking_the_reveal_button_toggles_the_password() {
        fn app() -> Element {
            rsx! { LiberoProvider { PasswordField { label: "Password" } } }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let reveal = find.click.expect("registered no click listener");
        let state = |dom: &VirtualDom| {
            let html = dioxus_ssr::render(dom);
            let button = attributes_of(&html, "button");
            (
                attributes_of(&html, "input")["type"].clone(),
                button["aria-label"].clone(),
                button["aria-pressed"].clone(),
            )
        };
        let expect =
            |kind: &str, pressed: &str| (kind.into(), "Show password".into(), pressed.into());

        assert_eq!(state(&dom), expect("password", "false"));
        for (kind, pressed) in [("text", "true"), ("password", "false")] {
            dom.runtime()
                .handle_event("click", Event::new(click_event(), true), reveal);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            assert_eq!(state(&dom), expect(kind, pressed));
        }
    }

    /// Every submit and every reset of the form hides a revealed secret again.
    #[test]
    fn a_submit_or_a_reset_hides_the_revealed_password() {
        fn app() -> Element {
            rsx! { LiberoProvider { Form::<()> { PasswordField { label: "Password" } } } }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let reveal = find.click.expect("registered no click listener");
        let submit = find.submit.expect("registered no submit listener");
        let reset = find.reset.expect("registered no reset listener");
        let kind =
            |dom: &VirtualDom| attributes_of(&dioxus_ssr::render(dom), "input")["type"].clone();

        // Twice through: the second submit hides it as the first did.
        for (name, target) in [("submit", submit), ("submit", submit), ("reset", reset)] {
            dom.runtime()
                .handle_event("click", Event::new(click_event(), true), reveal);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            assert_eq!(kind(&dom), "text", "revealed before the {name}");
            dom.runtime()
                .handle_event(name, Event::new(input_event(""), true), target);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            assert_eq!(kind(&dom), "password", "hidden by the {name}");
        }
    }

    /// The same defect on the field the todo names first: a `TextField` with rules,
    /// no `value` and no binding. The browser keeps its text, so only the rules
    /// need what was typed - the `value` attribute stays off it.
    #[test]
    fn typing_into_an_uncontrolled_text_field_satisfies_its_own_rules() {
        fn app() -> Element {
            let handle = use_form();
            let valid = use_signal(|| true);

            rsx! {
                LiberoProvider {
                    Button {
                        id: "submit",
                        onclick: move |_| {
                            let mut valid = valid;
                            valid.set(handle.validate());
                        },
                        "Submit"
                    }
                    Form::<()> { form: handle,
                        TextField {
                            label: "Email",
                            validate: (|text: &String| not_empty(text)).error("Enter your email"),
                        }
                    }
                    "valid: {valid}"
                }
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let submit = find.element("click", "id", "submit");
        let field = find.input.expect("registered no input listener");

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), submit);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert!(
            dioxus_ssr::render(&dom).contains("valid: false"),
            "an empty required field must block the submit"
        );

        dom.runtime().handle_event(
            "input",
            Event::new(input_event("tom@libero.dev"), true),
            field,
        );
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        let html = dioxus_ssr::render(&dom);
        assert!(
            !attributes_of(&body(&html), "input").contains_key("value"),
            "the field is still uncontrolled - the browser owns the text"
        );

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), submit);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert!(
            dioxus_ssr::render(&dom).contains("valid: true"),
            "the rules must judge what was typed, not the default"
        );
    }

    thread_local! {
        static PIN_EDITS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    /// The native `readonly` stops typing into a cell, but Backspace and Delete
    /// are answered by the field itself and used to clear one anyway - so a pin
    /// that "can be read and copied, but not changed" could be erased (todo 209).
    #[test]
    fn a_read_only_pin_field_answers_no_key_that_edits() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    PinField {
                        label: "Code",
                        length: 4,
                        value: "1234",
                        readonly: true,
                        oninput: move |next: String| {
                            PIN_EDITS.with_borrow_mut(|edits| edits.push(next));
                        },
                    }
                }
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        PIN_EDITS.with_borrow_mut(Vec::clear);
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let second = find.element("keydown", "aria-label", "Character 2 of 4");

        for key in [Key::Backspace, Key::Delete] {
            dom.runtime()
                .handle_event("keydown", Event::new(key_event(key), true), second);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }

        assert_eq!(PIN_EDITS.with_borrow(Clone::clone), Vec::<String>::new());
    }

    /// A cell whose character did not change skips its redraw, so its handlers
    /// must still edit the pin as it is now, not as it was when the cell drew.
    #[test]
    fn a_skipped_pin_cell_edits_the_current_pin() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    PinField {
                        length: 4,
                        oninput: move |next: String| {
                            PIN_EDITS.with_borrow_mut(|edits| edits.push(next));
                        },
                    }
                }
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        PIN_EDITS.with_borrow_mut(Vec::clear);
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let last = find.input.expect("registered no input listener");

        // The last cell stays empty: a pin has no holes, so each digit lands first.
        for digit in ["1", "2"] {
            dom.runtime()
                .handle_event("input", Event::new(input_event(digit), true), last);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }

        assert_eq!(PIN_EDITS.with_borrow(Clone::clone), ["1", "12"]);
    }

    /// `Form` took its `value` store once, so a parent that handed over a
    /// different one - another record in an edit view - kept fields reading and
    /// writing the old store while the form's own rules read the new one (todo
    /// 190). `Store`'s equality is its identity, so the swap is recognisable.
    #[test]
    fn swapping_the_forms_value_swaps_what_its_fields_read() {
        fn app() -> Element {
            let first = use_store(|| crate::validation::Order {
                name: "Tom".into(),
                ..Default::default()
            });
            let second = use_store(|| crate::validation::Order {
                name: "Ada".into(),
                ..Default::default()
            });
            let mut second_record = use_signal(|| false);

            rsx! {
                LiberoProvider {
                    Button { id: "next", onclick: move |_| second_record.toggle(), "Next record" }
                    Form {
                        value: if second_record() { second } else { first },
                        TextField { name: crate::validation::Order::FIELDS.name() }
                    }
                }
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let next = find.element("click", "id", "next");

        let value = |dom: &VirtualDom| {
            attributes_of(&body(&dioxus_ssr::render(dom)), "input")
                .get("value")
                .cloned()
        };

        assert_eq!(value(&dom).as_deref(), Some("Tom"));

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), next);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert_eq!(value(&dom).as_deref(), Some("Ada"));
    }

    fn readonly_number() -> Element {
        rsx! {
            LiberoProvider {
                NumberField {
                    label: "Quantity",
                    value: 3,
                    steppers: true,
                    readonly: READ_ONLY.get(),
                    onchange: move |next: Option<i32>| heard(next.unwrap()),
                }
            }
        }
    }

    /// Todo 306 (review 3 C2): the native `readonly` stops typing, but the arrows
    /// and the steppers are the field's own, so they are refused in Rust.
    #[test]
    fn a_read_only_number_field_does_not_step() {
        let arrow = || key_event(Key::ArrowUp);
        let (stepped, _) = send(readonly_number, false, "keydown", last_keydown, arrow);
        assert_eq!(stepped, ["4"], "the arrow is the control");
        let (heard, html) = send(readonly_number, true, "keydown", last_keydown, arrow);
        assert_eq!(heard, Vec::<String>::new());

        let input = attributes_of(&body(&html), "input");
        assert_eq!(
            input.get("readonly").map(String::as_str),
            Some("true"),
            "{input:?}"
        );
        assert!(!input.contains_key("disabled"), "{input:?}");

        let (stepped, _) = send(readonly_number, false, "click", last_click, click_event);
        assert_eq!(stepped, ["4"], "the stepper is the control");
        let (heard, _) = send(readonly_number, true, "click", last_click, click_event);
        assert_eq!(heard, Vec::<String>::new());
    }

    /// Todo 29: the steppers skip the render a press causes, so their handler must
    /// still step from the value the field holds now, not the one they drew with.
    #[test]
    fn a_stepper_steps_from_the_value_after_the_last_press() {
        fn app() -> Element {
            let mut value = use_signal(|| 3);
            rsx! {
                LiberoProvider {
                    NumberField {
                        value: value(),
                        steppers: true,
                        onchange: move |next: Option<i32>| {
                            heard(next.unwrap());
                            value.set(next.unwrap());
                        },
                    }
                }
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        HEARD.with_borrow_mut(Vec::clear);
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let mut send = |name: &str, target: ElementId, data: Rc<dyn std::any::Any>| {
            dom.runtime()
                .handle_event(name, Event::new(data, true), target);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        };

        let (minus, plus) = (find.first_click.unwrap(), last_click(&find));
        send("click", plus, click_event());
        send("click", plus, click_event());
        send("keydown", last_keydown(&find), key_event(Key::ArrowUp));
        send("click", minus, click_event());

        assert_eq!(HEARD.with_borrow(Clone::clone), ["4", "5", "6", "5"]);
        assert_eq!(
            attributes_of(&body(&dioxus_ssr::render(&dom)), "input")["value"],
            "5"
        );
    }

    /// A float field reads the formats' separator and `.` alike; under English
    /// `1,5` is not a number.
    #[test]
    fn a_float_field_parses_the_formats_decimal_separator() {
        use libero::localization::Formats;
        thread_local! {
            static FORMATS: std::cell::Cell<&'static Formats> = const { std::cell::Cell::new(&Formats::AMERICAN) };
        }
        fn app() -> Element {
            rsx! {
                LiberoProvider { formats: FORMATS.get(),
                    NumberField {
                        value: None::<f64>,
                        onchange: move |next: Option<f64>| heard(next),
                    }
                }
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        for (formats, typed, expected) in [
            (
                &Formats::GERMAN,
                ["1,5", "2.5"],
                vec!["Some(1.5)", "Some(2.5)"],
            ),
            (&Formats::AMERICAN, ["1,5", "2.5"], vec!["Some(2.5)"]),
        ] {
            FORMATS.set(formats);
            HEARD.with_borrow_mut(Vec::clear);
            let mut dom = VirtualDom::new(app);
            let mut find = FindClickListener::default();
            dom.rebuild(&mut find);
            let field = find.input.expect("registered no input listener");
            for text in typed {
                dom.runtime()
                    .handle_event("input", Event::new(input_event(text), true), field);
                dom.render_immediate(&mut dioxus::core::NoOpMutations);
            }
            assert_eq!(HEARD.with_borrow(Clone::clone), expected);
        }
    }

    fn readonly_color() -> Element {
        rsx! {
            LiberoProvider {
                ColorField {
                    label: "Accent",
                    value: "#ff0000".parse::<ColorCode>().unwrap(),
                    readonly: READ_ONLY.get(),
                    oninput: move |event: SliderChangeEvent<ColorCode>| {
                        heard(match event {
                            SliderChangeEvent::Start(_) => "Start",
                            SliderChangeEvent::Change(_) => "Change",
                            SliderChangeEvent::End(_) => "End",
                        })
                    },
                }
            }
        }
    }

    /// Todo 306: the text takes the native `readonly`, and the dropdown - the
    /// other editor - refuses to open, as every dropdown field's does.
    #[test]
    fn a_read_only_color_field_opens_no_dropdown() {
        let expanded = |html: &str| {
            attributes_of(&body(html), "input")
                .get("aria-expanded")
                .cloned()
        };
        let (_, html) = send(readonly_color, false, "click", last_click, click_event);
        assert_eq!(
            expanded(&html).as_deref(),
            Some("true"),
            "the click is the control"
        );
        let (_, html) = send(readonly_color, true, "click", last_click, click_event);
        assert_eq!(expanded(&html).as_deref(), Some("false"));
        let input = attributes_of(&body(&html), "input");
        assert_eq!(
            input.get("readonly").map(String::as_str),
            Some("true"),
            "{html}"
        );
    }

    /// Todo 291: typed text that parses is settled the moment it lands, so it
    /// emits `Change` then `End`, like a key press or a swatch in the dropdown - a
    /// caller committing on `End` used to miss every typed color.
    #[test]
    fn typing_a_color_emits_change_then_end() {
        let (heard, _) = send(readonly_color, false, "input", input_listener, || {
            input_event("#00ff00")
        });
        assert_eq!(heard, ["\"Change\"", "\"End\""]);
    }

    thread_local! {
        static PIN_COMPLETIONS: std::cell::RefCell<u32> = const { std::cell::RefCell::new(0) };
    }

    /// A parent resetting the controlled `value` between attempts - "wrong code,
    /// try again" - re-arms `oncomplete`, so a pasted retry completes (todo 413).
    #[test]
    fn oncomplete_fires_again_after_a_parent_resets_the_controlled_value() {
        fn app() -> Element {
            let mut pin = use_signal(String::new);

            rsx! {
                LiberoProvider {
                    Button { id: "reset", onclick: move |_| pin.set(String::new()), "Reset" }
                    PinField {
                        label: "Code",
                        length: 4,
                        value: pin(),
                        oninput: move |next: String| pin.set(next),
                        oncomplete: move |_: String| {
                            PIN_COMPLETIONS.with_borrow_mut(|count| *count += 1);
                        },
                    }
                }
            }
        }

        dioxus::html::set_event_converter(Box::new(TestConverter));
        PIN_COMPLETIONS.with_borrow_mut(|count| *count = 0);
        let mut dom = VirtualDom::new(app);
        let mut find = FindClickListener::default();
        dom.rebuild(&mut find);
        let cells = find.keydown.clone();
        assert_eq!(cells.len(), 4, "expected one cell per pin position");
        let reset = find.element("click", "id", "reset");

        let type_pin = |dom: &mut VirtualDom, digits: &str| {
            for (cell, digit) in cells.iter().zip(digits.chars()) {
                dom.runtime().handle_event(
                    "input",
                    Event::new(input_event(&digit.to_string()), true),
                    *cell,
                );
                dom.render_immediate(&mut dioxus::core::NoOpMutations);
            }
        };

        type_pin(&mut dom, "1234");
        assert_eq!(
            PIN_COMPLETIONS.with_borrow(|count| *count),
            1,
            "the first full pin must complete"
        );

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), reset);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        // An OTP autofill (or a paste) lands the whole code in one `input` event
        // on the first cell, rather than one keystroke per cell.
        dom.runtime()
            .handle_event("input", Event::new(input_event("5678"), true), cells[0]);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        assert_eq!(
            PIN_COMPLETIONS.with_borrow(|count| *count),
            2,
            "a pin pasted after the parent reset it must complete again"
        );
    }
}
