# Fieldset

Crate: `libero`
Import: `use libero::components::{Fieldset, Fields, Rule};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/fieldset.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Several fields that form one value - an address, a date range - under one
`<legend>`, with a description, helper and status of its own, and rules over
the group's value. Building reusable parts around it is explained in
[Forms: Getting Started](form_getting_started.md).

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Fields, Fieldset, Rule, TextField};

#[derive(Clone, PartialEq, Default, Fields)]
struct Address {
    street: String,
    zip: String,
    city: String,
}

#[component]
fn AddressFieldset() -> Element {
    let address = use_store(Address::default);

    rsx! {
        Fieldset {
            label: "Delivery address",
            value: address,
            validate: [
                (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                    .error("A city needs its zip code.")
                    .on([Address::FIELDS.zip()]),
                (|a: &Address| !a.street.is_empty() || a.city.is_empty())
                    .warn("Without a street we can only deliver to a pickup point."),
            ],
            TextField {
                label: "Street",
                name: Address::FIELDS.street(),
            }
            TextField {
                label: "Zip code",
                name: Address::FIELDS.zip(),
            }
            TextField {
                label: "City",
                name: Address::FIELDS.city(),
            }
        }
    }
}
```

Give the group a `path`. Field names and rule paths inside are relative to it.
On its own a fieldset takes a `value` store instead.

```rust
Form {
    value: order,
    Fieldset {
        label: "Delivery address",
        path: Order::FIELDS.address(),               // the group's value is order.address
        validate: (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
            .error("A city needs its zip code.")
            .on([Address::FIELDS.zip()]),            // rules are rooted at Address
        TextField { label: "Zip code", name: Address::FIELDS.zip() }  // posts "address.zip"
    }
}
```

## Known limits

- A fieldset with no `value`, `path` or rules writes
  `Fieldset::<()> { .. }` so Rust can infer its type.

## Props

### `Fieldset`

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Caption` | - | The group's caption, rendered as its `<legend>`. |
| `description` | `Caption` | - | Under the label. |
| `helper` | `Caption` | - | Under the fields. |
| `status` | `FieldStatus` | `Valid` | The group's own status, under the fields. A bare `&str` is an error. |
| `value` | `Store<V>` | - | The group's own value, for a fieldset outside a `Form`. Inside one the value is the form's, at `path`. |
| `validate` | `Validators<V>` | - | Composite rules over `value` - one rule, or an array. Without `.on(..)` a status shows under the fields once any field in the group was touched. |
| `path` | `FieldName<V>` | - | Where the group sits in a `Form`'s value, e.g. `Order::FIELDS.address()`. Field names and rule paths inside are relative to it. |
| `disabled` | `bool` | `false` | Disables every field inside, nested fieldsets included - their look, their own controls such as a `Select` trigger, and every native control. A field's own `disabled: false` cannot re-enable it. |
| `children` | `Element` | - | The fields. |

It also takes the shared props `sx`, `class`, `states`, and any `fieldset`
attribute.

## Theme defaults

| Field | Type | Description |
|---|---|---|
| `fieldset.gap` | `&'static str` | Gap between the fields inside a group (`12px`). The legend and caption typography comes from `FieldDefaults`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-fieldset-gap` | `FieldsetDefaults::gap`. |

## Data attributes

`data-state` carries the status token when there is one. The captions are
`data-slot="description" | "helper" | "status"`.
