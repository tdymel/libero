# Fieldset

Crate: `libero`
Import: `use libero::components::{Fieldset, Fields, Rule};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/fieldset.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Several fields that form one value under a `<legend>`, with composite rules over that value that land on the fields they name.

Several fields that form one value - an address, a date range - grouped in a
`<fieldset>` under one `<legend>`, with a description, helper and status of its
own. Its `validate` rules run over the group's `value` and put a problem on the
fields they name. It works on its own or inside a [Form](form.md), which
documents rules, the catalog and typed paths.

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
    let mut address = use_signal(Address::default);

    rsx! {
        Fieldset {
            legend: "Delivery address",
            value: address(),
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
                value: address().street,
                oninput: move |next| address.write().street = next,
            }
            TextField {
                label: "Zip code",
                name: Address::FIELDS.zip(),
                value: address().zip,
                oninput: move |next| address.write().zip = next,
            }
            TextField {
                label: "City",
                name: Address::FIELDS.city(),
                value: address().city,
                oninput: move |next| address.write().city = next,
            }
        }
    }
}
```

A rule with `.on(..)` shows on the named fields once all of them were touched.
A rule without `.on(..)` shows in the group's status slot once any field in the
group was touched. Either shows at once after a submit of the surrounding
`Form`.

## Inside a Form

Give the group a `path`. Its rules keep using paths rooted at the group's own
type, and the fieldset puts its prefix in front of them - `Address::FIELDS.zip()`
reaches the field named `address.zip`. On its own a fieldset opens a scope of
its own and needs no `path`.

```rust
use libero::components::{Button, Fields, Fieldset, Form, Rule, TextField, not_empty};

#[derive(Clone, PartialEq, Default, Fields)]
struct Order {
    name: String,
    #[fields(nested)]
    address: Address,
}

rsx! {
    Form {
        value: order(),
        TextField {
            label: "Name",
            name: Order::FIELDS.name(),
            value: order().name,
            oninput: move |next| order.write().name = next,
            validate: not_empty.error("Enter your name."),
        }
        Fieldset {
            legend: "Delivery address",
            path: Order::FIELDS.address(),
            value: order().address,
            validate: (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                .error("A city needs its zip code.")
                .on([Address::FIELDS.zip()]),
            TextField {
                label: "Zip code",
                name: Order::FIELDS.address().zip(),
                value: order().address.zip,
                oninput: move |next| order.write().address.zip = next,
            }
        }
        Button { r#type: "submit", "Order" }
    }
}
```

## Accessibility

The legend names the group, so a screen reader announces it as focus enters.
The description, helper and status join the fieldset's `aria-describedby`.

## Known limits

- `disabled` is `<fieldset disabled>`, which reaches native controls only. A
  `Select` trigger inside a disabled fieldset still opens.
- A fieldset without rules currently writes `value: ()` so Rust can infer its
  type.

## Props

### `Fieldset`

| Prop | Type | Default | Description |
|---|---|---|---|
| `legend` | `Caption` | - | The group's caption, a `<legend>`. |
| `description` | `Caption` | - | Under the legend. |
| `helper` | `Caption` | - | Under the fields. |
| `status` | `FieldStatus` | `Valid` | The group's own status, under the fields. A bare `&str` is an error. |
| `value` | `V` | - | The group's value, which `validate` checks. |
| `validate` | `Validators<V>` | - | Composite rules over `value` - one rule, or an array. |
| `path` | `String` | - | Where `value` sits inside a `Form`, e.g. `Order::FIELDS.address()`. |
| `disabled` | `bool` | `false` | `<fieldset disabled>`. |
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
