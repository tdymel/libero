//! `Form` with a bound, validated `TextField`, and outside buttons that write its value,
//! re-render the page, submit and reset.

use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{
    Autocomplete, Button, Cascader, CascaderOption, Checkbox, ColorCode, ColorField, DateField,
    Fields, Fieldset, FileField, Files, Flex, Form, MultiSelect, NativeSelect, NumberField,
    Options, PasswordField, PhoneField, PinField, RadioGroup, Rating, Rule, SegmentedControl,
    Select, Slider, SliderChangeEvent, Switch, TagsField, TextField, Textarea, min_length,
    not_empty, use_form,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/form", || rsx! { FormPage {} }),
    ("/form/summary", || rsx! { SummaryPage {} }),
    ("/form/targets", || rsx! { TargetsPage {} }),
];

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
}

#[component]
fn TargetsPage() -> Element {
    let mut plan = use_signal(|| None::<Plan>);
    let mut tier = use_signal(|| None::<Plan>);
    let mut pin = use_signal(String::new);
    let mut stars = use_signal(|| 0.0f64);
    let mut tags = use_signal(Vec::<String>::new);
    let mut notify = use_signal(|| false);
    let mut native = use_signal(|| None::<Plan>);
    let mut billing = use_signal(|| Plan::Free);
    let mut files = use_signal(Files::default);
    let mut phone = use_signal(String::new);
    let mut color = use_signal(|| ColorCode::rgba(0, 0, 0, 1.0));
    let mut due = use_signal(|| None::<NaiveDate>);
    let mut category = use_signal(|| None::<String>);
    let mut volume = use_signal(|| 0.0f64);
    let mut city = use_signal(String::new);
    let mut toppings = use_signal(Vec::<String>::new);
    let mut quantity = use_signal(|| None::<u32>);
    let mut notes = use_signal(String::new);
    let mut news = use_signal(|| false);
    let mut nickname = use_signal(String::new);
    rsx! {
        Form::<()> { summary_title: "Fix these:",
            RadioGroup {
                label: "Plan",
                value: plan(),
                onchange: move |next| plan.set(Some(next)),
                validate: not_empty.error("Pick a plan."),
            }
            Select {
                label: "Tier",
                value: tier(),
                onchange: move |next| tier.set(next),
                validate: not_empty.error("Pick a tier."),
            }
            PinField {
                label: "Code",
                value: pin(),
                oninput: move |next: String| pin.set(next),
                length: 4usize,
                validate: min_length(4).error("Enter the code."),
            }
            Rating {
                label: "Stars",
                value: stars(),
                onchange: move |next| stars.set(next),
                validate: (|v: &f64| *v > 0.0).error("Rate it."),
            }
            TagsField {
                label: "Tags",
                value: tags(),
                onchange: move |next| tags.set(next),
                validate: not_empty.error("Add a tag."),
            }
            Switch {
                label: "Notify me",
                checked: notify(),
                onchange: move |next| notify.set(next),
                validate: not_empty.error("Switch it on."),
            }
            NativeSelect {
                label: "Plan again",
                value: native(),
                onchange: move |next| native.set(Some(next)),
                validate: not_empty.error("Pick one."),
            }
            SegmentedControl {
                label: "Billing",
                value: Some(billing()),
                onchange: move |next| billing.set(next),
                validate: (|v: &Plan| *v == Plan::Pro).error("Pick Pro."),
            }
            FileField {
                label: "Receipt",
                value: files(),
                onchange: move |next| files.set(next),
                validate: (|_: &Files| false).error("Attach a file."),
            }
            PhoneField {
                label: "Phone",
                value: Some(phone()),
                oninput: move |next: String| phone.set(next),
                validate: min_length(5).error("Enter a number."),
            }
            ColorField {
                label: "Colour",
                value: color(),
                oninput: move |event: SliderChangeEvent<ColorCode>| color.set(event.value()),
                validate: (|_: &ColorCode| false).error("Pick a colour."),
            }
            DateField {
                label: "Due",
                value: due(),
                onchange: move |next| due.set(next),
                validate: not_empty.error("Pick a date."),
            }
            Cascader {
                label: "Category",
                data: vec![CascaderOption::new("fruit", "Fruit").children(vec![
                    CascaderOption::new("apple", "Apple"),
                ])],
                value: category(),
                onchange: move |next| category.set(next),
                validate: not_empty.error("Pick a category."),
            }
            Slider {
                label: "Volume",
                value: volume(),
                oninput: move |event: SliderChangeEvent| volume.set(event.value()),
                validate: (|v: &f64| *v > 0.0).error("Turn it up."),
            }
            Autocomplete {
                label: "City",
                value: city(),
                oninput: move |text| city.set(text),
                options: vec!["Berlin".to_string(), "Paris".to_string()],
                validate: not_empty.error("Enter a city."),
            }
            MultiSelect {
                label: "Toppings",
                options: vec!["Cheese".to_string(), "Olives".to_string()],
                value: toppings(),
                onchange: move |next| toppings.set(next),
                validate: not_empty.error("Pick a topping."),
            }
            NumberField {
                label: "Quantity",
                value: quantity(),
                onchange: move |next| quantity.set(next),
                validate: not_empty.error("Enter a quantity."),
            }
            Textarea {
                label: "Notes",
                value: notes(),
                oninput: move |text| notes.set(text),
                validate: not_empty.error("Add a note."),
            }
            // Named by `aria_label` only: the summary line still says which (1526).
            Checkbox {
                aria_label: "Newsletter",
                checked: news(),
                onchange: move |next| news.set(next),
                validate: not_empty.error("Tick the box."),
            }
            TextField {
                "aria-label": "Nickname",
                value: nickname(),
                oninput: move |text| nickname.set(text),
                validate: not_empty.error("Enter a nickname."),
            }
            // A group in error is one input in error: its line focuses a control inside (1273).
            Fieldset::<()> {
                label: "Address",
                status: "Address not found.",
                TextField { label: "Street", value: "" }
            }
            Button { r#type: "submit", "Send" }
        }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct NewPassword {
    value: String,
    repeat: String,
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Account {
    email: String,
    #[fields(nested)]
    password: NewPassword,
    terms: bool,
}

#[component]
fn SummaryPage() -> Element {
    let value = use_store(Account::default);
    rsx! {
        Form {
            value,
            summary_title: "Please fix these first:",
            validate: (|s: &Account| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Account::FIELDS.password().value()]),
            TextField {
                label: "Email",
                r#type: "email",
                name: Account::FIELDS.email(),
                validate: not_empty.error("Enter your email."),
            }
            Fieldset {
                label: "Password",
                path: Account::FIELDS.password(),
                validate: (|p: &NewPassword| p.value == p.repeat)
                    .error("The passwords differ.")
                    .on([NewPassword::FIELDS.repeat()]),
                PasswordField {
                    label: "Password",
                    name: NewPassword::FIELDS.value(),
                    validate: min_length(8).error("Use at least 8 characters."),
                }
                PasswordField { label: "Repeat password", name: NewPassword::FIELDS.repeat() }
            }
            Checkbox {
                label: "I accept the terms",
                name: Account::FIELDS.terms(),
                validate: not_empty.error("Accept the terms to continue."),
            }
            Button { r#type: "submit", "Create account" }
        }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    email: String,
}

#[component]
pub fn FormPage() -> Element {
    let mut value = use_store(Signup::default);
    let mut renders = use_signal(|| 0u32);
    let mut submits = use_signal(|| 0u32);
    let form = use_form();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { "data-renders": "{renders}", "Renders {renders}" }
            span { "data-submits": "{submits}", "Submits {submits}" }
            Form {
                value,
                form,
                onsubmit: move |_| submits += 1,
                TextField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Email needed") }
                button { r#type: "submit", "Send" }
            }
            Button { onclick: move |_| value.write().email.clear(), "Clear" }
            Button { onclick: move |_| renders += 1, "Rerender" }
            Button { onclick: move |_| { let _ = form.submit(); }, "Submit" }
            Button { onclick: move |_| form.reset(), "Reset" }
        }
    }
}
