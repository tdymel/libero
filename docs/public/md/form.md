# Form

Crate: `libero`
Import: `use libero::components::{Form, Fields, Rule};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/form.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A `<form>` that validates on submit - plain `Fn(&V) -> bool` rules, typed field paths from `#[derive(Fields)]`, and a focused error summary.

A `<form novalidate>` that holds the whole value in one signal, runs rules
across its fields, and validates on submit. How fields, fieldsets, rules, typed
paths and binding fit together is explained in
[Forms: Getting Started](form_getting_started.md).

## Usage

```rust
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
    let signup = use_signal(Signup::default);
    let mut sent = use_signal(|| false);

    rsx! {
        Form {
            value: signup,
            // Only the whole signup sees the email and the password together.
            validate: (|s: &Signup| s.email.is_empty() || !s.password.value.contains(&s.email))
                .warn("Your password contains your email address.")
                .on([Signup::FIELDS.password().value()]),
            onsubmit: move |_| sent.set(true),
            // A specialized field and a composed part - built in Getting Started.
            EmailField { label: "Email", name: Signup::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            NewPasswordFieldset { path: Signup::FIELDS.password() }
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                validate: not_empty.error("Accept the terms to continue."),
            }
            Button { r#type: "submit", "Create account" }
            if sent() {
                Text { "Account created." }
            }
        }
    }
}
```

## Submitting

A submit reveals every status. With any error it is cancelled, and a summary of
every problem appears above the fields and takes focus. Without errors
`onsubmit` fires; the browser's own submit is cancelled unless the form has an
`action`. Warnings never block. The summary is a snapshot, refreshed on the
next submit.

## Accessibility

The status joins the field's `aria-describedby`, and an error sets
`aria-invalid`. Checking while someone types stays visual. The announcement
happens on submit: the summary is a `role="alert"` that takes focus, reads every
problem at once, and links each one to its field.

A form becomes a `form` landmark only once it has a name. Name it when the page
holds more than one form, or when the form is the page's main task, like a
checkout: pass `aria-labelledby` pointing at a visible heading, or
`aria-label`. Intro text can join through `aria-describedby`. A form without a
name is still valid.

```rust
h2 { id: "checkout-title", "Checkout" }
Form { "aria-labelledby": "checkout-title", value: order, /* .. */ }
```

## Known limits

- A `Form` needs a `value` or rules for Rust to infer its type. A form with
  neither currently writes `Form::<()> { .. }`.
- Every bound field re-renders on any change to the form's value.

## Props

### `Form`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Signal<V>` | - | The whole form's value, which `validate` checks and fields named by a path read and write. `V` is inferred from it. |
| `validate` | `Validators<V>` | - | Composite rules over `value` - one rule, or an array. |
| `onsubmit` | `EventHandler<FormEvent>` | - | Fires on a submit nothing blocks. |
| `summary_title` | `String` | - | A heading over the error summary. |
| `children` | `Element` | - | The fields, fieldsets and buttons. |

It also takes the shared props `sx`, `class`, `states`, and any `form`
attribute - `action` and `method` among them.

### Every field

| Prop | Type | Default | Description |
|---|---|---|---|
| `validate` | `Validators<V>` | - | Rules over the field's own value - one rule, or an array. |
| `name` | `FieldName<T>` | - | What the field posts as and composite rules address it by. Built from a path, which also binds the field to the form's value unless it has a handler, or from a string. `T` is the field's value type. |

## Theme defaults

| Field | Type | Description |
|---|---|---|
| `form.gap` | `&'static str` | Gap between the summary and each child (`16px`). |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-form-gap` | `FormDefaults::gap`. |

## Data attributes

The summary is `data-slot="summary"`.
