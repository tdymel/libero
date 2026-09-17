# use_form

Crate: `libero`
Import: `use libero::components::use_form;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/handle.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A handle that controls a Form from the component that renders it.

`use_form() -> FormHandle` controls a `Form` from the component that renders
it: whether it is valid, a check, a submit and a reset. Pass it as the form's
`form`. [Form](form.md) documents validation and the error summary.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Fields, Flex, Form, Rule, Text, TextField, not_empty, use_form};

#[derive(Clone, PartialEq, Default, Fields)]
struct Newsletter {
    email: String,
}

#[component]
fn NewsletterForm() -> Element {
    let value = use_store(Newsletter::default);
    let form = use_form();

    rsx! {
        Form { form, value,
            TextField { label: "Email", name: Newsletter::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            Text { if form.is_valid() { "Ready to send." } else { "Not ready yet." } }
            Flex { direction: "row", gap: "sm",
                Button { r#type: "submit", "Subscribe" }
                Button { variant: "outlined", onclick: move |_| form.reset(), "Clear" }
            }
        }
    }
}
```

`is_valid()` follows every change, so it can drive other UI. `reset()` puts
the value back to its default with nothing touched.

## API

```rust,ignore
pub fn use_form() -> FormHandle
```

| Method | Returns | Description |
|---|---|---|
| `validate()` | `bool` | Checks like a submit without calling `onsubmit`. `true` when nothing is an error. |
| `submit()` | `Result<(), PlatformError>` | Submits as the submit button would. |
| `reset()` | `()` | The value back to `V::default()`, nothing touched, no summary. |
| `is_valid()` | `bool` | Whether nothing is an error, shown or not. |
| `clear_summary()` | `()` | Hides the error summary. Resets nothing. |

`Copy`. Inside the form, [use_form_context](use_form_context.md) returns the
same handle.
