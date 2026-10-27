# Form getting started

Crate: `libero`
Import: `use libero::components::{Form, Fieldset, Fields, FieldName, Rule, Validator, Validators};`
Index: [index.md](index.md) lists every other page
Description: How to build libero forms, with specialized fields, composed parts, validators at each layer, typed paths from `#[derive(Fields)]` and binding.

libero's forms are built in layers. A field holds one value. A specialized
field is a field with a narrower contract. A composed part groups fields into
one value with a [Fieldset](fieldset.md), and a [Form](form.md) holds the whole
value in one store, validates it and submits it.

## The layers

Rules live at the layer that can see the values they check, and typed paths
from `#[derive(Fields)]` tie every layer to your own structs.

```text
TextField                      // a field, one value with its label and rules
  └ PasswordField, EmailField  // specialized fields, each with a narrower contract
AddressFieldset                // a composed part, fields grouped into one value
  └ Fieldset { path: .. }
OrderForm                      // the whole value, its parts submitted together
  └ Form { value: order }
```

The sections below build one order form out of all four layers. The web page
runs it, so you can submit it empty and then fill it in.

## Specialization

Every field is a plain component with the same props, `label`,
`description`, `helper`, `status`, `validate` and `name`. A specialized field
wraps one and fixes part of its contract. `PasswordField` is one, a
`TextField` with a reveal button. Build your own domain fields the same way,
such as an email, an IBAN or a phone number. Their rules are built in, so no
form repeats them.

```rust
use dioxus::prelude::*;
use libero::components::{FieldName, Rule, TextField, Validators, is_email};

/// A text field that only accepts email addresses.
#[component]
fn EmailField(
    #[props(into)] label: String,
    #[props(default, into)] name: FieldName<String>,
    #[props(default, into)] validate: Validators<String>,
) -> Element {
    // The caller's rules go first, so a missing email shows before a malformed
    // one. Then the rule that makes this an email field.
    let rules = validate.with(is_email.error("That is not an email address."));
    rsx! {
        TextField { r#type: "email", autocomplete: "email", label, name, validate: rules, placeholder: "you@example.com" }
    }
}
```

Forward `name` as a `FieldName<T>` and the specialized field binds to a form
like any other. Forward `validate` and a caller can still add rules on top of
the built-in ones.

## Composition

A part of a form that recurs, such as an address, a date range or a contact,
becomes a component around a `Fieldset`. It takes a `path` saying where its
value sits, names its fields relative to its own type, and carries the rules
over its own fields. It knows nothing about the form around it, so the same
part serves the shipping and the billing address.

```rust
use dioxus::prelude::*;
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
            // Paths relative to the group work wherever the group sits.
            TextField { label: "Street", autocomplete: "street-address", name: Address::FIELDS.street(), validate: not_empty.error("Enter a street.") }
            TextField { label: "Zip code", autocomplete: "postal-code", name: Address::FIELDS.zip() }
            TextField { label: "City", autocomplete: "address-level2", name: Address::FIELDS.city() }
        }
    }
}
```

The form then arranges the parts and adds the rules only the whole value can
decide.

```rust
use dioxus::prelude::*;
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
            // Rendered only while unticked: ticked, the billing fields neither validate nor
            // post, though `order().billing` keeps what was typed.
            if !order().same_billing {
                AddressFieldset { label: "Billing address", path: Order::FIELDS.billing() }
            }
            Button { r#type: "submit", "Place order" }
            // Mounted before the submit, so a screen reader hears the text it gains.
            Text { role: "status",
                if placed() { "Order placed for {order().email}." }
            }
        }
    }
}
#
# use libero::components::{FieldName, Validators};
# #[derive(Clone, PartialEq, Default, Fields)]
# struct Address { street: String, zip: String, city: String }
# #[component]
# fn EmailField(#[props(into)] label: String, #[props(default, into)] name: FieldName<String>,
#     #[props(default, into)] validate: Validators<String>) -> Element { rsx! {} }
# #[component]
# fn AddressFieldset(#[props(into)] label: String, #[props(into)] path: FieldName<Address>) -> Element { rsx! {} }
```

## Domain field wrapper

A field for a domain type, such as an `Iban` from your own crate, edits text
and maps it with `.map(format, parse)` on a path to `Option<Iban>`. There are
two ways to wrap it. Either the caller maps and the wrapper takes
`#[props(into)] name: FieldName<String>`, so it fits any binding, or the
wrapper is generic over the form's value `R`, takes a
`FieldPath<R, Option<Iban>>` and maps inside, so no caller repeats the parse.

```rust
use dioxus::prelude::*;
use libero::components::{Button, FieldName, FieldPath, Fields, Form, Text, TextField};
use libero::localization::Localization;
use std::fmt;

/// A domain type from your own crate, which knows nothing about libero.
#[derive(Clone, PartialEq)]
struct Iban(String);

enum IbanError {
    TooShort,
    BadCountry,
}

impl Iban {
    fn parse(text: String) -> Result<Iban, IbanError> {
        let compact = text.split_whitespace().collect::<String>().to_uppercase();
        if compact.chars().count() < 15 {
            return Err(IbanError::TooShort);
        }
        match compact.chars().take(2).all(|c| c.is_ascii_alphabetic()) {
            true => Ok(Iban(compact)),
            false => Err(IbanError::BadCountry),
        }
    }
}

impl fmt::Display for Iban {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

fn iban_message(error: &IbanError, _: &Localization) -> String {
    match error {
        IbanError::TooShort => "An IBAN has at least 15 characters.".into(),
        IbanError::BadCountry => "An IBAN starts with a country code, such as DE.".into(),
    }
}

/// (a) The caller maps: the wrapper edits text and takes any name that converts.
#[component]
fn IbanField(#[props(into)] label: String, #[props(into)] name: FieldName<String>) -> Element {
    rsx! {
        TextField { label, name, autocomplete: "off", placeholder: "DE89 3704 0044 0532 0130 00" }
    }
}

/// (b) The wrapper maps: it takes a path to `Option<Iban>` in any form value `R`.
#[component]
fn IbanPathField<R: 'static>(#[props(into)] label: String, path: FieldPath<R, Option<Iban>>) -> Element {
    let name = path.map(Iban::to_string, Iban::parse).message(iban_message);
    rsx! {
        TextField { label, name, autocomplete: "off", placeholder: "DE89 3704 0044 0532 0130 00" }
    }
}

#[derive(Clone, PartialEq, Default, Fields)]
struct Payout {
    account: Option<Iban>,
    refunds: Option<Iban>,
}

#[component]
fn PayoutForm() -> Element {
    let payout = use_store(Payout::default);
    rsx! {
        Form { value: payout,
            IbanField {
                label: "Payout account",
                name: Payout::FIELDS.account().map(Iban::to_string, Iban::parse).message(iban_message),
            }
            IbanPathField { label: "Refund account", path: Payout::FIELDS.refunds() }
            Button { r#type: "submit", "Save" }
            Text { role: "status",
                if let Some(account) = payout().account { "Payouts go to {account}." }
            }
        }
    }
}
```

The first fits a field bound in several ways, the second a type mapped the
same way everywhere. The domain type must be `Clone + PartialEq`, as every
form value is.

## Validators

A rule is any function from a value to a `bool`, `true` when the value is
fine. `.error("..")` or `.warn("..")` gives it a message and turns it into a
`Validator<V>`. `.and(..)` and `.or(..)` combine rules before the message. Your
own rules work like the built-in ones, such as `not_empty`, `min_length` and
`is_email`. The `Rule` trait has to be in scope.

```rust
use libero::components::{Rule, Validator, max_length, not_empty};

// A rule is any `Fn(&V) -> bool`, so a plain function is one.
fn no_spaces(value: &String) -> bool {
    !value.contains(' ')
}

// A rule with a parameter is a function that returns one, like `min_length(8)`.
fn starts_with(prefix: &'static str) -> impl Fn(&String) -> bool {
    move |value| value.starts_with(prefix)
}

// A message turns a rule into a `Validator`. `.error` blocks a submit, `.warn` never does.
let username: Validator<String> = no_spaces.error("No spaces, please.");
let coupon = starts_with("LIB-").and(max_length(12)).warn("That does not look like our coupon.");

// A closure works too, and composite rules are closures over the whole value.
let adult = (|age: &Option<u8>| age.is_some_and(|age| age >= 18)).error("You must be 18.");

// A reusable rule for your own domain is a function that returns a `Validator`.
fn required(field: &str) -> Validator<String> {
    not_empty.error(format!("Enter your {field}."))
}
```

The built-in rules cover the common cases, over the value types they make
sense for.

| Rule | Over | Holds when |
|---|---|---|
| `not_empty` | `String`, `Option<T>`, `Vec<T>`, `bool` | not empty; whitespace counts as empty, `false` is empty |
| `min_length(n)` / `max_length(n)` | `String` | at least / at most `n` characters, not bytes |
| `min(x)` / `max(x)` | any `PartialOrd` | bounds included |
| `is_email` | `String` | one `@`, something before it and a dotted domain after it |

`rule.when_present()` lets an empty value pass, so an optional field checks
only what was typed: `is_email.when_present().error("..")`. It is opt-in, and
`not_empty` and every other rule still see the empty value. A specialized
field adds its own rules to the caller's with `validate.with(..)` or
`validate.extend([..])`. Rules that capture nothing still compare equal, so the
field does not re-render for them.

Every layer takes `validate`, one validator or an array, typed over the value
that layer can see.

| Layer | Checks | Example |
|---|---|---|
| Field | its own value | `TextField { validate: not_empty.error("..") }` |
| Specialized field | the rules that define it, built in. Callers can add more | `EmailField` always checks `is_email` |
| Fieldset | its fields against each other. `.on([..])` names where the problem shows, otherwise it shows under the group | a city needs a zip |
| Form | what spans parts. Without `.on([..])` it shows only in the error summary | a billing address unless the bill goes to the shipping address |

Put a rule on the lowest layer that sees every value it reads. A field's rules
show once it loses focus. A composite problem shows once every field it names
was touched. A submit shows everything, and an error cancels it and focuses a
summary of every problem. An explicit `status`, such as a server's answer,
shows at once, and the worse of it and the rules wins.

## Fields and paths

`#[derive(Fields)]` gives a struct a `FIELDS` constant with one method per
field. Each returns a `FieldPath<Root, T>`, which knows where a `T` sits inside
a `Root` and how it is spelled. `#[fields(nested)]` continues into a field
whose type derives `Fields` too, and the spelling becomes a dotted name like
`shipping.zip`.

```rust,ignore
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
TextField { name: Order::FIELDS.same_billing() }  // does not compile, a TextField holds a String
rule.on([Order::FIELDS.email()])                  // a composite rule over `Order` names a field
```

One path is the name the field posts under, the name a composite rule
addresses and the place the field reads and writes, so the three cannot drift
apart. A misspelled field is a compile error, not a rule that never shows. A path to a `bool` on a `TextField` is a compile error too,
because a field's `name` is typed by the value it holds. `.on(..)` only takes
paths rooted at the rule's own value type.

## Binding

A `Form` takes your value as a `Store`, made with `use_store`. A field inside
it named by a path reads its value from that store and writes every change
back, so no field needs a `value` and `oninput` pair. Typing re-renders only
the field that changed, plus whatever reads the whole value, such as the form's
own rules. A `Fieldset` with a `path` moves everything inside it one level
down, which is what lets a composed part use paths rooted at its own type.

```rust,ignore
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
form. A plain string `name` only posts. A path rooted at a different type than
the form's value warns in debug builds and leaves the field unbound.

## Conditional fields

A field shown only for some values is a plain `if` around it, as the company
switch on the Form page shows. Its rules leave with it, so a hidden field never
blocks a submit, and neither does a rule whose `.on(..)` names only hidden
fields. Its value stays in the store, and a rule on the whole form that names no
field still runs, so check the condition there too.

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
follows the value like any rule. The summary keeps the problems of that submit. A line leaves once it is fixed,
and new ones wait for the next submit. A server's answer is a field's `status`:
an error one set after a valid submit, in `onsubmit` or once a request returns,
shows the summary, which takes focus and is announced, as a blocked submit's
would.
