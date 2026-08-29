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
                Form::<()> {
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
                Form::<()> {
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
        let signup = use_signal(|| Signup {
            password: "a".into(),
            confirm: "b".into(),
        });
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
                    value: use_signal(Address::default),
                    validate: [
                        (|a: &Address| !a.zip.is_empty()).error("Zip needed").on([crate::path!(Address => zip)]),
                        (|_: &Address| false).warn("Check the whole address"),
                    ],
                    Spy {}
                    TextField { name: "zip", value: "" }
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
        let signup = use_signal(|| Signup {
            password: "a".into(),
            confirm: "b".into(),
        });
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

#[derive(Clone, PartialEq, Default)]
struct Order {
    email: String,
    address: Shipping,
}

#[derive(Clone, PartialEq, Default)]
struct Shipping {
    zip: String,
}

#[test]
fn a_path_name_binds_the_field_to_the_form_value_and_posts_in_full() {
    fn app() -> Element {
        let order = use_signal(|| Order {
            email: "tom@libero.dev".into(),
            address: Shipping {
                zip: "10115".into(),
            },
        });
        rsx! {
            LiberoProvider {
                Form {
                    value: order,
                    TextField { name: crate::path!(Order => email) }
                    Fieldset {
                        path: crate::path!(Order => address),
                        TextField { name: crate::path!(Shipping => zip) }
                    }
                }
            }
        }
    }

    let (_, html) = mount(app);
    assert!(html.contains(r#"name="email""#), "{html}");
    assert!(html.contains(r#"value="tom@libero.dev""#), "{html}");
    assert!(html.contains(r#"name="address.zip""#), "{html}");
    assert!(html.contains(r#"value="10115""#), "{html}");
}

#[test]
fn a_handler_or_a_string_name_leaves_the_field_unbound() {
    fn app() -> Element {
        let order = use_signal(|| Order {
            email: "tom@libero.dev".into(),
            address: Shipping::default(),
        });
        rsx! {
            LiberoProvider {
                Form {
                    value: order,
                    TextField { name: crate::path!(Order => email), oninput: move |_| {} }
                    TextField { name: "email" }
                    // Rooted at the wrong type: warns and stays unbound.
                    TextField { name: crate::path!(Shipping => zip) }
                }
            }
        }
    }

    let (_, html) = mount(app);
    assert!(!html.contains("tom@libero.dev"), "{html}");
}

#[test]
fn a_bound_write_lands_in_the_form_value() {
    use crate::components::{Binding, Source};
    use std::rc::Rc;

    let mut dom = VirtualDom::new(|| rsx! {});
    dom.rebuild_in_place();
    dom.in_scope(dioxus::core::ScopeId::ROOT, || {
        let order = Signal::new(Order::default());
        let form = Binding::root(Some(Rc::new(order) as Rc<dyn Source>));
        let address = crate::components::FieldName::from(crate::path!(Order => address));
        let fieldset = form.narrow(address.as_str(), address.steps());
        let zip = crate::components::FieldName::from(crate::path!(Shipping => zip));

        assert!(fieldset.set(zip.steps().expect("a path"), String::from("10115")));
        assert_eq!(order.peek().address.zip, "10115");
        assert_eq!(fieldset.prefix(), "address");
    });
}

thread_local! {
    static DISABLED: Cell<Option<Signal<bool>>> = const { Cell::new(None) };
}

#[test]
fn a_disabled_fieldset_disables_its_fields_and_nested_groups_and_follows_a_toggle() {
    fn app() -> Element {
        let disabled = use_signal(|| false);
        DISABLED.with(|cell| cell.set(Some(disabled)));
        rsx! {
            LiberoProvider {
                Fieldset::<()> {
                    disabled: disabled(),
                    TextField { label: "Outer", oninput: move |_| {} }
                    Fieldset::<()> {
                        TextField { label: "Inner", oninput: move |_| {} }
                    }
                }
            }
        }
    }

    let (mut dom, before) = mount(app);
    let markup = |html: &str| {
        let body = &html[html.find("<fieldset").unwrap_or(0)..];
        body[..body.find("<style").unwrap_or(body.len())].to_string()
    };
    assert!(!markup(&before).contains("disabled"), "{}", markup(&before));

    dom.in_runtime(|| DISABLED.with(Cell::get).expect("mounted").set(true));
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let after = dioxus_ssr::render(&dom);
    let body = markup(&after);
    // Wrapper and frame of both fields draw the state, the inner one through
    // the nested group; both fieldsets and both inputs carry the attribute.
    assert_eq!(body.matches("radius-sm disabled").count(), 4, "{body}");
    assert_eq!(body.matches("disabled=true").count(), 4, "{body}");
}
