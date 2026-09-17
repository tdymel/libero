# use_form_context

Crate: `libero`
Import: `use libero::components::use_form_context;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/handle.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The handle of the Form it is called inside, so a part of a form reaches it without a prop.

`use_form_context() -> Option<FormHandle>` returns the handle of the `Form` it
is called inside, or `None` outside one. A part of a form reaches the form
without a prop. [Form](form.md) documents the handle's methods.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Fields, Flex, Form, Rule, TextField, not_empty, use_form_context};

#[derive(Clone, PartialEq, Default, Fields)]
struct Contact {
    name: String,
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

#[component]
fn ContactForm() -> Element {
    let value = use_store(Contact::default);

    rsx! {
        Form { value,
            TextField { label: "Name", name: Contact::FIELDS.name(), validate: not_empty.error("Enter your name.") }
            Flex { direction: "row", gap: "sm",
                Button { r#type: "submit", "Send" }
                CheckButton {}
            }
        }
    }
}
```

It is the same handle [use_form](use_form.md) makes. A form given none makes
its own, and this hook reaches that one.

## API

```rust,ignore
pub fn use_form_context() -> Option<FormHandle>
```
