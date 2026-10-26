use crate::Route;
use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, Remount, SectionLink, Wrap, a11y, indent, prop,
    props,
};
use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, Code, FieldError, FieldName, Fields, Fieldset, Flex, Form, FormErrors,
    FormPart, PasswordField, Rule, Submit, SummaryMode, Text, TextField, Validators, checked,
    checked_async, is_email, min_length, not_empty, use_form, use_form_context,
};
use libero::localization::Localization;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    task::{Poll, Waker},
    time::Duration,
};

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
    pub username: Option<String>,
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
    password: NewPassword,__EXTRA_FIELDS__
    terms: bool,
}

#[component]
fn SignupForm() -> Element {
__STATE____HANDLE__

    rsx! {
        Form {__FORM__
            value: signup,
__RULE____SUMMARY__
__SUBMIT__
__FIELDS____USERNAME____COMPANY__
__TERMS__
__BUTTONS__
__STATUS__
        }
    }
}"#;

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("form.rs"));

/// A section of the live form at the depth `FORM_CODE` splices it in.
fn nest(name: &str) -> String {
    indent(&indent(&indent(&FILE.section(name))))
        .trim_end()
        .to_string()
}

fn form_code(values: &DemoValues, _: &str) -> String {
    let on = |name: &str| values.str(name) == "true";
    let (handle, loading, hidden, username) = (
        on("form"),
        on("async_submit"),
        on("summary_hidden"),
        on("username_check"),
    );
    let needs_form = handle || loading;
    let mut imports = Vec::new();
    if handle {
        imports.push("Flex");
    }
    if hidden {
        imports.push("SummaryMode");
    }
    if username {
        imports.push("checked_async");
    }
    if needs_form {
        imports.push("use_form");
    }
    if handle {
        imports.push("use_form_context");
    }
    let imports = match imports.is_empty() {
        true => String::new(),
        false => format!("\nuse libero::components::{{{}}};", imports.join(", ")),
    };
    let std_imports = match loading {
        true => "\nuse std::time::Duration;",
        false => "",
    };
    let mut extra_fields = String::new();
    if on("company") {
        extra_fields += "\n    company: bool,\n    company_name: String,";
    }
    if username {
        extra_fields += "\n    username: Option<String>,";
    }
    let optional = |is_on: bool, text: String| if is_on { text } else { String::new() };
    let mut options = String::new();
    if on("summary_title") {
        options += "\n            summary_title: \"Please fix these first:\",";
    }
    if hidden {
        options += "\n            summary: SummaryMode::Hidden,";
    }
    if username {
        options += "\n            pending_loader: true,";
    }
    let submit = format!(
        "            onsubmit: {},",
        nest(if loading {
            "async_submit"
        } else {
            "submit_sync"
        })
        .trim_start()
    );
    let buttons = match (handle, loading) {
        (true, _) => "handle_buttons",
        (false, true) => "loading_submit",
        (false, false) => "submit",
    };
    let code = FORM_CODE
        .replace("__STATE__", indent(&FILE.section("state")).trim_end())
        .replace("__RULE__", &nest("rule"))
        .replace("__FIELDS__", &nest("fields"))
        .replace("__TERMS__", &nest("terms"))
        .replace("__STATUS__", &nest("status"))
        .replace("__SUMMARY__", &options)
        .replace("__SUBMIT__", &submit)
        .replace("__IMPORTS__", &format!("{std_imports}{imports}"))
        .replace("__EXTRA_FIELDS__", &extra_fields)
        .replace(
            "__COMPANY__",
            &optional(on("company"), format!("\n{}", nest("company"))),
        )
        .replace(
            "__USERNAME__",
            &optional(username, format!("\n{}", nest("username"))),
        )
        .replace(
            "__HANDLE__",
            if needs_form {
                "\n    let form = use_form();"
            } else {
                ""
            },
        )
        .replace(
            "__FORM__",
            if needs_form {
                "\n            form,"
            } else {
                ""
            },
        )
        .replace("__BUTTONS__", &nest(buttons));
    let mut code = format!(
        "{code}\n\n{}\n\n{}",
        FILE.section("parse"),
        FILE.section("age")
    );
    if username {
        code += &format!("\n\n{}", FILE.section("username_check"));
    }
    if handle {
        code += &format!("\n\n{}", FILE.section("check_button"));
    }
    code
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
                    prop("pending_loader", "bool")
                        .default("false")
                        .doc("Draws a silent `Loader` at the end of a framed field's box while a `checked_async` rule checks its value, once the field shows its rules (after a blur or a submit). The box keeps its height; the control is `aria-busy=\"true\"` either way. Not in a `PinField`'s cells."),
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
                        .doc("Checks like a submit without calling `onsubmit`. Every status shows, and with an error the summary appears and takes focus. `true` when nothing is an error; a field an async rule is still checking counts as no error."),
                    prop("submit()", "Result<(), PlatformError>")
                        .doc("Submits as the submit button would. `Unsupported` once the form is gone."),
                    prop("reset()", "()")
                        .doc("Puts the value back to what the store held when the form mounted, or when another store was passed, and clears touched fields, the submit and the summary; a `type=\"reset\"` button does the same. A store loaded later, such as from a request, resets to its value at mount: key the `Form` on the record to start from it. A field with its own `value` and handler keeps what it shows. Under Blitz it clears unbound text fields, not checkboxes or selects."),
                    prop("is_valid()", "bool")
                        .doc("Whether nothing is an error, shown or not. It follows changes, so it can drive other UI."),
                    prop("is_submitting()", "bool")
                        .doc("Whether an async handler (`Submit::parse_async`, `Submit::task`) is running, or a submit waits for a `checked_async` rule's answer. It follows changes, so it can set a submit button's `loading`. Meanwhile a submit is ignored and the form has `aria-busy=\"true\"`."),
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
                        .doc("Words every error of type `E` on the field, the parse error, `checked` and `checked_async` rules alike. Needed when `E` has no `FieldError`, such as a domain crate's error: without it the name does not compile."),
                    prop(".rules(..)", "Validator<T>, Checked<T, E> or CheckedAsync<T, E>")
                        .doc("Rules over `T`, run only on a parsed value. The field's own `validate` runs first, over `W`, before the parse; the first error wins. Adds to earlier calls, so a mixed list is two calls."),
                    prop("checked(f)", "Checked<V, E>")
                        .doc("A rule from `Fn(&V) -> Result<(), E>`: the field shows `E`'s words. `.warning()` shows it as a warning, `.message(fn)` words an `E` with no `FieldError`. Fits any `validate` too."),
                    prop("checked_async(f)", "CheckedAsync<V, E>")
                        .doc("An async rule from `Fn(V) -> impl Future<Output = Result<(), E>>`, such as a request, for `.rules` only. It runs once the value parsed and the other rules found no error, after `.debounce(ms)` (300 by default); an answer for an older value is dropped. The answers for the last 8 values are kept, so going back to one asks again only after a reset. Meanwhile the field is `FieldStatus::Pending`, and a submit asks at once and waits for the answer. `.warning()` shows a failure as a warning, which a submit does not wait for; `.message(fn)` words an `E` that is not the field's error type."),
                    prop("pending", "FieldStatus::Pending")
                        .doc("An async rule is still checking: `aria-busy=\"true\"` on the control and `data-state` `pending`, no message, blocks nothing. It ranks below a warning and above `Valid`. A reset drops the check and its answer."),
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
                    Control::switch("summary_hidden").code(silent),
                    Control::switch("form").code(silent),
                    Control::switch("company").code(silent),
                    Control::switch("async_submit").code(silent),
                    Control::switch("username_check").code(silent),
                ],
                render: move |values: DemoValues| {
                    let on = |name: &str| values.str(name) == "true";
                    let (handle, async_submit) = (on("form"), on("async_submit"));
                    // Remounted: the `Form` takes its `form` handle once, on mount.
                    rsx! {
                        Remount { on: "{handle}{async_submit}",
                            SignupForm {
                                summary_title: on("summary_title"),
                                hidden: on("summary_hidden"),
                                handle,
                                company: on("company"),
                                async_submit,
                                username: on("username_check"),
                            }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn SignupForm(
    summary_title: bool,
    hidden: bool,
    handle: bool,
    company: bool,
    async_submit: bool,
    username: bool,
) -> Element {
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
            form: (handle || async_submit).then_some(form),
            value: signup,
            // demo-code: rule start
            // Only the whole signup sees the email and the password together.
            validate: (|s: &Signup| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Signup::FIELDS.password().value()]),
            // demo-code: rule end
            summary_title: summary_title.then(|| "Please fix these first:".to_string()),
            summary: if hidden { SummaryMode::Hidden } else { SummaryMode::Visible },
            pending_loader: username,
            onsubmit: if async_submit {
                // demo-code: async_submit start
                Submit::parse_async(new_account, move |account: NewAccount| async move {
                    // While the request runs, the form ignores another submit.
                    delay(Duration::from_secs(1)).await;
                    if account.email == "locked@example.com" {
                        // No field is at fault, so the error shows above the fields.
                        return Err(FormErrors::new().form("This account is locked."));
                    }
                    sent.set(account.email);
                    Ok(())
                })
                // demo-code: async_submit end
            } else {
                // demo-code: submit_sync start
                Submit::parse(new_account, move |account: NewAccount| sent.set(account.email))
                // demo-code: submit_sync end
            },
            // demo-code: fields start
            // `required` alone blocks an empty submit with "Fill in this field.".
            TextField { label: "Name", name: Signup::FIELDS.name(), required: true }
            // A specialized field and a composed part, built in Getting started.
            EmailField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            NewPasswordFieldset { path: Signup::FIELDS.password() }
            // The text parses into an `Age`; a rule on the age runs only once it did.
            TextField { label: "Age", name: Signup::FIELDS.age().parsed().rules([checked(Age::adult)]) }
            // demo-code: fields end
            if username {
                // demo-code: username start
                // Asked 400 ms after the last keystroke, once the name parsed.
                TextField {
                    label: "Username",
                    name: Signup::FIELDS.username().map(String::clone, username_text)
                        .message(username_message)
                        .rules([checked_async(username_free).debounce(400)]),
                }
                // demo-code: username end
            }
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
                    Button { r#type: "submit", loading: form.is_submitting(), "Create account" }
                    CheckButton {}
                    Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
                }
                // demo-code: handle_buttons end
            } else if async_submit {
                // demo-code: loading_submit start
                Button { r#type: "submit", loading: form.is_submitting(), "Create account" }
                // demo-code: loading_submit end
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

// demo-code: username_check start
enum UsernameError {
    Spaces,
    Taken,
}

fn username_text(text: String) -> Result<String, UsernameError> {
    match text.contains(' ') {
        true => Err(UsernameError::Spaces),
        false => Ok(text),
    }
}

fn username_message(error: &UsernameError, _: &Localization) -> String {
    match error {
        UsernameError::Spaces => "Use no spaces.".into(),
        UsernameError::Taken => "This name is taken. Try another.".into(),
    }
}

/// Stands in for a server: a second's wait, then `ada` is taken.
async fn username_free(name: String) -> Result<(), UsernameError> {
    delay(Duration::from_secs(1)).await;
    match name == "ada" {
        true => Err(UsernameError::Taken),
        false => Ok(()),
    }
}
// demo-code: username_check end

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

async fn delay(duration: Duration) {
    let Some(timer) = libero::platform::timer() else {
        return;
    };
    let done = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None::<Waker>));
    let (fired, wake) = (done.clone(), waker.clone());
    let _pending = timer.after(
        duration,
        Box::new(move || {
            fired.set(true);
            if let Some(waker) = wake.borrow_mut().take() {
                waker.wake();
            }
        }),
    );
    std::future::poll_fn(|context| match done.get() {
        true => Poll::Ready(()),
        false => {
            *waker.borrow_mut() = Some(context.waker().clone());
            Poll::Pending
        }
    })
    .await
}
