# Form

Crate: `libero`
Import: `use libero::components::{Form, Fields, Rule, not_empty, min_length, max_length, min, max, is_email};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/form.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A `<form>` that validates on submit - plain `Fn(&V) -> bool` rules, typed field paths from `#[derive(Fields)]`, and a focused error summary.

A `<form novalidate>` that validates on submit. Every field inside checks its
own value through `validate`; the form checks rules across several fields and
puts a problem on the fields it names. A submit with an error is cancelled,
every field shows its status, and a summary of every problem takes focus.

A rule is a plain function from a value to a `bool`, given a message with
`.error(..)` or `.warn(..)`. Groups of fields have their own component:
[Fieldset](fieldset.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{
    Button, Checkbox, Fields, Form, PasswordField, Rule, Text, TextField, is_email, min_length,
    not_empty,
};

#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    email: String,
    password: String,
    confirm: String,
    terms: bool,
}

#[component]
fn SignupForm() -> Element {
    let mut signup = use_signal(Signup::default);
    let mut sent = use_signal(|| false);

    rsx! {
        Form {
            value: signup(),
            validate: (|s: &Signup| s.password == s.confirm)
                .error("The passwords differ.")
                .on([Signup::FIELDS.password(), Signup::FIELDS.confirm()]),
            onsubmit: move |_| sent.set(true),
            TextField {
                label: "Email",
                name: Signup::FIELDS.email(),
                value: signup().email,
                oninput: move |next| signup.write().email = next,
                validate: [
                    not_empty.error("Enter your email."),
                    is_email.error("That is not an email address."),
                ],
            }
            PasswordField {
                label: "Password",
                name: Signup::FIELDS.password(),
                value: signup().password,
                oninput: move |next| signup.write().password = next,
                validate: min_length(8).error("Use at least 8 characters."),
            }
            PasswordField {
                label: "Repeat password",
                name: Signup::FIELDS.confirm(),
                value: signup().confirm,
                oninput: move |next| signup.write().confirm = next,
            }
            Checkbox {
                label: "I accept the terms",
                name: Signup::FIELDS.terms(),
                checked: signup().terms,
                onchange: move |next| signup.write().terms = next,
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

`validate` takes one rule or an array of them - on a field, a `Fieldset` and a
`Form` alike.

## Field rules

Every field takes `validate`, typed over its own value: `String` for text
fields, `bool` for `Checkbox` and `Switch`, `Option<T>` for `Select`,
`RadioGroup`, `NativeSelect` and `NumberField`, `Vec<T>` for `MultiSelect`,
`Files`, `ColorCode`, and the slider's own `V` or `(V, V)`. A field validates
itself with or without a `Form` around it.

```rust
use libero::components::{Rule, TextField, max_length, min_length, not_empty};

// A rule is any `Fn(&V) -> bool` - your own function is a peer of the catalog.
fn not_admin(name: &String) -> bool {
    name != "admin"
}

rsx! {
    TextField {
        label: "Username",
        value: username(),
        oninput: move |next| username.set(next),
        validate: [
            not_empty.error("Pick a username."),
            min_length(3).and(not_admin).error("At least 3 characters, and not \"admin\"."),
            max_length(12).warn("Long names get cut off in lists."),
        ],
    }
}
```

The rules run in order. The first error wins; a warning shows only when nothing
errors.

## The catalog

| Rule | Over | Holds when |
|---|---|---|
| `not_empty` | `String`, `Option<T>`, `Vec<T>`, `bool` | not empty; whitespace counts as empty, `false` is empty |
| `min_length(n)` / `max_length(n)` | `String` | at least / at most `n` characters, not bytes |
| `min(x)` / `max(x)` | any `PartialOrd` | bounds included |
| `is_email` | `String` | one `@`, something before it, a dotted domain after - a shape check |

`Rule` is implemented for every `Fn(&V) -> bool` and has to be in scope for
`.error`, `.warn`, `.and` and `.or`.

## Composite rules and typed paths

`.on(..)` puts a form rule's status on every field it names, matched by `name`.
A rule without `.on(..)` shows only in the summary. `.on(..)` only accepts paths
rooted at the rule's own value type, so a `Form<Signup>` rule cannot name an
`Address` path by mistake.

```rust
use libero::components::{Fields, path};

#[derive(Fields)]
struct Order {
    name: String,
    #[fields(nested)]
    address: Address, // derives Fields too
}

Order::FIELDS.name()            // FieldPath<Order, String>, posts as "name"
Order::FIELDS.address().zip()   // FieldPath<Order, String>, posts as "address.zip"
Order::FIELDS.address().path()  // FieldPath<Order, Address>, "address"

// For a type that cannot derive:
path!(Order => address.zip)     // FieldPath<Order, String>
```

A path is also the field's `name`, so what a rule names and what the form posts
are one string. A typo does not compile. In debug builds a submit warns about a
path no field inside the form carries - almost always a field missing its
`name`.

## When a status shows

- A field's rules wait until it loses focus for the first time, then follow
  every change. The field tracks this itself - no `onblur` wiring.
- A composite problem waits until every field it names was touched.
- A submit reveals everything.
- An explicit `status` - a server's answer - never waits. When it and the rules
  disagree, the worse one shows: error beats warning beats valid.

## Submitting

On submit the form marks every field touched and checks for errors. With any
error it cancels the submit, fills the error summary and focuses it. Without
errors it calls `onsubmit`; the browser's own submit is cancelled unless the
form has an `action`, in which case it posts natively. Warnings never block.

The summary is a snapshot of the failed submit, refreshed on the next one.

## Accessibility

The status joins the field's `aria-describedby`, and an error sets
`aria-invalid`. Checking while someone types stays visual. The announcement
happens on submit: the summary is a `role="alert"` that takes focus, reads every
problem at once, and links each one to its field.

## Known limits

- A `Form` needs a `value` for Rust to infer its type. A form without rules
  currently writes `value: ()`.

## Props

### `Form`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `V` | - | The whole form's value, which `validate` checks. `V` is inferred from it. |
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
| `name` | `FieldPath` or `String` | - | What the field posts as, and the path composite rules address it by. |

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
