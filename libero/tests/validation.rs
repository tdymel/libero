//! Typed field paths from `#[derive(Fields)]`, and rules over a whole value.

use libero::components::{FieldPath, FieldStatus, Fields, Rule, Validators, not_empty};

#[derive(Fields)]
pub struct Address {
    pub street: String,
    pub zip: String,
}

#[derive(Fields)]
pub struct Signup {
    pub email: String,
    pub password: String,
    pub confirm: String,
    #[fields(nested)]
    pub address: Address,
}

#[test]
fn derived_paths_spell_the_field_names_and_nest() {
    let email: FieldPath<Signup, String> = Signup::FIELDS.email();
    let zip: FieldPath<Signup, String> = Signup::FIELDS.address().zip();

    assert_eq!(email.as_str(), "email");
    assert_eq!(zip.as_str(), "address.zip");
    assert_eq!(Address::FIELDS.zip().as_str(), "zip");
}

#[test]
fn a_struct_path_is_the_prefix_of_its_fields() {
    let address: FieldPath<Signup, Address> = Signup::FIELDS.address().path();
    assert_eq!(address.as_str(), "address");
    assert_eq!(String::from(Signup::FIELDS.address()), "address");
    assert_eq!(
        address.join(Address::FIELDS.street()).as_str(),
        "address.street"
    );
}

#[test]
fn a_composite_rule_names_the_fields_it_concerns() {
    let rule = (|s: &Signup| s.password == s.confirm)
        .error("Passwords differ")
        .on([Signup::FIELDS.password(), Signup::FIELDS.confirm()]);

    let signup = Signup {
        email: "tom@libero.dev".into(),
        password: "a".into(),
        confirm: "b".into(),
        address: Address {
            street: String::new(),
            zip: String::new(),
        },
    };

    assert_eq!(
        rule.validate(&signup),
        FieldStatus::Error("Passwords differ".into())
    );
    assert_eq!(rule.targets(), ["password", "confirm"]);
}

#[test]
fn rules_of_different_field_types_share_one_set() {
    let rules: Validators<Signup> = [
        (|s: &Signup| not_empty(&s.email))
            .error("Email")
            .on([Signup::FIELDS.email()]),
        (|s: &Signup| not_empty(&s.address.zip))
            .warn("Zip")
            .on([Signup::FIELDS.address().zip()]),
    ]
    .into();

    assert_eq!(rules.iter().count(), 2);
}
