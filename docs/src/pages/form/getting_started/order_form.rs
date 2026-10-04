use super::{Address, AddressFieldset, EmailField};
use dioxus::prelude::*;

// demo-code: start
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
pub fn OrderForm() -> Element {
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
// demo-code: end
