use crate::Route;
use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, DocSection, Remount, SectionLink, Wrap, a11y,
    indent, prop, props,
};
use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, Code, CodeBlock, FieldError, FieldName, Fields, Fieldset, Flex, Form,
    FormErrors, FormPart, PasswordField, Rule, Submit, Text, TextField, Validators, checked,
    is_email, min_length, not_empty, use_form, use_form_context,
};
use libero::localization::Localization;

#[derive(Clone, PartialEq, Default, Fields)]
pub struct NewPassword {
    pub value: String,
    pub repeat: String,
}

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Signup {
    pub name: String,
    pub email: String,
    pub age: Option<Age>,
    #[fields(nested)]
    pub password: NewPassword,
    pub company: bool,
    pub company_name: String,
    pub terms: bool,
}

// snippet: ignore - builds on Getting Started's `EmailField` and `NewPasswordFieldset`
// snippet: mirrors SignupForm, Signup, Age, AgeError
const FORM_CODE: &str = r#"use dioxus::prelude::*;
use libero::components::{Button, Checkbox, FieldError, Fields, Form, FormErrors, Rule, Submit, Text, TextField, checked, not_empty};
use libero::localization::Localization;__IMPORTS__

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    name: String,
    email: String,
    age: Option<Age>,
    #[fields(nested)]
    password: NewPassword,__COMPANY_FIELDS__
    terms: bool,
}

#[component]
fn SignupForm() -> Element {
__STATE____HANDLE__

    rsx! {
        Form {__FORM__
            value: signup,
__RULE____SUMMARY__
            onsubmit: Submit::parse(new_account, move |account: NewAccount| sent.set(account.email)),
__FIELDS____COMPANY__
__TERMS__
__BUTTONS__
__STATUS__
        }
    }
}"#;

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("form.rs"));

// Prints from its `demo-code: start` marker, below the imports it compiles with.
#[rustfmt::skip]
mod sign_in;
use sign_in::SignInForm;
const SIGN_IN_CODE: DemoFile = DemoFile(include_str!("form/sign_in.rs"));

const UPGRADE_CODE: &str = r#"use dioxus::prelude::*;
use libero::components::Form;
use libero::localization::{FormLabels, Localization};

/// Written for the old `onsubmit`, and still forwarding it.
#[component]
fn AppForm(onsubmit: Option<EventHandler<FormEvent>>, children: Element) -> Element {
    rsx! {
        Form::<()> { onsubmit, {children} }
    }
}

static WORDS: Localization = Localization {
    form: FormLabels {
        required: "This one is needed.",
        ..FormLabels::ENGLISH
    },
    ..Localization::ENGLISH
};"#;

/// A section of the live form at the depth `FORM_CODE` splices it in.
fn nest(name: &str) -> String {
    indent(&indent(&indent(&FILE.section(name))))
        .trim_end()
        .to_string()
}

fn form_code(values: &DemoValues, _: &str) -> String {
    let handle = values.str("form") == "true";
    let pick = |on: &'static str| if handle { on } else { "" };
    let company = |on: &'static str| {
        if values.str("company") == "true" {
            on
        } else {
            ""
        }
    };
    let company_fields = match values.str("company") == "true" {
        true => format!("\n{}", nest("company")),
        false => String::new(),
    };
    let code = FORM_CODE
        .replace("__STATE__", indent(&FILE.section("state")).trim_end())
        .replace("__RULE__", &nest("rule"))
        .replace("__FIELDS__", &nest("fields"))
        .replace("__TERMS__", &nest("terms"))
        .replace("__STATUS__", &nest("status"))
        .replace(
            "__SUMMARY__",
            match values.str("summary_title").as_str() {
                "true" => "\n            summary_title: \"Please fix these first:\",",
                _ => "",
            },
        )
        .replace(
            "__IMPORTS__",
            pick("\nuse libero::components::{Flex, use_form, use_form_context};"),
        )
        .replace(
            "__COMPANY_FIELDS__",
            company("\n    company: bool,\n    company_name: String,"),
        )
        .replace("__COMPANY__", &company_fields)
        .replace("__HANDLE__", pick("\n    let form = use_form();"))
        .replace("__FORM__", pick("\n            form,"))
        .replace(
            "__BUTTONS__",
            &nest(if handle { "handle_buttons" } else { "submit" }),
        );
    let code = format!(
        "{code}\n\n{}\n\n{}",
        FILE.section("parse"),
        FILE.section("age")
    );
    match handle {
        true => format!("{code}\n\n{}", FILE.section("check_button")),
        false => code,
    }
}

fn silent(_: &Control, _: &DemoValues) -> Vec<String> {
    Vec::new()
}

#[component]
pub fn FormPage() -> Element {
    rsx! {
        DocPage {
            title: "Form",
            source: "libero/src/components/form/form.rs",
            markdown: "/md/form.md",
            properties: vec![
                props("Form", vec![
                    prop("value", "Store<V>")
                        .doc("The whole form's value. `validate` checks it, and fields named by a path read and write it. `V` is inferred from it. A form with no `value` and no rules needs `Form::<()>`."),
                    prop("validate", "Validators<V>")
                        .doc("Rules over `value`, one or an array. A rule with `.on(..)` shows its status on each field it names, and runs only while each of them holds a parsed value or is empty and optional."),
                    prop("onsubmit", "Submit<V>")
                        .doc("Fires on a submit with no errors: a closure over the `FormEvent` or an `EventHandler<FormEvent>`, optional too, `Submit::parse(parse, handler)`, or the async `Submit::parse_async(parse, handler)` and `Submit::task(handler)`. Without an `action`, the browser's own submit is cancelled. A `javascript:`, `data:` or `vbscript:` `action` is dropped with a debug warning, so it counts as none."),
                    prop("summary_title", "String")
                        .doc("A heading over the error summary."),
                    prop("summary", "SummaryMode")
                        .default("Visible")
                        .doc("`Visible`: a blocked submit or `set_errors` shows a summary of every error above the fields and focuses it. `Hidden`: no summary list; focus goes to the first field that shows its error, and a visually hidden status says how many errors there are (`form.error_count`). Errors no field shows, from `FormErrors::form` or a form rule without `.on(..)`, still show in the summary, which then takes focus when no field does."),
                    prop("form", "FormHandle")
                        .doc("Controls the form from outside, made with `use_form()`. Taken once, on mount. Without it the form makes its own, which `use_form_context()` returns inside the form."),
                    prop("children", "Element")
                        .default("required")
                        .doc("The fields, fieldsets and buttons."),
                ]).extends("form")
                .parts("FormPart", vec![
                    (FormPart::Summary, "The error summary, an `Alert`, shown after a blocked submit."),
                    (FormPart::SummaryTitle, "The summary's heading, with `summary_title`."),
                    (FormPart::SummaryList, "The summary's list of errors."),
                ]),
                props("FormHandle", vec![
                    prop("validate()", "bool")
                        .doc("Checks like a submit without calling `onsubmit`. Every status shows, and with an error the summary appears and takes focus. `true` when nothing is an error."),
                    prop("submit()", "Result<(), PlatformError>")
                        .doc("Submits as the submit button would. `Unsupported` once the form is gone."),
                    prop("reset()", "()")
                        .doc("Puts the value back to `V::default()` and clears touched fields, the submit and the summary; a `type=\"reset\"` button clears those too. A field with its own `value` and handler keeps what it shows. On the desktop WebView it cannot clear a control that is not bound to the value. Under Blitz it clears unbound text fields, not checkboxes or selects."),
                    prop("is_valid()", "bool")
                        .doc("Whether nothing is an error, shown or not. It follows changes, so it can drive other UI."),
                    prop("is_submitting()", "bool")
                        .doc("Whether an async handler (`Submit::parse_async`, `Submit::task`) is running. It follows changes, so it can set a submit button's `loading`. Meanwhile a submit is ignored and the form has `aria-busy=\"true\"`."),
                    prop("clear_summary()", "()")
                        .doc("Hides the summary and resets nothing."),
                    prop("set_errors(errors)", "FormErrors")
                        .doc("Shows each error on the field with its path, as an error `status` would, lists it in the summary and focuses the summary: a server's answer. Replaces the errors set before. A field's error clears when it is edited, all of them on the next submit or reset. An error on a path no field carries, or from `FormErrors::form`, gets an unlinked summary line."),
                    prop("clear_errors()", "()")
                        .doc("Drops the errors `set_errors` set."),
                ]).without_base_props(),
                props("Every field", vec![
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the label. Inside a `Form`, an empty field fails the submit with `form.required` unless its own rules fail first: `form.required_check` for an unchecked `Checkbox` or `Switch`, `form.required_select` for a select, `form.required_option` for a `RadioGroup` or `SegmentedControl`. A read-only field never fails it. A slider and a `ColorField` always hold a value, and a lone `Radio` leaves it to its `RadioGroup`. A field with a handler but no `value` or binding counts as filled: pass its `value` or a `name` path."),
                    prop("validate", "Validators<T>")
                        .doc("Rules over the field's own value, one or an array. Shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<T>")
                        .doc("What the field posts as, and how rules address it. A path from `#[derive(Fields)]` also binds the field to the form's value, unless the field has a handler of its own. `T` is the field's value type."),
                ]).without_base_props(),
                props("Domain values", vec![
                    prop("parse(parse, handler)", "Submit<V>")
                        .doc("`Submit::parse(parse, handler)`, for `onsubmit`: after the field rules pass, `parse: Fn(&V) -> Result<C, FormErrors>` builds a command from `value` and `handler` gets it. An `Err` puts each error on its field and in the summary, as `set_errors` does, and `handler` doesn't run. Hold checks in `validate` for live feedback; `parse` builds the command and reports what only the submit can tell."),
                    prop("parse_async(parse, handler)", "Submit<V>")
                        .doc("`Submit::parse` with an async `handler: FnMut(C) -> impl Future<Output = Result<(), FormErrors>>`, such as a request. It runs in the form's scope and ends with it. While it runs, `is_submitting()` is `true` and a submit is ignored; an `Err` lands as `set_errors` does. `Submit::task(handler)` is the same over the `FormEvent`, without `parse`."),
                    prop(".map(format, parse)", "Mapped<T, W, E>")
                        .doc("On a path to `Option<T>`: binds a field that edits `W` (text, a date, an option) to the domain type `T`. `format` shows a stored `T`, `parse` reads `W` back and never sees an empty one (blank text, `None`, `[]`); text reaches it untrimmed. An `Err` shows its words, resets the value to `None` and blocks the submit, while the text stays, also across an unmount. A focused field keeps the text as typed: `format` runs on blur and when the value changes elsewhere. `T` must be `Clone + PartialEq`. `Rc<T>` is no way out: it compares only when `T: PartialEq`. Wrap a type with neither, such as a secret, in `ByAddress<T>`."),
                    prop(".map(.., ByAddress::new)", "ByAddress<T>")
                        .doc("Makes a `T` with no `Clone` or `PartialEq`, such as a password type, fit for a form value: clones share it, two compare equal only when they are the same one, and it derefs to `T`. A field holds `Option<ByAddress<T>>`, and its `.map` parse wraps the value with `ByAddress::new`."),
                    prop(".parsed()", "Mapped<T, String, T::Err>")
                        .doc("`.map` through `T`'s `Display` and `FromStr`, for a text field."),
                    prop(".message(fn)", "fn(&E, &Localization) -> String")
                        .doc("Words every error of type `E` on the field, the parse error and `checked` rules alike. Needed when `E` has no `FieldError`, such as a domain crate's error: without it the name does not compile."),
                    prop(".rules(..)", "Validator<T> or Checked<T, E>")
                        .doc("Rules over `T`, run only on a parsed value. The field's own `validate` runs first, over `W`, before the parse; the first error wins. Adds to earlier calls, so a mixed list is two calls."),
                    prop("checked(f)", "Checked<V, E>")
                        .doc("A rule from `Fn(&V) -> Result<(), E>`: the field shows `E`'s words. `.warning()` shows it as a warning, `.message(fn)` words an `E` with no `FieldError`. Fits any `validate` too."),
                    prop("impl FieldError", "trait")
                        .doc("`fn message(&self, &Localization) -> String`: an error's words in the active language, worded again on a switch. Implemented for `Fn(&Localization) -> String`, `String` and `&'static str`; there is no `Display` fallback."),
                    prop(".at(path, error)", "FormErrors")
                        .doc("Errors by field path for `set_errors` and `parse`: `FormErrors::new().at(path, error)` with a `FieldError`, or `.at_with(path, error, fn)` worded by a `fn(&E, &Localization) -> String`. `path` is a `FieldPath`, a `FieldName` or its spelling. `.form(error)` is an error on the whole form, such as wrong credentials: an unlinked summary line, shown with `SummaryMode::Hidden` too, cleared by the next submit, a reset or `clear_errors`, not by an edit."),
                    prop(".required(path)", "Result<T, FormErrors>")
                        .doc("On an `Option<T>`, with `RequiredValue` in scope: the value, else an error on `path` worded as an empty `required` field. Turns a missing value into a `FormErrors` entry with `?` instead of an `unwrap()`."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .handles([
                    "A form becomes a `form` landmark only once it has a name. A form without a name is still valid.",
                    "Each summary line starts with its field's label, or its `aria_label` when it has no text label.",
                    "A `Fieldset` with an error `status` is one field in error: it blocks the submit and has a line named by its legend.",
                    "A summary line of a rule with `.on(..)` links to the first field it names that is on the page; a rule on the whole form that names no field links to the form's first field.",
                    "A rule shows once focus leaves its field, not on a move inside it, such as from one `PinField` cell to the next. The desktop WebView cannot tell the two apart, so there any move shows it.",
                ])
                .must([
                    "Name the form when the page holds more than one form, or when the form is the page's main task, such as a checkout. Pass `aria-labelledby` pointing at a visible heading, or `aria-label`.",
                    "Join intro text through `aria-describedby`, if any.",
                    "Put the field's name into its messages when its label is markup (`Caption::Node`): the summary cannot read markup, so the line has no name.",
                ])
                .example("A checkout, `Form { \"aria-labelledby\": \"checkout-title\", .. }` under a visible \"Checkout\" heading: a screen reader lists it as the \"Checkout\" form landmark, and each line of the error summary starts with its field's label."),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "<form novalidate>" }
                    " that holds the whole value in one store, runs rules across its fields and "
                    "validates on submit. The Form getting started page shows how fields, "
                    "fieldsets, rules and paths fit together, what a "
                    SectionLink { to: Route::FormGettingStartedPage {}, section: "submit", "submit" }
                    " shows and blocks, and how "
                    SectionLink { to: Route::FormGettingStartedPage {}, section: "conditional-fields", "conditional fields" }
                    " drop their rules."
                }
            },
            // snippet: ignore - builds on Getting Started's `EmailField` and `NewPasswordFieldset`
            Demo {
                component: "Form",
                children_text: "",
                wrap: Wrap(form_code),
                // `form` passes a `use_form()` handle, and the buttons
                // that use it appear.
                controls: vec![
                    Control::switch("summary_title").code(silent),
                    Control::switch("form").code(silent),
                    Control::switch("company").code(silent),
                ],
                render: move |values: DemoValues| {
                    let handle = values.str("form") == "true";
                    // Remounted: the `Form` takes its `form` handle once, on mount.
                    rsx! {
                        Remount { on: "{handle}",
                            SignupForm {
                                summary_title: values.str("summary_title") == "true",
                                handle,
                                company: values.str("company") == "true",
                            }
                        }
                    }
                },
            }
            DocSection { id: "async-submit", title: "Async submit",
                Text {
                    Code { source: "Submit::parse_async(parse, handler)" }
                    " runs an async "
                    Code { source: "handler" }
                    ", such as a request. "
                    Code { source: "form.is_submitting()" }
                    " is "
                    Code { source: "true" }
                    " until it ends, so the button shows "
                    Code { source: "loading" }
                    ", and a second submit meanwhile is ignored. An "
                    Code { source: "Err" }
                    " it returns lands like "
                    Code { source: "set_errors" }
                    ", and "
                    Code { source: "FormErrors::form(..)" }
                    " is an error on no field, such as wrong credentials. With "
                    Code { source: "summary: SummaryMode::Hidden" }
                    " a blocked submit focuses the first invalid field instead of a summary, and "
                    "a screen reader hears how many errors there are. The password is a type with no "
                    Code { source: "Clone" }
                    " or "
                    Code { source: "PartialEq" }
                    ", held as "
                    Code { source: "Option<ByAddress<Password>>" }
                    " and bound with "
                    Code { source: ".map" }
                    ". Sign in empty, then with any email and a wrong password, then with "
                    Code { source: "secret" }
                    "."
                }
                SignInForm {}
                CodeBlock { source: SIGN_IN_CODE.shown(), language: "rust" }
            }
            DocSection { id: "upgrading", title: "Upgrading",
                Text {
                    Code { source: "onsubmit" }
                    " was an "
                    Code { source: "Option<EventHandler<FormEvent>>" }
                    " before it became "
                    Code { source: "Submit<V>" }
                    ". A closure still fits, and so does a handler a wrapper forwards, "
                    "with or without the "
                    Code { source: "Option" }
                    ". "
                    Code { source: "FormLabels" }
                    " gains fields in minor releases, such as "
                    Code { source: "error_count" }
                    ", so build yours from a built-in set rather than listing every field."
                }
                CodeBlock { source: UPGRADE_CODE, language: "rust" }
            }
        }
    }
}

#[component]
fn SignupForm(summary_title: bool, handle: bool, company: bool) -> Element {
    // demo-code: state start
    let signup = use_store(Signup::default);
    let mut sent = use_signal(String::new);
    // demo-code: state end
    // Always made, so the hook order holds; the form only gets it while the
    // switch is on.
    let form = use_form();

    rsx! {
        Form {
            sx: libero::sx::sx().width("100%").max_width("320px"),
            form: handle.then_some(form),
            value: signup,
            // demo-code: rule start
            // Only the whole signup sees the email and the password together.
            validate: (|s: &Signup| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Signup::FIELDS.password().value()]),
            // demo-code: rule end
            summary_title: summary_title.then(|| "Please fix these first:".to_string()),
            onsubmit: Submit::parse(new_account, move |account: NewAccount| sent.set(account.email)),
            // demo-code: fields start
            // `required` alone blocks an empty submit with "Fill in this field.".
            TextField { label: "Name", name: Signup::FIELDS.name(), required: true }
            // A specialized field and a composed part, built in Getting started.
            EmailField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            NewPasswordFieldset { path: Signup::FIELDS.password() }
            // The text parses into an `Age`; a rule on the age runs only once it did.
            TextField { label: "Age", name: Signup::FIELDS.age().parsed().rules([checked(Age::adult)]) }
            // demo-code: fields end
            if company {
                // demo-code: company start
                Checkbox { label: "Sign up as a company", name: Signup::FIELDS.company() }
                // Rendered only while ticked, so its rules leave with it.
                if signup.read().company {
                    TextField {
                        label: "Company name",
                        name: Signup::FIELDS.company_name(),
                        validate: not_empty.error("Enter the company name."),
                    }
                }
                // demo-code: company end
            }
            // demo-code: terms start
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                validate: not_empty.error("Accept the terms to continue."),
            }
            // demo-code: terms end
            if handle {
                // demo-code: handle_buttons start
                Text { role: "status", if form.is_valid() { "Ready to send." } else { "Not ready yet." } }
                Flex { gap: "sm",
                    Button { r#type: "submit", "Create account" }
                    CheckButton {}
                    Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
                }
                // demo-code: handle_buttons end
            } else {
                // demo-code: submit start
                Button { r#type: "submit", "Create account" }
                // demo-code: submit end
            }
            // demo-code: status start
            // Mounted before the submit, so a screen reader hears the text it gains.
            Text { role: "status",
                if !sent().is_empty() { "Account created for {sent}." }
            }
            // demo-code: status end
        }
    }
}

// demo-code: parse start
/// What the form hands over: built from the value once every field passed.
pub struct NewAccount {
    pub email: String,
}

/// Reports what only the whole submit can tell; try `taken@example.com`.
fn new_account(signup: &Signup) -> Result<NewAccount, FormErrors> {
    if signup.email == "taken@example.com" {
        let taken = FormErrors::new().at(Signup::FIELDS.email(), "This address is taken.");
        return Err(taken);
    }
    Ok(NewAccount {
        email: signup.email.clone(),
    })
}
// demo-code: parse end

// demo-code: age start
/// A domain type: text that parses is an age, and its errors carry their own words.
#[derive(Clone, PartialEq)]
pub struct Age(u8);

pub enum AgeError {
    NotANumber,
    TooYoung,
}

impl FieldError for AgeError {
    fn message(&self, locale: &Localization) -> String {
        match (self, locale.lang) {
            (AgeError::NotANumber, "de") => "Gib dein Alter in Jahren ein.".into(),
            (AgeError::NotANumber, _) => "Enter your age in years.".into(),
            (AgeError::TooYoung, "de") => "Du musst mindestens 16 sein.".into(),
            (AgeError::TooYoung, _) => "You must be 16 or older.".into(),
        }
    }
}

impl std::str::FromStr for Age {
    type Err = AgeError;

    fn from_str(text: &str) -> Result<Age, AgeError> {
        text.trim()
            .parse()
            .map(Age)
            .map_err(|_| AgeError::NotANumber)
    }
}

impl std::fmt::Display for Age {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl Age {
    fn adult(&self) -> Result<(), AgeError> {
        match self.0 >= 16 {
            true => Ok(()),
            false => Err(AgeError::TooYoung),
        }
    }
}
// demo-code: age end

// demo-code: check_button start
/// Anything inside a form reaches its handle without a prop.
#[component]
fn CheckButton() -> Element {
    let form = use_form_context();
    rsx! {
        Button {
            variant: "tonal",
            onclick: move |_| {
                if let Some(form) = form {
                    form.validate();
                }
            },
            "Check"
        }
    }
}
// demo-code: check_button end

/// The specialized field the example uses, the one Getting started builds.
#[component]
fn EmailField(
    #[props(into)] label: String,
    #[props(default, into)] name: FieldName<String>,
    #[props(default, into)] validate: Validators<String>,
) -> Element {
    let rules = validate.with(is_email.error("That is not an email address."));
    rsx! {
        TextField { r#type: "email", autocomplete: "email", label, name, validate: rules, placeholder: "you@example.com" }
    }
}

/// The composed part the example uses: a new password and its repetition.
#[component]
fn NewPasswordFieldset(#[props(into)] path: FieldName<NewPassword>) -> Element {
    rsx! {
        Fieldset {
            label: "Choose a password",
            path,
            validate: (|p: &NewPassword| p.value == p.repeat)
                .error("The passwords differ.")
                .on([NewPassword::FIELDS.repeat()]),
            PasswordField {
                label: "New password",
                autocomplete: "new-password",
                name: NewPassword::FIELDS.value(),
                validate: min_length(8).error("Use at least 8 characters."),
            }
            PasswordField {
                label: "Repeat password",
                autocomplete: "new-password",
                name: NewPassword::FIELDS.repeat(),
            }
        }
    }
}
