# Form

Crate: `libero`
Import: `use libero::components::{Form, Fields, Rule, use_form, use_form_context};`
Feature: `form-root` (on by default, turns on `alert`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/form.rs>
Index: [index.md](index.md) lists every other page
Description: A `<form>` that validates on submit, with plain `Fn(&V) -> bool` rules, typed field paths from `#[derive(Fields)]` and a focused error summary.

A `<form novalidate>` that holds the whole value in one store, runs rules
across its fields and validates on submit. The Form
[getting started](form_getting_started.md) page shows how fields, fieldsets,
rules and paths fit together.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Checkbox, FieldError, Fields, Form, FormErrors, Rule, Submit, Text, TextField, checked, not_empty};
use libero::localization::Localization;

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    name: String,
    email: String,
    age: Option<Age>,
    #[fields(nested)]
    password: NewPassword,
    terms: bool,
}

#[component]
fn SignupForm() -> Element {
    let signup = use_store(Signup::default);
    let mut sent = use_signal(String::new);

    rsx! {
        Form {
            value: signup,
            // Only the whole signup sees the email and the password together.
            validate: (|s: &Signup| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Signup::FIELDS.password().value()]),
            onsubmit: Submit::parse(new_account, move |account: NewAccount| sent.set(account.email)),
            // `required` alone blocks an empty submit with "Fill in this field.".
            TextField { label: "Name", name: Signup::FIELDS.name(), required: true }
            // A specialized field and a composed part, built in Getting started.
            EmailField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            NewPasswordFieldset { path: Signup::FIELDS.password() }
            // The text parses into an `Age`; a rule on the age runs only once it did.
            TextField { label: "Age", name: Signup::FIELDS.age().parsed().rules([checked(Age::adult)]) }
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                validate: not_empty.error("Accept the terms to continue."),
            }
            Button { r#type: "submit", "Create account" }
            // Mounted before the submit, so a screen reader hears the text it gains.
            Text { role: "status",
                if !sent().is_empty() { "Account created for {sent}." }
            }
        }
    }
}

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
        text.trim().parse().map(Age).map_err(|_| AgeError::NotANumber)
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
#
# use libero::components::{FieldName, Validators};
# #[derive(Clone, PartialEq, Default, Fields)]
# struct NewPassword { value: String, confirm: String }
# #[component]
# fn EmailField(#[props(into)] label: String, #[props(default, into)] name: FieldName<String>,
#     #[props(default, into)] validate: Validators<String>) -> Element { rsx! {} }
# #[component]
# fn NewPasswordFieldset(#[props(into)] path: FieldName<NewPassword>) -> Element { rsx! {} }
```

## Conditional fields

A field shown only for some values is a plain `if` around it. Its rules leave
with it, so a hidden field never blocks a submit, and neither does a rule whose
`.on(..)` names only hidden fields. Its value stays in the store, and a rule on
the whole form that names no field still runs, so check the condition there too.

```rust
# use dioxus::prelude::*;
# use libero::components::{Checkbox, Fields, Form, Rule, TextField, not_empty};
# #[derive(Clone, PartialEq, Default, Fields)]
# struct Signup { company: bool, company_name: String }
# #[component]
# fn CompanyFields() -> Element {
#     let signup = use_store(Signup::default);
#     rsx! { Form { value: signup,
Checkbox { label: "Sign up as a company", name: Signup::FIELDS.company() }
// Rendered only while ticked, so its rules leave with it.
if signup.read().company {
    TextField {
        label: "Company name",
        name: Signup::FIELDS.company_name(),
        validate: not_empty.error("Enter the company name."),
    }
}
# } } }
```

## Submit

A submit shows every status. With an error, it is cancelled, and a summary of
every problem appears above the fields and takes focus. Warnings never block.
A `required` field left empty is an error too, "Fill in this field.", or
"Check this box.", "Select an item in the list." and "Select one of these
options." for a box, a select or a group of options (`Localization::form`), unless a rule of its own says more; a disabled or
read-only one never blocks. That error shows only once the form was submitted,
so tabbing through an empty required field paints nothing; after that it
follows the value like any rule. A field's own rules show once it loses focus.
The summary keeps the problems of that submit. A line leaves once it is fixed,
and new ones wait for the next submit. A server's answer is a field's `status`:
an error one set after a valid submit, in `onsubmit` or once a request returns,
shows the summary, which takes focus and is announced, as a blocked submit's
would.

## Async submit

`Submit::parse_async(parse, handler)` runs an async `handler`, such as a
request. `form.is_submitting()` is `true` until it ends, so the button shows
`loading`, and a second submit meanwhile is ignored. An `Err` it returns lands
like `set_errors`, and `FormErrors::form(..)` is an error on no field, such as
wrong credentials. With `summary: SummaryMode::Hidden` a blocked submit focuses
the first invalid field instead of a summary, and a screen reader hears how
many errors there are. The password is a type with no `Clone` or `PartialEq`,
held as `Option<ByAddress<Password>>` and bound with `.map`.

```rust
use dioxus::prelude::*;
use libero::components::{
    Button, ByAddress, Fields, Form, FormErrors, PasswordField, RequiredValue, Submit, SummaryMode,
    Text, TextField, use_form,
};
use std::convert::Infallible;

/// A secret: no `Clone` and no `PartialEq`, so it is never copied by accident.
struct Password(String);

#[derive(Clone, PartialEq, Default, Fields)]
struct Login {
    email: String,
    // `ByAddress` makes the secret fit for a form value.
    password: Option<ByAddress<Password>>,
}

fn password_text(password: &ByAddress<Password>) -> String {
    password.0.clone()
}

fn password(text: String) -> Result<ByAddress<Password>, Infallible> {
    Ok(ByAddress::new(Password(text)))
}

struct Credentials {
    email: String,
    password: ByAddress<Password>,
}

fn credentials(login: &Login) -> Result<Credentials, FormErrors> {
    let password = login.password.clone().required(Login::FIELDS.password())?;
    Ok(Credentials { email: login.email.clone(), password })
}

#[component]
fn SignInForm() -> Element {
    let login = use_store(Login::default);
    let form = use_form();
    let mut signed_in = use_signal(String::new);

    rsx! {
        Form {
            sx: libero::sx::sx().width("100%").max_width("320px"),
            form,
            value: login,
            // Each field shows its own error, and focus goes to the first one.
            summary: SummaryMode::Hidden,
            // While the request runs, the form ignores another submit.
            onsubmit: Submit::parse_async(credentials, move |sent: Credentials| async move {
                if !sign_in(&sent).await {
                    // No field is at fault, so the error shows above the fields.
                    return Err(FormErrors::new().form("Wrong email or password."));
                }
                signed_in.set(sent.email);
                Ok(())
            }),
            TextField { label: "Email", r#type: "email", autocomplete: "username", name: Login::FIELDS.email(), required: true }
            PasswordField {
                label: "Password",
                autocomplete: "current-password",
                name: Login::FIELDS.password().map(password_text, password),
                required: true,
            }
            Button { r#type: "submit", loading: form.is_submitting(), "Sign in" }
            Text { role: "status",
                if !signed_in().is_empty() { "Signed in as {signed_in}." }
            }
        }
    }
}
#
# async fn sign_in(sent: &Credentials) -> bool { sent.password.0 == "secret" }
```

`Submit::task(handler)` is the same over the `FormEvent`, without a `parse`
step. A type with neither `Clone` nor `PartialEq`, such as a password type,
fits a form value as `ByAddress<T>`: clones share it and two compare equal only
when they are the same one.

## Upgrading

`onsubmit` was an `Option<EventHandler<FormEvent>>` before it became
`Submit<V>`. A closure still fits, and so does a handler a wrapper forwards,
with or without the `Option`. `FormLabels` gains fields in minor releases, such
as `error_count`, so build yours from a built-in set rather than listing every
field.

```rust
use dioxus::prelude::*;
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
};
```

## Sticky action bar

A long form keeps its buttons in view with a `Flex` that sticks to the bottom of
the scrolling area. It is the last child of the `Form`, so its submit button still
submits, and a surface background keeps the fields from showing through. Give it
a `z_index` above the fields. The bar keeps its place in the DOM, so Tab still
reaches it after the fields. It needs a scrolling ancestor, such as the page or
a `Paper` with a fixed height.

```rust
# use dioxus::prelude::*;
# use libero::components::{Button, Fields, Flex, Form, TextField};
# use libero::sx::sx;
# #[derive(Clone, PartialEq, Default, Fields)]
# struct Order { email: String }
# #[component]
# fn Checkout() -> Element {
#     let order = use_store(Order::default);
#     rsx! {
Form { value: order, onsubmit: move |_| {},
    TextField { label: "Email", name: Order::FIELDS.email(), required: true }
    // ... more fields
    Flex {
        direction: "row",
        gap: "sm",
        justify: "flex-end",
        sx: sx()
            .position("sticky")
            .bottom("0")
            .z_index("1")
            .padding("0.75em 0")
            .background("surface")
            .border_top("1px solid"),
        Button { variant: "outlined", color: "neutral", "Cancel" }
        Button { r#type: "submit", "Place order" }
    }
}
# } }
```

The same bar suits a `Stepper`: put it in the `panel`, with Back and Continue in
place of Cancel and the submit button, so every step has its buttons in the same
place. Disable Back on the first step instead of leaving it out.

```rust
# use dioxus::prelude::*;
# use libero::components::{Button, Flex, Options, Stepper};
# use libero::sx::sx;
# #[derive(Clone, PartialEq, Options)]
# enum Stage { Account, Review }
# #[component]
# fn Wizard() -> Element {
#     let mut stage = use_signal(|| Some(Stage::Account));
#     rsx! {
Stepper {
    value: stage(),
    aria_label: "Order steps",
    panel: move |s: Stage| rsx! {
        // ... the step's fields
        Flex {
            direction: "row",
            gap: "sm",
            justify: "space-between",
            sx: sx().position("sticky").bottom("0").z_index("1").padding("0.75em 0").background("surface"),
            Button {
                variant: "outlined",
                color: "neutral",
                disabled: s == Stage::Account,
                onclick: move |_| stage.set(Some(Stage::Account)),
                "Back"
            }
            Button { onclick: move |_| stage.set(Some(Stage::Review)), "Continue" }
        }
    },
}
# } }
```

## Props

### `Form`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Store<V>` | - | The whole form's value. `validate` checks it, and fields named by a path read and write it. `V` is inferred from it. A form with no `value` and no rules needs `Form::<()>`. |
| `validate` | `Validators<V>` | - | Rules over `value`, one or an array. A rule with `.on(..)` shows its status on each field it names, and runs only while each of them holds a parsed value or is empty and optional. |
| `onsubmit` | `Submit<V>` | - | Fires on a submit with no errors: a closure over the `FormEvent` or an `EventHandler<FormEvent>`, optional too, `Submit::parse(parse, handler)`, or the async `Submit::parse_async(parse, handler)` and `Submit::task(handler)`. Without an `action`, the browser's own submit is cancelled. A `javascript:`, `data:` or `vbscript:` `action` is dropped with a debug warning, so it counts as none. |
| `summary_title` | `String` | - | A heading over the error summary. |
| `summary` | `SummaryMode` | `Visible` | `Visible`: a blocked submit or `set_errors` shows a summary of every error above the fields and focuses it. `Hidden`: no summary list; focus goes to the first field that shows its error, and a visually hidden status says how many errors there are (`form.error_count`). Errors no field shows, from `FormErrors::form` or a form rule without `.on(..)`, still show in the summary, which then takes focus when no field does. |
| `form` | `FormHandle` | - | Controls the form from outside, made with `use_form()`. Taken once, on mount. Without it the form makes its own, which `use_form_context()` returns inside the form. |
| `children` | `Element` | required | The fields, fieldsets and buttons. |

`Form` also takes the `<form>` HTML attributes (`action`, `method`, ...) and,
like every component, the shared props `sx`, `class`, `style`, `states`, and
any extra HTML attributes.

### `FormHandle`

| Method | Returns | Description |
|---|---|---|
| `validate()` | `bool` | Checks like a submit without calling `onsubmit`. Every status shows, and with an error the summary appears and takes focus. `true` when nothing is an error. |
| `submit()` | `Result<(), PlatformError>` | Submits as the submit button would. `Unsupported` once the form is gone. |
| `reset()` | `()` | Puts the value back to `V::default()` and clears touched fields, the submit and the summary; a `type="reset"` button clears those too. A field with its own `value` and handler keeps what it shows. On the desktop WebView it cannot clear a control that is not bound to the value. Under Blitz it clears unbound text fields, not checkboxes or selects. |
| `is_valid()` | `bool` | Whether nothing is an error, shown or not. It follows changes, so it can drive other UI. |
| `is_submitting()` | `bool` | Whether an async handler (`Submit::parse_async`, `Submit::task`) is running. It follows changes, so it can set a submit button's `loading`. Meanwhile a submit is ignored and the form has `aria-busy="true"`. |
| `clear_summary()` | `()` | Hides the summary and resets nothing. |
| `set_errors(errors)` | `FormErrors` | Shows each error on the field with its path, as an error `status` would, lists it in the summary and focuses the summary: a server's answer. Replaces the errors set before. A field's error clears when it is edited, all of them on the next submit or reset. An error on a path no field carries, or from `FormErrors::form`, gets an unlinked summary line. |
| `clear_errors()` | `()` | Drops the errors `set_errors` set. |

### Every field

| Prop | Type | Default | Description |
|---|---|---|---|
| `required` | `bool` | `false` | Marks the label. Inside a `Form`, an empty field fails the submit with `form.required` unless its own rules fail first: `form.required_check` for an unchecked `Checkbox` or `Switch`, `form.required_select` for a select, `form.required_option` for a `RadioGroup` or `SegmentedControl`. A read-only field never fails it. A slider and a `ColorField` always hold a value, and a lone `Radio` leaves it to its `RadioGroup`. A field with a handler but no `value` or binding counts as filled: pass its `value` or a `name` path. |
| `validate` | `Validators<T>` | - | Rules over the field's own value, one or an array. Shown once the field loses focus or its form is submitted. |
| `name` | `FieldName<T>` | - | What the field posts as, and how rules address it. A path from `#[derive(Fields)]` also binds the field to the form's value, unless the field has a handler of its own. `T` is the field's value type. |

### Domain values

| Method | Type | Description |
|---|---|---|
| `parse(parse, handler)` | `Submit<V>` | `Submit::parse(parse, handler)`, for `onsubmit`: after the field rules pass, `parse: Fn(&V) -> Result<C, FormErrors>` builds a command from `value` and `handler` gets it. An `Err` puts each error on its field and in the summary, as `set_errors` does, and `handler` doesn't run. Hold checks in `validate` for live feedback; `parse` builds the command and reports what only the submit can tell. |
| `parse_async(parse, handler)` | `Submit<V>` | `Submit::parse` with an async `handler: FnMut(C) -> impl Future<Output = Result<(), FormErrors>>`, such as a request. It runs in the form's scope and ends with it. While it runs, `is_submitting()` is `true` and a submit is ignored; an `Err` lands as `set_errors` does. `Submit::task(handler)` is the same over the `FormEvent`, without `parse`. |
| `.map(format, parse)` | `Mapped<T, W, E>` | On a path to `Option<T>`: binds a field that edits `W` (text, a date, an option) to the domain type `T`. `format` shows a stored `T`, `parse` reads `W` back and never sees an empty one (blank text, `None`, `[]`); text reaches it untrimmed. An `Err` shows its words, resets the value to `None` and blocks the submit, while the text stays, also across an unmount. A focused field keeps the text as typed: `format` runs on blur and when the value changes elsewhere. `T` must be `Clone + PartialEq`. `Rc<T>` is no way out: it compares only when `T: PartialEq`. Wrap a type with neither, such as a secret, in `ByAddress<T>`. |
| `.map(.., ByAddress::new)` | `ByAddress<T>` | Makes a `T` with no `Clone` or `PartialEq`, such as a password type, fit for a form value: clones share it, two compare equal only when they are the same one, and it derefs to `T`. A field holds `Option<ByAddress<T>>`, and its `.map` parse wraps the value with `ByAddress::new`. |
| `.parsed()` | `Mapped<T, String, T::Err>` | `.map` through `T`'s `Display` and `FromStr`, for a text field. |
| `.message(fn)` | `fn(&E, &Localization) -> String` | Words every error of type `E` on the field, the parse error and `checked` rules alike. Needed when `E` has no `FieldError`, such as a domain crate's error: without it the name does not compile. |
| `.rules(..)` | `Validator<T> or Checked<T, E>` | Rules over `T`, run only on a parsed value. The field's own `validate` runs first, over `W`, before the parse; the first error wins. Adds to earlier calls, so a mixed list is two calls. |
| `checked(f)` | `Checked<V, E>` | A rule from `Fn(&V) -> Result<(), E>`: the field shows `E`'s words. `.warning()` shows it as a warning, `.message(fn)` words an `E` with no `FieldError`. Fits any `validate` too. |
| `impl FieldError` | `trait` | `fn message(&self, &Localization) -> String`: an error's words in the active language, worded again on a switch. Implemented for `Fn(&Localization) -> String`, `String` and `&'static str`; there is no `Display` fallback. |
| `.at(path, error)` | `FormErrors` | Errors by field path for `set_errors` and `parse`: `FormErrors::new().at(path, error)` with a `FieldError`, or `.at_with(path, error, fn)` worded by a `fn(&E, &Localization) -> String`. `path` is a `FieldPath`, a `FieldName` or its spelling. `.form(error)` is an error on the whole form, such as wrong credentials: an unlinked summary line, shown with `SummaryMode::Hidden` too, cleared by the next submit, a reset or `clear_errors`, not by an edit. |
| `.required(path)` | `Result<T, FormErrors>` | On an `Option<T>`, with `RequiredValue` in scope: the value, else an error on `path` worded as an empty `required` field. Turns a missing value into a `FormErrors` entry with `?` instead of an `unwrap()`. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work. The fields take their own `parts`.

| Part | `data-slot` | Description |
|---|---|---|
| `FormPart::Summary` | `summary` | The error summary, an `Alert`, shown after a blocked submit. |
| `FormPart::SummaryTitle` | `title` | The summary's heading, with `summary_title`. |
| `FormPart::SummaryList` | `list` | The summary's list of errors. |

## Accessibility

### Libero handles

- A form becomes a `form` landmark only once it has a name. A form without a
  name is still valid.
- Each summary line starts with its field's label, or its `aria_label` when it
  has no text label.
- A `Fieldset` with an error `status` is one field in error: it blocks the
  submit and has a line named by its legend.
- A summary line of a rule with `.on(..)` links to the first field it names that
  is on the page; a rule on the whole form that names no field links to the
  form's first field.
- A rule shows once focus leaves its field, not on a move inside it, such as
  from one `PinField` cell to the next. The desktop WebView cannot tell the two
  apart, so there any move shows it.

### You must

- Name the form when the page holds more than one form, or when the form is the
  page's main task, such as a checkout. Pass `aria-labelledby` pointing at a
  visible heading, or `aria-label`.
- Join intro text through `aria-describedby`, if any.
- Put the field's name into its messages when its label is markup
  (`Caption::Node`): the summary cannot read markup, so the line has no name.

```rust
# use dioxus::prelude::*;
# use libero::components::{Fields, Form};
# #[derive(Clone, PartialEq, Default, Fields)]
# struct Order { email: String }
# #[component]
# fn Checkout() -> Element {
#     let order = use_store(Order::default);
#     rsx! {
h2 { id: "checkout-title", "Checkout" }
Form { "aria-labelledby": "checkout-title", value: order, /* .. */ }
# } }
```

### Example

A checkout, `Form { "aria-labelledby": "checkout-title", .. }` under a visible
"Checkout" heading: a screen reader lists it as the "Checkout" form landmark,
and each line of the error summary starts with its field's label.

## Theme defaults

| Field | Type | Description |
|---|---|---|
| `form.gap` | `&'static str` | Gap between the summary and each child, `16px`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-form-gap` | `FormDefaults::gap`. |

## Data attributes

The summary is `data-slot="summary"`. It is an [`Alert`](alert.md) with
`color: "error"`, so its tint, radius and padding are `AlertDefaults`'.
