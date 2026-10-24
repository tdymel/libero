use dioxus::prelude::*;

// demo-code: start
use libero::components::{Button, FieldName, FieldPath, Fields, Form, Text, TextField};
use libero::localization::Localization;
use std::fmt;

/// A domain type from your own crate, which knows nothing about libero.
#[derive(Clone, PartialEq)]
pub struct Iban(String);

pub enum IbanError {
    TooShort,
    BadCountry,
}

impl Iban {
    pub fn parse(text: String) -> Result<Iban, IbanError> {
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
pub fn IbanField(#[props(into)] label: String, #[props(into)] name: FieldName<String>) -> Element {
    rsx! {
        TextField { label, name, autocomplete: "off", placeholder: "DE89 3704 0044 0532 0130 00" }
    }
}

/// (b) The wrapper maps: it takes a path to `Option<Iban>` in any form value `R`.
#[component]
pub fn IbanPathField<R: 'static>(#[props(into)] label: String, path: FieldPath<R, Option<Iban>>) -> Element {
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
pub fn PayoutForm() -> Element {
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
// demo-code: end
