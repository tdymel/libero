//! The chrome `use_field` puts around a control: the five slots, and the a11y
//! wiring that ties them to it. `TextField` carries most of the cases;
//! `NativeSelect` covers what changed when it was ported off its wrapping `<label>`.

use crate::common::{attributes_of, body, render};

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
    assert!(!body.contains("data-slot"), "{body}");
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
    assert!(!body.contains("data-slot"), "{body}");
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
    let group_attributes = attributes_of(&body[body.find("<div").unwrap() + 4..], "div");
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
    let values: Vec<String> = body
        .match_indices("<input")
        .map(|(at, _)| attributes_of(&body[at..], "input")["value"].clone())
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
    assert_eq!(unset.matches("<input").count(), 3);
    assert_eq!(unset.matches("disabled").count(), 0, "{unset}");

    // Only the flagged option is refused, and the selected one keeps the one
    // tab stop the group has.
    let flagged = body(&render(flagged));
    assert_eq!(flagged.matches("<input").count(), 3);
    let team = &flagged[flagged
        .find(r#"data-radio-index="2""#)
        .expect("the third radio")..];
    assert!(team[..team.find('>').expect("an unterminated tag")].contains("disabled"));
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
    let group = attributes_of(&body[body.find("<div").unwrap() + 4..], "div");
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

    let radios: Vec<&str> = body
        .match_indices("<input")
        .map(|(at, _)| &body[at..at + body[at..].find('>').unwrap()])
        .collect();
    assert_eq!(radios.len(), 3, "{body}");
    assert!(
        radios.iter().all(|radio| radio.contains("disabled")),
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
    let group = attributes_of(&body[body.find("<div").unwrap() + 4..], "div");
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
            LiberoProvider { theme: &NO_REVEAL, PasswordField { label: "Password" } }
        }
    }
    fn overridden() -> Element {
        rsx! {
            LiberoProvider { theme: &NO_REVEAL,
                PasswordField { label: "Password", reveal_button: true }
            }
        }
    }

    let html = body(&render(themed));
    assert!(!html.contains("<button"), "{html}");
    assert!(body(&render(overridden)).contains("<button"));
}
