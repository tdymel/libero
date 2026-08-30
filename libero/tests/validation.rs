//! Typed field paths from `#[derive(Fields)]`, and rules over a whole value.

use libero::components::{FieldPath, FieldStatus, Fields, Rule, Validators, not_empty};

#[derive(Clone, PartialEq, Default, Fields)]
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

/// The generated struct is `<Name>FieldPaths`, so `AddressFields` stays free
/// for a component that renders an address's fields.
#[test]
fn the_generated_paths_leave_the_fields_name_free() {
    let _: AddressFieldPaths<Signup> = Signup::FIELDS.address();
    #[allow(dead_code, non_snake_case)]
    fn AddressFields() {}
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

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Order {
    pub name: String,
    #[fields(nested)]
    pub address: Address,
}

#[test]
fn derived_paths_bind_fields_to_the_form_value_and_nest_under_a_fieldset() {
    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Fieldset, Form, TextField},
    };

    fn app() -> Element {
        let order = use_store(|| Order {
            name: "Tom".into(),
            address: Address {
                street: "Hauptstr. 1".into(),
                zip: "10115".into(),
            },
        });
        rsx! {
            LiberoProvider {
                Form {
                    value: order,
                    TextField { name: Order::FIELDS.name() }
                    // The full path works outside a fieldset too.
                    TextField { name: Order::FIELDS.address().street() }
                    Fieldset {
                        path: Order::FIELDS.address(),
                        TextField { name: Address::FIELDS.zip() }
                    }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    for expected in [
        r#"name="name""#,
        r#"value="Tom""#,
        r#"name="address.street""#,
        r#"value="Hauptstr. 1""#,
        r#"name="address.zip""#,
        r#"value="10115""#,
    ] {
        assert!(html.contains(expected), "missing {expected} in {html}");
    }
}

/// `#[derive(Fields)]` keys a field by its position, like `#[derive(Store)]`,
/// so a write through the caller's own store selector reaches the bound field.
#[test]
fn a_write_through_a_derived_store_selector_reaches_the_bound_field() {
    use std::cell::Cell;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Fieldset, Form, TextField},
    };

    #[derive(Clone, PartialEq, Default, Fields, Store)]
    pub struct Parcel {
        pub street: String,
        pub zip: String,
    }

    #[derive(Clone, PartialEq, Default, Fields, Store)]
    pub struct Shipment {
        pub name: String,
        #[fields(nested)]
        pub parcel: Parcel,
    }

    thread_local! {
        static SHIPMENT: Cell<Option<Store<Shipment>>> = const { Cell::new(None) };
    }

    fn app() -> Element {
        let shipment = use_store(Shipment::default);
        SHIPMENT.with(|cell| cell.set(Some(shipment)));
        rsx! {
            LiberoProvider {
                Form {
                    value: shipment,
                    TextField { name: Shipment::FIELDS.name() }
                    Fieldset {
                        path: Shipment::FIELDS.parcel(),
                        TextField { name: Parcel::FIELDS.zip() }
                    }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    dom.in_runtime(|| {
        let shipment = SHIPMENT.with(Cell::get).expect("mounted");
        shipment.parcel().zip().set("10115".into());
        shipment.name().set("Tom".into());
    });
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains(r#"value="10115""#), "{html}");
    assert!(html.contains(r#"value="Tom""#), "{html}");
}
