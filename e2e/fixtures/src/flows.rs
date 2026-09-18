//! Use-case flows across several components (todo 824): a signup form with
//! every common field type, a filtered and paged table, and a form in a dialog.

use dioxus::prelude::*;
use libero::{
    components::{
        Button, Checkbox, Dialog, Fields, Fieldset, Flex, Form, NumberField, Options, Pagination,
        RadioGroup, Rule, Select, Switch, Table, TextField, Textarea, column, is_email, max_length,
        not_empty,
    },
    hooks::{ModalScope, use_modal},
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/flows/signup", || rsx! { SignupPage {} }),
    ("/flows/orders", || rsx! { OrdersPage {} }),
    ("/flows/dialog-form", || rsx! { DialogFormPage {} }),
];

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Plan {
    Free,
    Team,
    Enterprise,
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Contact {
    Email,
    Phone,
}

#[derive(Clone, PartialEq, Default, Debug, Fields)]
struct Address {
    street: String,
    zip: String,
    city: String,
}

#[derive(Clone, PartialEq, Default, Debug, Fields)]
struct Signup {
    name: String,
    email: String,
    age: Option<u32>,
    plan: Option<Plan>,
    contact: Option<Contact>,
    #[fields(nested)]
    address: Address,
    notes: String,
    newsletter: bool,
    terms: bool,
}

/// Every field bound by path; the submitted value is printed in `#submitted`.
#[component]
fn SignupPage() -> Element {
    let value = use_store(Signup::default);
    let mut submitted = use_signal(|| None::<String>);
    let mut submits = use_signal(|| 0u32);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            Form {
                value,
                summary_title: "Fix these fields",
                onsubmit: move |_| {
                    submits += 1;
                    submitted.set(Some(format!("{:?}", value())));
                },
                TextField { label: "Name", name: Signup::FIELDS.name(), validate: not_empty.error("Enter your name.") }
                TextField {
                    label: "Email",
                    r#type: "email",
                    name: Signup::FIELDS.email(),
                    validate: [not_empty.error("Enter your email."), is_email.error("That is not an email address.")],
                }
                NumberField::<u32> {
                    label: "Age",
                    name: Signup::FIELDS.age(),
                    validate: (|age: &Option<u32>| age.is_some_and(|age| age >= 18)).error("You must be 18."),
                }
                Select::<Plan> { label: "Plan", name: Signup::FIELDS.plan(), placeholder: "Pick a plan", validate: not_empty.error("Pick a plan.") }
                RadioGroup::<Contact> { label: "Contact me by", name: Signup::FIELDS.contact(), validate: not_empty.error("Pick a way to reach you.") }
                Fieldset {
                    label: "Address",
                    path: Signup::FIELDS.address(),
                    validate: (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                        .error("A city needs its zip code.")
                        .on([Address::FIELDS.zip()]),
                    TextField { label: "Street", name: Address::FIELDS.street(), validate: not_empty.error("Enter a street.") }
                    TextField { label: "Zip code", name: Address::FIELDS.zip() }
                    TextField { label: "City", name: Address::FIELDS.city() }
                }
                Textarea { label: "Notes", name: Signup::FIELDS.notes(), validate: max_length(20).error("Keep notes under 20 characters.") }
                Switch { label: "Newsletter", name: Signup::FIELDS.newsletter() }
                Checkbox { label: "I accept the terms", name: Signup::FIELDS.terms(), validate: not_empty.error("Accept the terms.") }
                Button { r#type: "submit", "Sign up" }
            }
            span { id: "submits", "data-submits": "{submits}", "Submits {submits}" }
            if let Some(submitted) = submitted() {
                pre { id: "submitted", white_space: "pre-wrap", "{submitted}" }
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct Order {
    id: u32,
    customer: &'static str,
    total: u32,
}

const CUSTOMERS: [&str; 12] = [
    "Ada", "Alan", "Barbara", "Claude", "Dana", "Edsger", "Frances", "Grace", "Hedy", "Ivan",
    "Joan", "Ken",
];

/// Five rows a page.
const PER_PAGE: usize = 5;

fn orders() -> Vec<Order> {
    CUSTOMERS
        .iter()
        .enumerate()
        .map(|(i, customer)| Order {
            id: 100 + i as u32,
            customer,
            total: (i as u32 * 37) % 90 + 10,
        })
        .collect()
}

/// A filter field over a sortable table, paged by `Pagination`. A new filter
/// goes back to page 1.
#[component]
fn OrdersPage() -> Element {
    let mut query = use_signal(String::new);
    let mut page = use_signal(|| 1u32);
    let matching: Vec<Order> = orders()
        .into_iter()
        .filter(|order| {
            order
                .customer
                .to_lowercase()
                .contains(&query.read().to_lowercase())
        })
        .collect();
    let pages = matching.len().div_ceil(PER_PAGE).max(1) as u32;
    let current = page().min(pages);
    let rows: Vec<Order> = matching
        .iter()
        .skip((current as usize - 1) * PER_PAGE)
        .take(PER_PAGE)
        .cloned()
        .collect();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "420px",
            TextField {
                label: "Filter customers",
                value: query(),
                oninput: move |next: String| {
                    query.set(next);
                    page.set(1);
                },
            }
            span { id: "matching", "data-matching": "{matching.len()}", "{matching.len()} orders" }
            Table {
                aria_label: "Orders",
                empty: rsx! { "No orders match" },
                data: rows,
                columns: vec![
                    column("Order").value(|order: &Order| order.id).row_header(),
                    column("Customer").value(|order: &Order| order.customer.to_string()).sortable(),
                    column("Total").value(|order: &Order| order.total).sortable(),
                ],
            }
            Pagination {
                total: pages,
                page: current,
                aria_label: "Order pages",
                onchange: move |next: u32| page.set(next),
            }
        }
    }
}

#[derive(Clone, PartialEq, Default, Debug, Fields)]
struct Member {
    name: String,
    plan: Option<Plan>,
}

/// A list and an "Add member" dialog holding a form. A valid submit resolves
/// the dialog with the member, which the list appends; Escape adds nothing.
#[component]
fn DialogFormPage() -> Element {
    let mut members = use_signal(Vec::<String>::new);
    let mut dismissed = use_signal(|| 0u32);
    let add = use_modal(|s: ModalScope<(), Member>| {
        rsx! {
            Dialog { title: "Add member", size: "sm",
                MemberForm { onsave: move |member| s.resolve(member) }
            }
        }
    });

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Button {
                id: "add-member",
                onclick: move |_| {
                    add.open().onresult(move |member: Option<Member>| match member {
                        Some(member) => members.write().push(format!("{} ({:?})", member.name, member.plan.unwrap())),
                        None => dismissed += 1,
                    });
                },
                "Add member"
            }
            span { id: "dismissed", "data-dismissed": "{dismissed}", "Dismissed {dismissed}" }
            ul { id: "members",
                for member in members() {
                    li { "{member}" }
                }
            }
        }
    }
}

#[component]
fn MemberForm(onsave: EventHandler<Member>) -> Element {
    let value = use_store(Member::default);
    rsx! {
        Form {
            value,
            onsubmit: move |_| onsave.call(value()),
            TextField { label: "Member name", name: Member::FIELDS.name(), validate: not_empty.error("Enter a name.") }
            Select::<Plan> { label: "Member plan", name: Member::FIELDS.plan(), placeholder: "Pick a plan", validate: not_empty.error("Pick a plan.") }
            Button { r#type: "submit", "Save member" }
        }
    }
}
