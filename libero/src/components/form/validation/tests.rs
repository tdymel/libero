//! When a status shows: a field's rules wait for a blur or a submit, an
//! explicit status never waits, and a composite rule lands on the fields it
//! names. A submit cannot be fired under SSR, so the tests submit the scope
//! directly, the way `Form`'s handler does.

use std::cell::Cell;

use dioxus::prelude::*;

use crate::{
    LiberoProvider,
    components::{
        FieldStatus, Fieldset, Form, FormHandle, FormScope, Rule, TextField, not_empty, use_form,
        use_form_context,
    },
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
        let signup = use_store(|| Signup {
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
                    label: "Address",
                    path: "address",
                    value: use_store(Address::default),
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
        let signup = use_store(|| Signup {
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
        let order = use_store(|| Order {
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
        let order = use_store(|| Order {
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
        let order = Store::new(Order::default());
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
    static EMAIL_RENDERS: Cell<usize> = const { Cell::new(0) };
    static STREET_RENDERS: Cell<usize> = const { Cell::new(0) };
    static ZIP_RENDERS: Cell<usize> = const { Cell::new(0) };
    static SET_ZIP: std::cell::RefCell<Option<crate::components::form::Setter<String>>> =
        const { std::cell::RefCell::new(None) };
}

/// A bound field reduced to what binding does: reads its value, counts its
/// renders, and hands out its setter.
#[component]
fn Probe(#[props(into)] name: crate::components::FieldName<String>, renders: usize) -> Element {
    let bound = crate::components::form::use_bound(&name, false);
    let value = bound.value().unwrap_or_default();
    let counter = match renders {
        0 => &EMAIL_RENDERS,
        1 => &STREET_RENDERS,
        _ => &ZIP_RENDERS,
    };
    counter.with(|count| count.set(count.get() + 1));
    if renders == 2 {
        SET_ZIP.with(|cell| *cell.borrow_mut() = bound.setter());
    }
    rsx! { "{value}" }
}

#[test]
fn a_bound_write_re_renders_only_the_field_it_names() {
    #[derive(Clone, PartialEq, Default)]
    struct Place {
        email: String,
        address: Address,
    }
    #[derive(Clone, PartialEq, Default)]
    struct Address {
        street: String,
        zip: String,
    }

    fn app() -> Element {
        let place = use_store(Place::default);
        rsx! {
            LiberoProvider {
                Form {
                    value: place,
                    validate: [(|p: &Place| !p.email.is_empty()).error("Email needed")],
                    Probe { name: crate::path!(Place => email), renders: 0 }
                    Fieldset {
                        path: crate::path!(Place => address),
                        Probe { name: crate::path!(Address => street), renders: 1 }
                        Probe { name: crate::path!(Address => zip), renders: 2 }
                    }
                }
            }
        }
    }

    let (mut dom, _) = mount(app);
    let counts =
        || [&EMAIL_RENDERS, &STREET_RENDERS, &ZIP_RENDERS].map(|count| count.with(Cell::get));
    let before = counts();

    dom.in_runtime(|| {
        SET_ZIP.with(|cell| {
            cell.borrow()
                .clone()
                .expect("zip mounted")
                .set("10115".into())
        })
    });
    let html = settle(&mut dom);
    let after = counts();

    assert!(html.contains("10115"), "{html}");
    assert_eq!(
        after[2],
        before[2] + 1,
        "the written field did not re-render"
    );
    assert_eq!(
        after[0], before[0],
        "a field elsewhere in the form re-rendered"
    );
    assert_eq!(
        after[1], before[1],
        "a sibling field in the fieldset re-rendered"
    );
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

thread_local! {
    static HANDLE: Cell<Option<FormHandle>> = const { Cell::new(None) };
    static PASSED: Cell<Option<FormHandle>> = const { Cell::new(None) };
    static LOGIN: Cell<Option<Store<Login>>> = const { Cell::new(None) };
}

#[derive(Clone, Debug, PartialEq, Default)]
struct Login {
    email: String,
    name: String,
}

/// Grabs what `use_form_context` answers where it is placed.
#[component]
fn HandleSpy() -> Element {
    let handle = use_form_context();
    HANDLE.with(|cell| cell.set(handle));
    rsx! {}
}

fn handle() -> FormHandle {
    HANDLE.with(Cell::get).expect("a HandleSpy inside a form")
}

fn settle(dom: &mut VirtualDom) -> String {
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(dom)
}

/// The summary's markup alone, or `None` while it is not shown.
fn summary_of(html: &str) -> Option<&str> {
    let start = html.find(r#"data-slot="summary""#)?;
    let body = &html[start..];
    Some(&body[..body.find("</ul>").unwrap_or(body.len())])
}

/// Both fields fail while empty, so a submit lists two lines.
fn login_app() -> Element {
    let login = use_store(Login::default);
    LOGIN.with(|cell| cell.set(Some(login)));
    rsx! {
        LiberoProvider {
            Form {
                value: login,
                HandleSpy {}
                TextField {
                    label: "Email",
                    name: crate::path!(Login => email),
                    validate: [not_empty.error("Email needed")],
                }
                TextField {
                    label: "Name",
                    name: crate::path!(Login => name),
                    validate: [not_empty.error("Name needed")],
                }
            }
        }
    }
}

fn login() -> Store<Login> {
    LOGIN.with(Cell::get).expect("the login app mounted")
}

#[test]
fn the_context_reaches_a_forms_handle_and_nothing_outside_one() {
    fn inside() -> Element {
        let form = use_form();
        PASSED.with(|cell| cell.set(Some(form)));
        rsx! {
            LiberoProvider {
                Form::<()> { form, HandleSpy {} }
            }
        }
    }
    fn alone() -> Element {
        rsx! {
            LiberoProvider {
                Fieldset::<()> { HandleSpy {} }
            }
        }
    }

    mount(inside);
    assert!(
        HANDLE.with(Cell::get) == PASSED.with(Cell::get),
        "the context is not the handle passed as `form`"
    );

    HANDLE.with(|cell| cell.set(None));
    mount(alone);
    assert!(
        HANDLE.with(Cell::get).is_none(),
        "a fieldset without a form hands out a handle"
    );
}

#[test]
fn validate_shows_the_summary_and_reset_clears_it_with_the_value() {
    let (mut dom, before) = mount(login_app);
    assert!(summary_of(&before).is_none());

    let valid = dom.in_runtime(|| handle().validate());
    let failed = settle(&mut dom);
    assert!(!valid);
    let summary = summary_of(&failed).expect("a failed validate shows no summary");
    assert!(summary.contains("Email: Email needed"), "{summary}");
    assert!(summary.contains("Name: Name needed"), "{summary}");

    dom.in_runtime(|| {
        login().set(Login {
            email: "tom@libero.dev".into(),
            name: "Tom".into(),
        });
        handle().reset();
    });
    let reset = settle(&mut dom);
    assert!(summary_of(&reset).is_none(), "the summary survived a reset");
    assert!(
        !reset.contains("needed"),
        "a status survived a reset: {reset}"
    );
    assert_eq!(dom.in_runtime(|| login().peek().clone()), Login::default());
}

#[test]
fn a_fixed_line_leaves_the_summary_and_a_new_error_does_not_join_it() {
    let (mut dom, _) = mount(login_app);
    dom.in_runtime(|| handle().validate());
    settle(&mut dom);

    dom.in_runtime(|| login().write().email = "tom@libero.dev".into());
    let one_fixed = settle(&mut dom);
    let summary = summary_of(&one_fixed).expect("the summary vanished with a line left");
    assert!(!summary.contains("Email needed"), "{summary}");
    assert!(summary.contains("Name needed"), "{summary}");

    // Broken again: the field shows it, the summary does not take it back.
    dom.in_runtime(|| login().write().email.clear());
    let broken_again = settle(&mut dom);
    let summary = summary_of(&broken_again).expect("the summary vanished with a line left");
    assert!(!summary.contains("Email needed"), "{summary}");
    assert!(broken_again.contains("Email needed"));

    dom.in_runtime(|| login().write().name = "Tom".into());
    let all_fixed = settle(&mut dom);
    assert!(
        summary_of(&all_fixed).is_none(),
        "an empty summary still shows"
    );
}

#[test]
fn is_valid_follows_the_fields_without_revealing_them() {
    fn app() -> Element {
        let form = use_form();
        let login = use_store(Login::default);
        LOGIN.with(|cell| cell.set(Some(login)));
        rsx! {
            LiberoProvider {
                span { if form.is_valid() { "valid" } else { "invalid" } }
                Form {
                    form,
                    value: login,
                    TextField { name: crate::path!(Login => email), validate: [not_empty.error("Email needed")] }
                }
            }
        }
    }

    let (mut dom, _) = mount(app);
    let before = settle(&mut dom);
    assert!(before.contains(">invalid<"), "{before}");
    assert!(
        !before.contains("Email needed"),
        "is_valid revealed a status"
    );

    dom.in_runtime(|| login().write().email = "tom@libero.dev".into());
    let after = settle(&mut dom);
    assert!(after.contains(">valid<"), "{after}");
}
