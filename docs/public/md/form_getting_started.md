# Forms: Getting Started

Crate: `libero`
Import: `use libero::components::{Form, Fieldset, Fields, FieldName, Rule, Validator, Validators};`
Index: [index.md](index.md) - every other component's markdown page
Description: How libero forms are meant to be built - specialized fields, composed parts, validators at each layer, typed paths from `#[derive(Fields)]` and binding.

libero's forms are built in layers. A field holds one value. A specialized
field is a field with a narrower contract. A composed part groups fields into
one value with a [Fieldset](fieldset.md), and a [Form](form.md) holds the whole
value in one store, validates it and submits it. Rules live at the layer that
can see the values they check, and typed paths from `#[derive(Fields)]` tie
every layer to your own structs.

## The layers

```text
TextField                      // a field: one value, its label, its own rules
  └ PasswordField, EmailField  // specialized: a field with a narrower contract
AddressFieldset                // composed: specialized fields grouped into one value
  └ Fieldset { path: .. }
OrderForm                      // the whole value: composed parts, submitted together
  └ Form { value: order }
```

The sections below build one order form out of all four layers.

## Specialization

Every field is a plain component with the same slots - `label`,
`description`, `helper`, `status`, `validate`, `name`. A specialized field
wraps one and fixes part of its contract. libero does this itself:
`PasswordField` is a `TextField` with a reveal button. Your domain fields are
built the same way - an email, an IBAN, a phone number - with their rules built
in, so no form repeats them.

```rust
use libero::components::{FieldName, Rule, TextField, Validators, is_email};

/// A text field that only accepts email addresses.
#[component]
fn EmailField(
    #[props(into)] label: String,
    #[props(default, into)] name: FieldName<String>,
    #[props(default, into)] validate: Validators<String>,
) -> Element {
    // The caller's rules first - a required field says so before it says the
    // format is wrong - then the rule that makes this an email field.
    let rules: Vec<_> = validate
        .iter()
        .cloned()
        .chain(std::iter::once(is_email.error("That is not an email address.")))
        .collect();
    rsx! {
        TextField { r#type: "email", label, name, validate: rules, placeholder: "you@example.com" }
    }
}
```

Forward `name` as a `FieldName<T>` and the specialized field binds to a form
like any other. Forward `validate` and a caller can still add rules on top of
the built-in ones.

## Composition

A part of a form that recurs - an address, a date range, a contact - becomes a
component around a `Fieldset`. It takes a `path` saying where its value sits,
names its fields relative to its own type, and carries the rules that concern
only its own fields. It knows nothing about the form it ends up in, so the same
part serves the shipping and the billing address.

```rust
use libero::components::{FieldName, Fields, Fieldset, Rule, TextField, not_empty};

#[derive(Clone, PartialEq, Default, Fields)]
struct Address {
    street: String,
    zip: String,
    city: String,
}

/// The address block, for any form whose value holds an `Address` somewhere.
#[component]
fn AddressFieldset(#[props(into)] label: String, #[props(into)] path: FieldName<Address>) -> Element {
    rsx! {
        Fieldset {
            label,
            path,
            validate: (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                .error("A city needs its zip code.")
                .on([Address::FIELDS.zip()]),
            // Paths relative to the group: they work wherever the group sits.
            TextField { label: "Street", name: Address::FIELDS.street(), validate: not_empty.error("Enter a street.") }
            TextField { label: "Zip code", name: Address::FIELDS.zip() }
            TextField { label: "City", name: Address::FIELDS.city() }
        }
    }
}
```

The form then only arranges parts, and adds what only the whole value can
decide:

```rust
use libero::components::{Button, Checkbox, Fields, Form, Rule, Text, not_empty};

#[derive(Clone, PartialEq, Default, Fields)]
struct Order {
    email: String,
    #[fields(nested)]
    shipping: Address,
    same_billing: bool,
    #[fields(nested)]
    billing: Address,
}

#[component]
fn OrderForm() -> Element {
    let order = use_store(|| Order { same_billing: true, ..Default::default() });
    let mut placed = use_signal(|| false);

    rsx! {
        Form {
            value: order,
            // Only the whole order knows whether a billing address is needed.
            validate: (|o: &Order| o.same_billing || !o.billing.street.is_empty())
                .error("Enter a billing address, or bill to the shipping address.")
                .on([Order::FIELDS.billing().street()]),
            onsubmit: move |_| placed.set(true),
            EmailField { label: "Email", name: Order::FIELDS.email(), validate: not_empty.error("Enter your email.") }
            AddressFieldset { label: "Shipping address", path: Order::FIELDS.shipping() }
            Checkbox { label: "Bill to the shipping address", name: Order::FIELDS.same_billing() }
            if !order().same_billing {
                AddressFieldset { label: "Billing address", path: Order::FIELDS.billing() }
            }
            Button { r#type: "submit", "Place order" }
            if placed() {
                Text { "Order placed for {order().email}." }
            }
        }
    }
}
```

## Validators

A rule is any function from a value to a `bool` - `true` when the value is
fine. Nothing else to implement. `.error("..")` or `.warn("..")` gives it a
message and turns it into a `Validator<V>`; `.and(..)` and `.or(..)` combine
rules before the message. Your own rules are peers of the catalog
(`not_empty`, `min_length`, `is_email`, ..). The `Rule` trait has to be in
scope.

```rust
use libero::components::{Rule, Validator, max_length, not_empty};

// A rule is any `Fn(&V) -> bool`. A plain function is one:
fn no_spaces(value: &String) -> bool {
    !value.contains(' ')
}

// A rule with a parameter is a function returning one - the catalog's `min_length(8)` shape:
fn starts_with(prefix: &'static str) -> impl Fn(&String) -> bool {
    move |value| value.starts_with(prefix)
}

// A message turns a rule into a `Validator`. `.error` blocks a submit, `.warn` never does.
let username: Validator<String> = no_spaces.error("No spaces, please.");
let coupon = starts_with("LIB-").and(max_length(12)).warn("That does not look like our coupon.");

// A closure works too, and composite rules are closures over the whole value.
let adult = (|age: &Option<u8>| age.is_some_and(|age| age >= 18)).error("You must be 18.");

// A reusable rule for your own domain is just a function returning a `Validator`:
fn required(field: &str) -> Validator<String> {
    not_empty.error(format!("Enter your {field}."))
}
```

The catalog - rules for the common cases, over whichever value types they make
sense for:

| Rule | Over | Holds when |
|---|---|---|
| `not_empty` | `String`, `Option<T>`, `Vec<T>`, `bool` | not empty; whitespace counts as empty, `false` is empty |
| `min_length(n)` / `max_length(n)` | `String` | at least / at most `n` characters, not bytes |
| `min(x)` / `max(x)` | any `PartialOrd` | bounds included |
| `is_email` | `String` | one `@`, something before it, a dotted domain after - a shape check |

`Rule` is implemented for every `Fn(&V) -> bool` and has to be in scope for
`.error`, `.warn`, `.and` and `.or`.

Every layer takes `validate` - one validator, or an array - typed over the
value that layer can see:

| Layer | Checks | Example |
|---|---|---|
| Field | its own value | `TextField { validate: not_empty.error("..") }` |
| Specialized field | the rules that define it, built in; callers can add more | `EmailField` always checks `is_email` |
| Fieldset | its fields against each other; `.on([..])` names where the problem shows, otherwise it shows under the group | a city needs a zip |
| Form | what spans parts; without `.on([..])` it shows only in the error summary | a billing address unless billing goes to the shipping address |

Put a rule on the lowest layer that sees every value it reads. A field's rules
show once it loses focus, a composite problem once every field it names was
touched, and a submit reveals everything: an error cancels it and focuses a
summary of every problem. An explicit `status`, such as a server's answer,
shows at once, and the worse of it and the rules wins.

## Fields and paths

`#[derive(Fields)]` gives a struct a `FIELDS` constant with one method per
field. Each returns a `FieldPath<Root, T>`: where a `T` sits inside a `Root`,
how to reach it, and its spelling. `#[fields(nested)]` continues into a field
whose type derives `Fields` too, and the spelling becomes a dotted name like
`shipping.zip`.

```rust
#[derive(Clone, PartialEq, Default, Fields)]
struct Order {
    email: String,
    #[fields(nested)] // `Address` derives `Fields` too
    shipping: Address,
}

Order::FIELDS.email()               // FieldPath<Order, String>   posts as "email"
Order::FIELDS.shipping().zip()      // FieldPath<Order, String>   posts as "shipping.zip"
Order::FIELDS.shipping().path()     // FieldPath<Order, Address>  "shipping"
Order::FIELDS.shiping().zip()       // does not compile
path!(Order => shipping.zip)        // the same path, for a type that cannot derive

TextField { name: Order::FIELDS.email() }         // binds and posts
TextField { name: Order::FIELDS.same_billing() }  // does not compile: a TextField holds a String
rule.on([Order::FIELDS.email()])                  // a composite rule over `Order` names a field
```

One path does three jobs, so they cannot drift apart: it is the name the field
posts under, the name a composite rule addresses, and the place the field reads
and writes. A misspelled field is a compile error, not a rule that silently
never shows. A path to a `bool` on a `TextField` is a compile error too,
because a field's `name` is typed by the value it holds. `.on(..)` only takes
paths rooted at the rule's own value type.

## Binding

A `Form` takes your value as a `Store`, made with `use_store`. A field inside it named by a path
reads its value from that store and writes every change back - no `value` and
`oninput` pair per field. Typing re-renders only the field that changed, plus
whatever reads the whole value, such as the form's own rules. A `Fieldset` with a `path` moves everything inside it
one level down, which is what lets a composed part use paths rooted at its own
type.

```rust
let order = use_store(Order::default);

Form {
    value: order,                                  // the form holds the store
    TextField { name: Order::FIELDS.email() }      // reads and writes order.email
    Fieldset {
        path: Order::FIELDS.shipping(),            // everything inside is relative to order.shipping
        TextField { name: Address::FIELDS.zip() }  // reads and writes order.shipping.zip, posts "shipping.zip"
    }
    TextField {                                    // a handler of its own takes the field back
        name: Order::FIELDS.email(),
        value: draft(),
        oninput: move |next| draft.set(next),
    }
}
```

A field with a handler of its own is controlled by that handler, as outside a
form. A plain string `name` only posts. The path's root is checked when the
field renders: one rooted at a different type than the form's value warns in
debug builds and leaves the field unbound. Every bound field re-renders when
any part of the value changes.
