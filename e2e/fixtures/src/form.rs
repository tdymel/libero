//! `Form` with a bound, validated `TextField`, and outside buttons that write its value,
//! re-render the page, submit and reset.

use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{
    Autocomplete, Button, ByAddress, Cascader, CascaderOption, Checkbox, Chip, ColorCode, ColorField,
    DateField, DatePicker, DateRangePicker, DateTimeField, FieldError, FieldStatus, Fields, Fieldset, FileField, Files, Flex, Form,
    FormErrors, MonthPicker, MultiSelect, NativeSelect, NumberField, Options, PasswordField, PhoneField, PinField, Radio,
    RadioGroup, RangeSlider, Rating, RequiredValue, Rule, SegmentedControl, Select, Slider, SliderChangeEvent,
    Submit, SummaryMode, Switch, TagsField, Text, TextField, Textarea, TimeField, TimePicker, YearPicker, checked_async, min_length, not_empty, use_form,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/form", || rsx! { FormPage {} }),
    ("/form/summary", || rsx! { SummaryPage {} }),
    ("/form/targets", || rsx! { TargetsPage {} }),
    ("/form/required", || rsx! { RequiredPage {} }),
    ("/form/fieldset-disabled", || rsx! { DisabledFieldsetPage {} }),
    ("/form/fieldset-submit", || rsx! { DisabledFieldsetSubmitPage {} }),
    ("/form/server", || rsx! { ServerStatusPage { at_once: false } }),
    ("/form/server-sync", || rsx! { ServerStatusPage { at_once: true } }),
    ("/form/domain", || rsx! { DomainPage {} }),
    ("/form/command", || rsx! { CommandPage {} }),
    ("/form/login", || rsx! { LoginPage {} }),
    ("/form/prefill", || rsx! { PrefillPage {} }),
    ("/form/prefill-more", || rsx! { PrefillMorePage {} }),
    ("/form/async", || rsx! { AsyncSignInPage {} }),
    ("/form/secret", || rsx! { SecretPage {} }),
    ("/form/async-rule", || rsx! { AsyncRulePage { loader: false } }),
    ("/form/async-rule-loader", || rsx! { AsyncRulePage { loader: true } }),
    ("/form/order", || rsx! { OrderPage { hidden: false } }),
    ("/form/order-hidden", || rsx! { OrderPage { hidden: true } }),
];

#[derive(Clone, PartialEq, Default, Fields)]
struct Mailbox {
    email: Option<String>,
}

enum EmailError {
    Invalid,
    Taken,
}

fn signup_email(text: String) -> Result<String, EmailError> {
    match text.contains('@') {
        true => Ok(text),
        false => Err(EmailError::Invalid),
    }
}

fn email_words(error: &EmailError, _: &libero::localization::Localization) -> String {
    match error {
        EmailError::Invalid => "Not an email address.".into(),
        EmailError::Taken => "This address is taken.".into(),
    }
}

/// The latest address the rule asked about, which the page's buttons answer.
#[derive(Clone, Default)]
struct Lookup {
    taken: std::rc::Rc<std::cell::RefCell<Option<bool>>>,
    waker: std::rc::Rc<std::cell::RefCell<Option<std::task::Waker>>>,
}

impl Lookup {
    fn answer(&self, taken: bool) {
        *self.taken.borrow_mut() = Some(taken);
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }

    async fn ask(self) -> Result<(), EmailError> {
        self.taken.borrow_mut().take();
        std::future::poll_fn(move |context| match self.taken.borrow_mut().take() {
            Some(true) => std::task::Poll::Ready(Err(EmailError::Taken)),
            Some(false) => std::task::Poll::Ready(Ok(())),
            None => {
                *self.waker.borrow_mut() = Some(context.waker().clone());
                std::task::Poll::Pending
            }
        })
        .await
    }
}

/// Todo 4087: an async rule the buttons answer; a submit waits for it. `loader`: todo 4116.
#[component]
fn AsyncRulePage(loader: bool) -> Element {
    let signup = use_store(Mailbox::default);
    let form = use_form();
    let lookup = use_hook(Lookup::default);
    let asked = use_signal(|| 0u32);
    let mut sent = use_signal(|| 0u32);
    let (free, taken) = (lookup.clone(), lookup.clone());
    let rule = checked_async(move |_: String| {
        let mut asked = asked;
        asked += 1;
        lookup.clone().ask()
    })
    .debounce(100);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { "data-asked": "{asked}", "data-sent": "{sent}", "data-submitting": "{form.is_submitting()}" }
            Form { form, value: signup, pending_loader: loader, onsubmit: move |_| sent += 1,
                TextField { id: "email", label: "Email", required: true,
                    name: Mailbox::FIELDS.email().map(String::clone, signup_email).message(email_words).rules([rule]) }
                Button { r#type: "submit", loading: form.is_submitting(), "Sign up" }
            }
            Button { onclick: move |_| free.answer(false), "Free" }
            Button { onclick: move |_| taken.answer(true), "Taken" }
        }
    }
}

/// A secret with neither `Clone` nor `PartialEq`.
struct Secret(String);

#[derive(Clone, PartialEq, Default, Fields)]
struct Vault {
    password: Option<ByAddress<Secret>>,
}

fn secret(text: String) -> Result<ByAddress<Secret>, std::convert::Infallible> {
    Ok(ByAddress::new(Secret(text)))
}

fn vault_secret(vault: &Vault) -> Result<ByAddress<Secret>, FormErrors> {
    vault.password.clone().required(Vault::FIELDS.password())
}

/// Todo 4106: a password mapped to a secret held by address, as the docs' sign-in does.
#[component]
fn SecretPage() -> Element {
    let vault = use_store(Vault::default);
    let mut sent = use_signal(String::new);
    let mut shared = use_signal(String::new);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { id: "sent", "data-sent": "{sent}", "data-shared": "{shared}" }
            Form { value: vault,
                onsubmit: Submit::parse(vault_secret, move |secret: ByAddress<Secret>| {
                    // The form's store still holds the secret, so it is not handed over (4114).
                    let secret = match ByAddress::try_unwrap(secret) {
                        Ok(Secret(text)) => {
                            shared.set("unwrapped".into());
                            sent.set(text);
                            return;
                        }
                        Err(secret) => secret,
                    };
                    shared.set("shared".into());
                    sent.set(secret.0.clone());
                }),
                PasswordField { id: "pass", label: "Password", required: true,
                    name: Vault::FIELDS.password().map(|secret: &ByAddress<Secret>| secret.0.clone(), secret) }
                Button { id: "send", r#type: "submit", "Unlock" }
            }
        }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct SignIn {
    email: String,
    code: String,
}

/// A request the page's buttons answer.
#[derive(Clone, Default)]
struct Pending {
    answer: std::rc::Rc<std::cell::RefCell<Option<Result<(), FormErrors>>>>,
    waker: std::rc::Rc<std::cell::RefCell<Option<std::task::Waker>>>,
}

impl Pending {
    fn answer(&self, result: Result<(), FormErrors>) {
        *self.answer.borrow_mut() = Some(result);
        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }

    async fn wait(self) -> Result<(), FormErrors> {
        std::future::poll_fn(move |context| match self.answer.borrow_mut().take() {
            Some(result) => std::task::Poll::Ready(result),
            None => {
                *self.waker.borrow_mut() = Some(context.waker().clone());
                std::task::Poll::Pending
            }
        })
        .await
    }
}

/// Todos 4097, 4099, 4100: an async submit the buttons answer, behind a hidden summary.
#[component]
fn AsyncSignInPage() -> Element {
    let sign_in = use_store(SignIn::default);
    let form = use_form();
    let pending = use_hook(Pending::default);
    let mut calls = use_signal(|| 0u32);
    let request = pending.clone();
    let (accept, refuse) = (pending.clone(), pending);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { "data-calls": "{calls}", "data-submitting": "{form.is_submitting()}" }
            Form { form, value: sign_in, summary: SummaryMode::Hidden,
                onsubmit: Submit::task(move |_| {
                    calls += 1;
                    request.clone().wait()
                }),
                TextField { id: "email", label: "Email", required: true, name: SignIn::FIELDS.email() }
                TextField { id: "code", label: "Code", required: true, name: SignIn::FIELDS.code() }
                Button { r#type: "submit", loading: form.is_submitting(), "Sign in" }
            }
            Button { onclick: move |_| accept.answer(Ok(())), "Accept" }
            Button { onclick: move |_| refuse.answer(Err(FormErrors::new().form("Wrong code."))), "Refuse" }
        }
    }
}

/// What `CommandPage` sends: a German IBAN only, refused after the field's own parse.
fn payout_command(payout: &Payout) -> Result<String, FormErrors> {
    let iban = payout.iban.required(Payout::FIELDS.iban())?;
    match iban.0.starts_with("GB") {
        true => Err(FormErrors::new().at(Payout::FIELDS.iban(), "Use a German account.")),
        false => Ok(iban.0),
    }
}

/// Todo 4084: `Submit::parse` refuses a parsed IBAN onto its field; an edit clears it.
#[component]
fn CommandPage() -> Element {
    let payout = use_store(Payout::default);
    let mut sent = use_signal(String::new);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { "data-sent": "{sent}" }
            Form {
                value: payout,
                onsubmit: Submit::parse(payout_command, move |iban: String| sent.set(iban)),
                TextField {
                    id: "iban",
                    label: "IBAN",
                    required: true,
                    name: Payout::FIELDS.iban().map(Iban::text, Iban::new),
                }
                Button { r#type: "submit", "Send" }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Iban(String);

struct NotAnIban;

impl FieldError for NotAnIban {
    fn message(&self, _: &libero::localization::Localization) -> String {
        "Not an IBAN.".into()
    }
}

impl Iban {
    fn new(text: String) -> Result<Iban, NotAnIban> {
        let text: String = text.split_whitespace().collect::<String>().to_uppercase();
        match text.len() == 22 && text.chars().take(2).all(|c| c.is_ascii_alphabetic()) {
            true => Ok(Iban(text)),
            false => Err(NotAnIban),
        }
    }

    fn text(&self) -> String {
        let chars: Vec<char> = self.0.chars().collect();
        chars.chunks(4).map(|chunk| chunk.iter().collect::<String>()).collect::<Vec<_>>().join(" ")
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Share(f64);

fn share(percent: f64) -> Result<Share, NotAnIban> {
    match (0.0..=100.0).contains(&percent) {
        true => Ok(Share(percent)),
        false => Err(NotAnIban),
    }
}

/// An E.164 number with at least ten digits, dial code included.
#[derive(Clone, PartialEq, Debug)]
struct Phone(String);

impl Phone {
    fn new(e164: String) -> Result<Phone, NotAnIban> {
        match e164.chars().filter(char::is_ascii_digit).count() >= 10 {
            true => Ok(Phone(e164)),
            false => Err(NotAnIban),
        }
    }

    fn text(&self) -> String {
        self.0.clone()
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Payout {
    iban: Option<Iban>,
    share: Option<Share>,
    phone: Option<Phone>,
}

/// Concept domain-values: `#iban` maps text to an `Iban`, the share and phone their own buffers'
/// values. Typed text stays as typed until a blur; invalid text stays, resets the value and blocks the submit.
#[component]
fn DomainPage() -> Element {
    let payout = use_store(Payout::default);
    let mut submits = use_signal(|| 0u32);
    let stored = payout.read().iban.as_ref().map_or("none".to_string(), |iban| iban.0.clone());
    let shared = payout.read().share.as_ref().map_or("none".to_string(), |share| share.0.to_string());
    let phoned = payout.read().phone.as_ref().map_or("none".to_string(), |phone| phone.0.clone());
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { "data-submits": "{submits}", "data-stored": "{stored}", "data-share": "{shared}", "data-phone": "{phoned}" }
            Form {
                value: payout,
                onsubmit: move |_| submits += 1,
                TextField {
                    id: "iban",
                    label: "IBAN",
                    required: true,
                    name: Payout::FIELDS.iban().map(Iban::text, Iban::new),
                }
                NumberField::<f64> {
                    id: "share",
                    label: "Share",
                    name: Payout::FIELDS.share().map(|share: &Share| share.0, share),
                }
                PhoneField {
                    label: "Phone",
                    country: "DE",
                    name: Payout::FIELDS.phone().map(Phone::text, Phone::new),
                }
                Button { r#type: "submit", "Send" }
            }
        }
    }
}

/// Todo 2573: a server's status after a valid submit, set in `onsubmit` (`at_once`) or
/// later by `#answer`, shows the summary; `#code`'s rule, broken after the submit, does not.
#[component]
fn ServerStatusPage(at_once: bool) -> Element {
    let mut name = use_signal(|| "ada".to_string());
    let mut code = use_signal(|| "abcd".to_string());
    let mut server = use_signal(|| FieldStatus::Valid);
    let mut submits = use_signal(|| 0u32);
    let form = use_form();
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { id: "submits", "{submits}" }
            Form::<()> {
                form,
                onsubmit: move |_| {
                    submits += 1;
                    server.set(FieldStatus::Valid);
                    if at_once {
                        server.set(FieldStatus::Error(format!("Taken, submit {submits}.")));
                    }
                },
                TextField { id: "name", label: "Username", value: name(), oninput: move |text| name.set(text), status: server() }
                TextField {
                    id: "code",
                    label: "Code",
                    value: code(),
                    oninput: move |text| code.set(text),
                    validate: min_length(3).error("At least 3 characters."),
                }
                Button { id: "send", r#type: "submit", "Send" }
            }
            Button {
                id: "answer",
                onclick: move |_| server.set(FieldStatus::Error(format!("Taken, submit {submits}."))),
                "Answer"
            }
            Button { id: "break", onclick: move |_| code.set("a".to_string()), "Break code" }
            Button { id: "reset", onclick: move |_| form.reset(), "Reset" }
        }
    }
}

/// Todo 2572: `required` alone, no rules, blocks an empty submit; `#fill` fills all three.
/// Todo 2655: the empty, read-only `#note` never blocks. Todo 4091: `#colour` blocks once its
/// text is cleared and kept.
#[component]
fn RequiredPage() -> Element {
    let mut name = use_signal(String::new);
    let mut colour = use_signal(|| ColorCode::rgba(34, 139, 230, 1.0));
    let mut plan = use_signal(|| None::<Plan>);
    let mut tier = use_signal(|| None::<Plan>);
    let mut submits = use_signal(|| 0u32);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { id: "submits", "{submits}" }
            Form::<()> {
                onsubmit: move |_| submits += 1,
                TextField { id: "name", label: "Name", required: true, value: name(), oninput: move |text| name.set(text) }
                NativeSelect { id: "plan", label: "Plan", placeholder: "Pick one", required: true, value: plan(), onchange: move |next| plan.set(Some(next)) }
                Select { id: "tier", label: "Tier", placeholder: "Pick one", required: true, value: tier(), onchange: move |next| tier.set(next) }
                TextField { id: "note", label: "Note", required: true, readonly: true, value: "" }
                ColorField {
                    id: "colour",
                    label: "Colour",
                    required: true,
                    fix_on_blur: false,
                    value: colour(),
                    oninput: move |event: SliderChangeEvent<ColorCode>| colour.set(event.value()),
                }
                Button { id: "send", r#type: "submit", "Send" }
            }
            Button {
                id: "fill",
                onclick: move |_| {
                    name.set("Ada".to_string());
                    plan.set(Some(Plan::Pro));
                    tier.set(Some(Plan::Free));
                },
                "Fill"
            }
        }
    }
}

/// Todo 2608: a disabled group's legend and captions dim to `muted.6`, as `#probe` is.
#[component]
fn DisabledFieldsetPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Fieldset::<()> {
                id: "group",
                label: "Address",
                description: "Where we ship.",
                helper: "Kept private.",
                disabled: true,
                TextField { label: "Street" }
            }
            Text { id: "probe", sx: libero::sx::sx().color("muted.6"), "Probe" }
        }
    }
}

/// Todo 3873: the group renders no native `disabled`, so each named field inside
/// must leave the submit by its own; `outside` proves the submit is read.
#[component]
fn DisabledFieldsetSubmitPage() -> Element {
    let one = vec!["x".to_string()];
    let date = NaiveDate::from_ymd_opt(2026, 1, 2);
    rsx! {
        form { id: "sent",
            Fieldset::<()> { label: "Locked", disabled: true,
                TextField { label: "Text", name: "text", value: "x" }
                PasswordField { label: "Password", name: "password", value: "x" }
                Textarea { label: "Textarea", name: "textarea", value: "x" }
                NumberField::<u32> { label: "Number", name: "number", value: Some(3), onchange: |_| {} }
                PinField { label: "Pin", name: "pin", value: "12", length: 4usize }
                PhoneField { label: "Phone", name: "phone", value: Some("+4912345".to_string()) }
                TagsField { label: "Tags", name: "tags", value: one.clone(), onchange: |_| {} }
                Autocomplete { label: "City", name: "city", value: "x", options: one.clone(), oninput: |_| {} }
                Checkbox { label: "Check", name: "check", checked: true, onchange: |_| {} }
                Switch { label: "Switch", name: "switch", checked: true, onchange: |_| {} }
                Chip { name: "chip", checked: true, onchange: |_| {}, "Chip" }
                Radio { label: "Radio", name: "radio", value: "x", checked: true, onselect: |_| {} }
                RadioGroup::<Plan> { label: "Group", name: "group", value: Some(Plan::Pro) }
                NativeSelect::<Plan> { label: "Native", name: "native", value: Some(Plan::Pro), onchange: |_| {} }
                Select::<Plan> { label: "Select", name: "select", value: Some(Plan::Pro), onchange: |_| {} }
                MultiSelect { label: "Multi", name: "multi", options: one.clone(), value: one.clone(), onchange: |_| {} }
                SegmentedControl::<Plan> { label: "Segments", name: "segments", value: Some(Plan::Pro) }
                Cascader::<String> {
                    label: "Cascader",
                    name: "cascader",
                    data: vec![CascaderOption::new("x", "X")],
                    value: Some("x".to_string()),
                    onchange: |_| {},
                }
                Rating { label: "Rating", name: "rating", value: 3.0 }
                Slider { label: "Slider", name: "slider", value: 3.0 }
                RangeSlider { label: "Range", name: "range", value: (2.0, 4.0) }
                ColorField { label: "Colour", name: "colour", value: ColorCode::rgba(1, 2, 3, 1.0) }
                FileField { label: "File", name: "file" }
                DateField { label: "Date", name: "date", value: date }
                DateTimeField { label: "Date time", name: "datetime" }
                TimeField { label: "Time", name: "time" }
                // No soft-disabled form yet: `inert`, and a disabled hidden input.
                DatePicker { name: "date-picker", value: date }
                DateRangePicker { name: "range-picker" }
                MonthPicker { name: "month-picker" }
                YearPicker { name: "year-picker" }
                TimePicker { name: "time-picker" }
            }
            TextField { label: "Outside", name: "outside", value: "x" }
            button { id: "submit", r#type: "submit", "Send" }
        }
    }
}

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
            // A native reset: typed text a field keeps must go with it (2304).
            Button { r#type: "reset", "Clear" }
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
struct Login {
    username: String,
    password: String,
}

/// Todo 4094: a browser's saved-login autofill writes both inputs; the form must see the text.
#[component]
fn LoginPage() -> Element {
    let login = use_store(Login::default);
    let mut submits = use_signal(|| 0u32);
    let user = login.read().username.clone();
    let pass = login.read().password.clone();
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { id: "stored", "data-submits": "{submits}", "data-user": "{user}", "data-pass": "{pass}" }
            Form {
                value: login,
                onsubmit: move |_| submits += 1,
                TextField { id: "user", label: "Username", required: true, autocomplete: "username", name: Login::FIELDS.username() }
                PasswordField { id: "pass", label: "Password", required: true, autocomplete: "current-password", name: Login::FIELDS.password() }
                Button { id: "send", r#type: "submit", "Log in" }
            }
        }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Prefill {
    user: String,
    pass: String,
    note: String,
    count: Option<u32>,
    phone: String,
}

/// Todo 4094: `#show` mounts text-like fields into which the browser already wrote, as an
/// autofill into server-rendered HTML before hydration; the form must take that text.
#[component]
fn PrefillPage() -> Element {
    let value = use_store(Prefill::default);
    let mut shown = use_signal(|| false);
    let mut submits = use_signal(|| 0u32);
    let stored = value.read().clone();
    let count = stored.count.map_or("none".to_string(), |count| count.to_string());
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span {
                id: "stored",
                "data-submits": "{submits}",
                "data-user": "{stored.user}",
                "data-pass": "{stored.pass}",
                "data-note": "{stored.note}",
                "data-count": "{count}",
                "data-phone": "{stored.phone}",
            }
            Button { id: "show", onclick: move |_| shown.set(true), "Show" }
            if shown() {
                Form {
                    value,
                    onsubmit: move |_| submits += 1,
                    TextField { label: "Username", required: true, name: Prefill::FIELDS.user() }
                    PasswordField { label: "Password", required: true, name: Prefill::FIELDS.pass() }
                    Textarea { label: "Note", required: true, name: Prefill::FIELDS.note() }
                    NumberField::<u32> { label: "Count", required: true, name: Prefill::FIELDS.count() }
                    PhoneField { label: "Phone", required: true, country: "DE", name: Prefill::FIELDS.phone() }
                    Button { id: "send", r#type: "submit", "Send" }
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Fields)]
struct PrefillMore {
    city: String,
    tags: Vec<String>,
    pin: String,
    due: Option<NaiveDate>,
    color: ColorCode,
}

impl Default for PrefillMore {
    fn default() -> Self {
        Self {
            city: String::new(),
            tags: Vec::new(),
            pin: String::new(),
            due: None,
            color: ColorCode::rgba(0, 0, 0, 1.0),
        }
    }
}

/// Todo 4096: as `PrefillPage`, for the fields whose input is text on the way to another value.
#[component]
fn PrefillMorePage() -> Element {
    let value = use_store(PrefillMore::default);
    let mut shown = use_signal(|| false);
    let stored = value.read().clone();
    let due = stored.due.map_or("none".to_string(), |due| due.to_string());
    let tags = stored.tags.join("|");
    let color = stored.color.to_hex();
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span {
                id: "stored",
                "data-city": "{stored.city}",
                "data-tags": "{tags}",
                "data-pin": "{stored.pin}",
                "data-due": "{due}",
                "data-color": "{color}",
            }
            Button { id: "show", onclick: move |_| shown.set(true), "Show" }
            if shown() {
                Form {
                    value,
                    onsubmit: move |_| {},
                    Autocomplete::<String> {
                        id: "city",
                        label: "City",
                        options: vec!["Berlin".to_string(), "Paris".to_string()],
                        name: PrefillMore::FIELDS.city(),
                    }
                    TagsField { id: "tags", label: "Tags", name: PrefillMore::FIELDS.tags() }
                    PinField { id: "pin", label: "Code", length: 4usize, name: PrefillMore::FIELDS.pin() }
                    DateField { id: "due", label: "Due", format: "YYYY-MM-DD", name: PrefillMore::FIELDS.due() }
                    ColorField { id: "color", label: "Colour", name: PrefillMore::FIELDS.color() }
                }
            }
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

#[derive(Clone, PartialEq, Default, Fields)]
struct Order {
    name: String,
    company: String,
    #[fields(nested)]
    password: NewPassword,
    age: String,
    locked: String,
    frozen: String,
}

/// Todos 4207-4209: a field mounted late above a Fieldset rule, and a disabled and a readonly
/// field whose own rules fail.
#[component]
fn OrderPage(hidden: bool) -> Element {
    let value = use_store(Order::default);
    let mut company = use_signal(|| false);
    let mut calls = use_signal(|| 0u32);
    let summary = if hidden { SummaryMode::Hidden } else { SummaryMode::Visible };
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            span { "data-calls": "{calls}" }
            Button { onclick: move |_| company.set(true), "Add company" }
            Form { value, summary, onsubmit: move |_| calls += 1,
                TextField { id: "name", label: "Name", required: true, name: Order::FIELDS.name() }
                if company() {
                    TextField { id: "company", label: "Company name", required: true, name: Order::FIELDS.company() }
                }
                Fieldset {
                    label: "Password",
                    path: Order::FIELDS.password(),
                    validate: (|p: &NewPassword| !p.repeat.is_empty())
                        .error("Repeat the password.")
                        .on([NewPassword::FIELDS.repeat()]),
                    PasswordField { label: "Password", name: NewPassword::FIELDS.value() }
                    PasswordField { id: "repeat", label: "Repeat password", name: NewPassword::FIELDS.repeat() }
                }
                TextField { id: "age", label: "Age", required: true, name: Order::FIELDS.age() }
                TextField { id: "locked", label: "Locked", disabled: true, name: Order::FIELDS.locked(),
                    validate: not_empty.error("Locked is empty."),
                }
                TextField { id: "frozen", label: "Frozen", readonly: true, name: Order::FIELDS.frozen(),
                    validate: not_empty.error("Frozen is empty."),
                }
                Button { r#type: "submit", "Order" }
            }
        }
    }
}
