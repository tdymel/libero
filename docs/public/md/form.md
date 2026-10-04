# Form

Crate: `libero`
Import: `use libero::components::{Form, Fields, Rule, use_form, use_form_context};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/form.rs>
Index: [index.md](index.md) lists every other page
Description: A `<form>` that validates on submit, with plain `Fn(&V) -> bool` rules, typed field paths from `#[derive(Fields)]` and a focused error summary.

A `<form novalidate>` that holds the whole value in one store, runs rules
across its fields and validates on submit. The Form
[getting started](form_getting_started.md) page shows how fields, fieldsets,
rules and paths fit together.

## Usage

```rust,ignore
use dioxus::prelude::*;
use libero::components::{Button, Checkbox, Fields, Form, Rule, Text, not_empty};

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    email: String,
    #[fields(nested)]
    password: NewPassword,
    terms: bool,
}

#[component]
fn SignupForm() -> Element {
    let signup = use_store(Signup::default);
    let mut sent = use_signal(|| false);

    rsx! {
        Form {
            value: signup,
            // Only the whole signup sees the email and the password together.
            validate: (|s: &Signup| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Signup::FIELDS.password().value()]),
            onsubmit: move |_| sent.set(true),
            // A specialized field and a composed part, built in Getting started.
            EmailField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            NewPasswordFieldset { path: Signup::FIELDS.password() }
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                validate: not_empty.error("Accept the terms to continue."),
            }
            Button { r#type: "submit", "Create account" }
            // Mounted before the submit, so a screen reader hears the text it gains.
            Text { role: "status",
                if sent() { "Account created." }
            }
        }
    }
}
```

`use_form()` makes a handle to pass as `form`. Inside a form,
`use_form_context()` returns the same handle.

```rust,ignore
#[component]
fn TermsForm() -> Element {
    let terms = use_store(Terms::default);
    let form = use_form();

    rsx! {
        Form {
            form,
            value: terms,
            EmailField { label: "Email", name: Terms::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            Checkbox { label: "I accept the terms", name: Terms::FIELDS.accepted(), validate: not_empty.error("Accept the terms to continue.") }
            Text { role: "status", if form.is_valid() { "Ready to send." } else { "Not ready yet." } }
            Flex { gap: "sm",
                Button { r#type: "submit", "Send" }
                CheckButton {}
                Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
            }
        }
    }
}

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
```

## Conditional fields

A field shown only for some values is a plain `if` around it. Its rules leave
with it, so a hidden field never blocks a submit. Its value stays in the store,
and a rule on the whole form still runs, so check the condition there too.

```rust,ignore
Checkbox { label: "Sign up as a company", name: Signup::FIELDS.company() }
// Rendered only while ticked, so its rules leave with it.
if signup.read().company {
    TextField {
        label: "Company name",
        name: Signup::FIELDS.company_name(),
        validate: not_empty.error("Enter the company name."),
    }
}
```

## Submit

A submit shows every status. With an error, it is cancelled, and a summary of
every problem appears above the fields and takes focus. Warnings never block.
The summary keeps the problems of that submit. A line leaves once it is fixed,
and new ones wait for the next submit.

## Props

### `Form`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Store<V>` | - | The whole form's value. `validate` checks it, and fields named by a path read and write it. `V` is inferred from it. A form with no `value` and no rules needs `Form::<()>`. |
| `validate` | `Validators<V>` | - | Rules over `value`, one or an array. A rule with `.on(..)` shows its status on each field it names. |
| `onsubmit` | `EventHandler<FormEvent>` | - | Fires on a submit with no errors. Without an `action`, the browser's own submit is cancelled. |
| `summary_title` | `String` | - | A heading over the error summary. |
| `form` | `FormHandle` | - | Controls the form from outside, made with `use_form()`. Without it the form makes its own, which `use_form_context()` returns inside the form. |
| `children` | `Element` | required | The fields, fieldsets and buttons. |

`Form` also takes the `<form>` HTML attributes (`action`, `method`, ...) and,
like every component, the shared props `sx`, `class`, `style`, `states`, and
any extra HTML attributes.

### `FormHandle`

| Method | Returns | Description |
|---|---|---|
| `validate()` | `bool` | Checks like a submit without calling `onsubmit`. Every status shows, and with an error the summary appears and takes focus. `true` when nothing is an error. |
| `submit()` | `Result<(), PlatformError>` | Submits as the submit button would. `Unsupported` once the form is gone. |
| `reset()` | `()` | Puts the value back to `V::default()` and clears touched fields, the submit and the summary. A field with its own `value` and handler keeps what it shows. On the desktop WebView it cannot clear a control that is not bound to the value. Under Blitz it clears unbound text fields, not checkboxes or selects. |
| `is_valid()` | `bool` | Whether nothing is an error, shown or not. It follows changes, so it can drive other UI. |
| `clear_summary()` | `()` | Hides the summary and resets nothing. |

### Every field

| Prop | Type | Default | Description |
|---|---|---|---|
| `validate` | `Validators<T>` | - | Rules over the field's own value, one or an array. Shown once the field loses focus or its form is submitted. |
| `name` | `FieldName<T>` | - | What the field posts as, and how rules address it. A path from `#[derive(Fields)]` also binds the field to the form's value, unless the field has a handler of its own. `T` is the field's value type. |

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

```rust,ignore
h2 { id: "checkout-title", "Checkout" }
Form { "aria-labelledby": "checkout-title", value: order, /* .. */ }
```

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
