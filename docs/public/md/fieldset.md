# Fieldset

Crate: `libero`
Import: `use libero::components::{Fieldset, Fields, Rule};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/fieldset.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Several fields that form one value under a `<legend>`, with rules over that value that land on the fields they name.

Several fields that form one value, such as an address or a date range, under
one `<legend>`. The group has its own description, helper and status, and
rules over its value. Inside a `Form`, give it a `path`. On its own, give it a
`value` store. The Form [getting started](form_getting_started.md) page shows
how to build reusable parts around it.

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

Inside a `Form`, the group takes a `path`. Field names and rule paths inside
are relative to it.

```rust,ignore
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

## Props

### `Fieldset`

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Caption` | - | The group's caption, rendered as its `<legend>`. |
| `description` | `Caption` | - | Under the label. |
| `helper` | `Caption` | - | Under the fields. |
| `status` | `FieldStatus` | `Valid` | The group's own status, under the fields. A bare `&str` is an error. |
| `value` | `Store<V>` | - | The group's own value, for a fieldset outside a `Form`. A fieldset with no `value`, `path` or rules needs `Fieldset::<()>` so Rust can infer its type. |
| `validate` | `Validators<V>` | - | Rules over the group's value, one or an array. A rule with `.on(..)` shows on the fields it names. One without shows under the fields once any field in the group was touched. |
| `path` | `FieldName<V>` | - | Where the group sits inside a `Form`'s value, such as `Order::FIELDS.address()`. The names of the fields inside and the paths in `validate` are relative to it. |
| `disabled` | `bool` | `false` | Disables every field inside, nested fieldsets included. A field's own `disabled: false` cannot re-enable it. |
| `children` | `Element` | required | The fields. |

`Fieldset` also takes the `<fieldset>` HTML attributes and, like every
component, the shared props `sx`, `class`, `style`, `states`, and any extra
HTML attributes.

## Theme defaults

| Field | Type | Description |
|---|---|---|
| `fieldset.gap` | `&'static str` | Gap between the fields inside a group, `12px`. The legend and caption type comes from `FieldDefaults`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-fieldset-gap` | `FieldsetDefaults::gap`. |

## Data attributes

`data-state` carries the status token when there is one. The captions carry
`data-slot="description"`, `"helper"` or `"status"`.
