//! When a status shows: a field's rules wait for a blur or a submit, an
//! explicit status never waits, and a composite rule lands on the fields it
//! names. A submit cannot be fired under SSR, so the tests submit the scope
//! directly, the way `Form`'s handler does.

use std::cell::Cell;

use dioxus::prelude::*;

use crate::{
    LiberoProvider,
    components::{FieldStatus, Fieldset, Form, FormScope, Rule, TextField, not_empty},
};

thread_local! {
    static SCOPE: Cell<Option<FormScope>> = const { Cell::new(None) };
}

/// Grabs the scope the nearest `Form` or `Fieldset` provides.
#[component]
fn Spy() -> Element {
    let scope = use_context::<FormScope>();
    SCOPE.with(|cell| cell.set(Some(scope)));
    rsx! {}
}

fn submit(dom: &mut VirtualDom) -> String {
    dom.in_runtime(|| {
        let mut scope = SCOPE.with(Cell::get).expect("a Spy inside a form");
        scope.submit();
    });
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(dom)
}

fn mount(app: fn() -> Element) -> (VirtualDom, String) {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    (dom, html)
}

#[derive(Clone, PartialEq, Default)]
struct Signup {
    password: String,
    confirm: String,
}

#[test]
fn a_rule_waits_for_a_blur_or_a_submit() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Form { value: (),
                    Spy {}
                    TextField { name: "email", value: "", validate: [not_empty.error("Required")] }
                }
            }
        }
    }

    let (mut dom, before) = mount(app);
    assert!(
        !before.contains("Required"),
        "a pristine field shows its rule"
    );

    let after = submit(&mut dom);
    assert!(
        after.contains("Required"),
        "a submit does not reveal the rule"
    );
    assert!(after.contains(r#"aria-invalid="true""#));
}

#[test]
fn an_explicit_status_never_waits_and_the_worst_one_shows() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Form { value: (),
                    Spy {}
                    TextField {
                        value: "",
                        status: FieldStatus::Warning("Server is slow".into()),
                        validate: [not_empty.error("Required")],
                    }
                }
            }
        }
    }

    let (mut dom, before) = mount(app);
    assert!(before.contains("Server is slow"));

    let after = submit(&mut dom);
    assert!(
        after.contains("Required"),
        "the error does not beat the warning"
    );
    assert!(!after.contains("Server is slow"));
}

#[test]
fn a_composite_rule_lands_on_every_field_it_names() {
    fn app() -> Element {
        let signup = Signup {
            password: "a".into(),
            confirm: "b".into(),
        };
        rsx! {
            LiberoProvider {
                Form {
                    value: signup,
                    validate: [
                        (|s: &Signup| s.password == s.confirm)
                            .error("Passwords differ")
                            .on([crate::path!(Signup => password), crate::path!(Signup => confirm)]),
                    ],
                    Spy {}
                    TextField { name: "password", value: "a" }
                    TextField { name: "confirm", value: "b" }
                    TextField { name: "other", value: "" }
                }
            }
        }
    }

    let (mut dom, before) = mount(app);
    assert!(!before.contains("Passwords differ"));

    let after = submit(&mut dom);
    assert_eq!(after.matches("Passwords differ").count(), 2);
}

#[test]
fn a_fieldset_puts_its_prefix_in_front_and_shows_an_unnamed_rule_itself() {
    #[derive(Clone, PartialEq, Default)]
    struct Address {
        zip: String,
    }

    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Fieldset {
                    legend: "Address",
                    path: "address",
                    value: Address::default(),
                    validate: [
                        (|a: &Address| !a.zip.is_empty()).error("Zip needed").on([crate::path!(Address => zip)]),
                        (|_: &Address| false).warn("Check the whole address"),
                    ],
                    Spy {}
                    TextField { name: "address.zip", value: "" }
                }
            }
        }
    }

    let (mut dom, before) = mount(app);
    assert!(before.contains("<legend"));
    assert!(!before.contains("Zip needed"));

    let after = submit(&mut dom);
    assert_eq!(after.matches("Zip needed").count(), 1);
    assert!(after.contains("Check the whole address"));
}

#[test]
fn the_summary_lists_field_errors_and_composite_errors() {
    fn app() -> Element {
        let signup = Signup {
            password: "a".into(),
            confirm: "b".into(),
        };
        rsx! {
            LiberoProvider {
                Form {
                    value: signup,
                    validate: [(|s: &Signup| s.password == s.confirm).error("Passwords differ")],
                    Spy {}
                    TextField { label: "Email", name: "email", value: "", validate: [not_empty.error("Required")] }
                }
            }
        }
    }

    let (dom, _) = mount(app);
    let summary = dom.in_runtime(|| {
        let scope = SCOPE.with(Cell::get).expect("a Spy inside a form");
        assert!(scope.has_errors());
        scope.summary()
    });
    let messages: Vec<_> = summary.iter().map(|item| item.message.as_str()).collect();
    assert_eq!(messages, ["Email: Required", "Passwords differ"]);
    assert!(summary[0].target.is_some());
    assert!(summary[1].target.is_none());
}
