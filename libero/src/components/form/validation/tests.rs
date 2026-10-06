//! When a status shows. SSR cannot fire a submit, so the tests submit the
//! scope directly, as `Form`'s handler does.

use std::{cell::Cell, rc::Rc};

use dioxus::{core::provide_root_context, prelude::*};

use crate::{
    LiberoProvider,
    components::{
        FieldStatus, Fieldset, Form, FormHandle, FormScope, Rule, TextField, min_length, not_empty,
        use_form, use_form_context,
    },
};

/// Hands `value` to the test, which reads it back with `exposed`. Root context,
/// not a `thread_local!`, so every `VirtualDom` keeps its own.
fn expose<T: Clone + 'static>(value: T) {
    use_hook(|| provide_root_context(value));
}

fn exposed<T: Clone + 'static>(dom: &VirtualDom) -> T {
    dom.in_scope(ScopeId::ROOT, consume_context::<T>)
}

/// Exposes the scope the nearest `Form` or `Fieldset` provides.
#[component]
fn Spy() -> Element {
    expose(use_context::<FormScope>());
    rsx! {}
}

fn scope(dom: &VirtualDom) -> FormScope {
    exposed(dom)
}

fn submit(dom: &mut VirtualDom) -> String {
    let mut scope = scope(dom);
    dom.in_runtime(|| scope.submit());
    settle(dom)
}

fn mount(app: fn() -> Element) -> (VirtualDom, String) {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    (dom, html)
}

fn settle(dom: &mut VirtualDom) -> String {
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(dom)
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

/// The composite rule of `a_composite_rule_lands_on_every_field_it_names`,
/// with both fields behind a toggle.
fn toggled_signup() -> Element {
    let signup = use_store(|| Signup {
        password: "a".into(),
        confirm: "b".into(),
    });
    let shown = use_signal(|| true);
    expose(shown);
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
                if shown() {
                    TextField { name: "password", value: "a" }
                    TextField { name: "confirm", value: "b" }
                }
            }
        }
    }
}

fn toggle(dom: &mut VirtualDom, to: bool) -> String {
    let mut shown = exposed::<Signal<bool>>(dom);
    dom.in_runtime(|| shown.set(to));
    settle(dom)
}

/// What each field's `focusout` does to the shared scope.
fn touch(dom: &mut VirtualDom, names: &[&str]) -> String {
    let mut scope = scope(dom);
    dom.in_runtime(|| {
        for name in names {
            scope.touch(name);
        }
    });
    settle(dom)
}

#[test]
fn a_remounted_field_waits_for_its_own_blur_again() {
    let (mut dom, _) = mount(toggled_signup);
    let touched = touch(&mut dom, &["password", "confirm"]);
    assert_eq!(touched.matches("Passwords differ").count(), 2, "{touched}");

    toggle(&mut dom, false);
    let remounted = toggle(&mut dom, true);
    assert!(
        !remounted.contains("Passwords differ"),
        "a remounted field kept its old touched state: {remounted}"
    );
    assert!(!dom.in_runtime(|| scope(&dom).touched_under("")));
}

#[test]
fn a_remounted_field_after_a_failed_submit_still_shows_its_error() {
    let (mut dom, _) = mount(toggled_signup);
    submit(&mut dom);

    toggle(&mut dom, false);
    let remounted = toggle(&mut dom, true);
    assert_eq!(
        remounted.matches("Passwords differ").count(),
        2,
        "{remounted}"
    );
}

#[test]
fn a_field_leaving_keeps_the_touched_name_another_field_still_carries() {
    fn app() -> Element {
        let shown = use_signal(|| true);
        expose(shown);
        rsx! {
            LiberoProvider {
                Form::<()> {
                    Spy {}
                    TextField { name: "topic", value: "" }
                    if shown() {
                        TextField { name: "topic", value: "" }
                    }
                }
            }
        }
    }

    let (mut dom, _) = mount(app);
    touch(&mut dom, &["topic"]);
    toggle(&mut dom, false);
    assert!(dom.in_runtime(|| scope(&dom).touched_under("topic")));
}

#[test]
fn a_fieldset_puts_its_prefix_in_front_and_shows_an_unnamed_rule_itself() {
    #[derive(Clone, PartialEq, Default)]
    struct Address {
        zip: String,
    }

    fn app() -> Element {
        let address = use_store(Address::default);
        rsx! {
            LiberoProvider {
                Fieldset {
                    label: "Address",
                    path: "address",
                    value: address,
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
        let scope = scope(&dom);
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
    /// Has a `zip` like `Shipping`, so only the root type tells them apart.
    #[derive(Clone, PartialEq, Default)]
    struct Parcel {
        zip: String,
    }

    fn app() -> Element {
        let order = use_store(|| Order {
            email: "tom@libero.dev".into(),
            address: Shipping::default(),
        });
        let parcel = use_store(|| Parcel {
            zip: "99999".into(),
        });
        rsx! {
            LiberoProvider {
                Form {
                    value: order,
                    TextField { name: crate::path!(Order => email), oninput: move |_| {} }
                    TextField { name: "email" }
                }
                Form {
                    value: parcel,
                    // Rooted at the wrong type: warns and stays unbound.
                    TextField { name: crate::path!(Shipping => zip) }
                }
            }
        }
    }

    let (_, html) = mount(app);
    assert!(!html.contains("tom@libero.dev"), "{html}");
    assert!(!html.contains("99999"), "{html}");
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

/// The render count of each `Probe` by slot, and the setter of the last one.
#[derive(Default)]
struct Probes {
    renders: [Cell<usize>; 3],
    setter: std::cell::RefCell<Option<crate::components::form::Setter<String>>>,
}

/// A bound field reduced to what binding does: reads its value, counts its
/// renders, and hands out its setter.
#[component]
fn Probe(#[props(into)] name: crate::components::FieldName<String>, slot: usize) -> Element {
    let bound = crate::components::form::use_bound(&name, false);
    let value = bound.value().unwrap_or_default();
    let probes = use_context::<Rc<Probes>>();
    probes.renders[slot].set(probes.renders[slot].get() + 1);
    if slot == 2 {
        *probes.setter.borrow_mut() = bound.setter();
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
        expose(Rc::new(Probes::default()));
        rsx! {
            LiberoProvider {
                Form {
                    value: place,
                    validate: [(|p: &Place| !p.email.is_empty()).error("Email needed")],
                    Probe { name: crate::path!(Place => email), slot: 0 }
                    Fieldset {
                        path: crate::path!(Place => address),
                        Probe { name: crate::path!(Address => street), slot: 1 }
                        Probe { name: crate::path!(Address => zip), slot: 2 }
                    }
                }
            }
        }
    }

    let (mut dom, _) = mount(app);
    let probes = exposed::<Rc<Probes>>(&dom);
    let counts = || probes.renders.each_ref().map(Cell::get);
    let before = counts();

    let set_zip = probes.setter.borrow().clone().expect("zip mounted");
    dom.in_runtime(|| set_zip.set("10115".into()));
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

#[test]
fn a_disabled_fieldset_disables_its_fields_and_nested_groups_and_follows_a_toggle() {
    fn app() -> Element {
        let disabled = use_signal(|| false);
        expose(disabled);
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

    // Both fieldsets and inputs by attribute, each field's wrapper and frame by
    // `data-state`; the inner field through the nested group.
    let parts = [
        "fieldset", "wrapper", "frame", "input", "fieldset", "wrapper", "frame", "input",
    ];
    let (mut dom, before) = mount(app);
    assert_eq!(disabled_parts(&before), parts.map(|part| (part, false)));

    let mut disabled = exposed::<Signal<bool>>(&dom);
    dom.in_runtime(|| disabled.set(true));
    let after = settle(&mut dom);
    assert_eq!(disabled_parts(&after), parts.map(|part| (part, true)));
}

/// Each fieldset, input and `data-state` element of a fieldset's markup, in
/// document order, and whether it says disabled.
fn disabled_parts(html: &str) -> Vec<(&str, bool)> {
    let body = &html[html.find("<fieldset").unwrap_or(0)..];
    let body = &body[..body.find("<style").unwrap_or(body.len())];
    body.split('<')
        .filter_map(|tag| {
            let (name, attributes) = tag[..tag.find('>')?].split_once(' ')?;
            match name {
                "fieldset" | "input" => {
                    Some((name, attribute(attributes, "disabled") == Some("true")))
                }
                _ => {
                    let state = attribute(attributes, "data-state")?;
                    let part = attribute(attributes, "data-slot").unwrap_or("wrapper");
                    Some((part, state.split(' ').any(|token| token == "disabled")))
                }
            }
        })
        .collect()
}

/// The value of attribute `name` in an SSR tag's attribute text.
fn attribute<'a>(attributes: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=");
    let (at, _) = attributes
        .match_indices(&key)
        .find(|(at, _)| *at == 0 || attributes.as_bytes()[at - 1] == b' ')?;
    let value = &attributes[at + key.len()..];
    match value.strip_prefix('"') {
        Some(quoted) => quoted.split('"').next(),
        None => value.split(' ').next(),
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
struct Login {
    email: String,
    name: String,
}

/// Exposes what `use_form_context` answers where it is placed.
#[component]
fn HandleSpy() -> Element {
    expose(use_form_context());
    rsx! {}
}

fn handle(dom: &VirtualDom) -> FormHandle {
    exposed::<Option<FormHandle>>(dom).expect("a HandleSpy inside a form")
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
    expose(login);
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

fn login(dom: &VirtualDom) -> Store<Login> {
    exposed(dom)
}

#[test]
fn the_context_reaches_a_forms_handle_and_nothing_outside_one() {
    fn inside() -> Element {
        let form = use_form();
        expose(form);
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

    let (dom, _) = mount(inside);
    assert!(
        handle(&dom) == exposed::<FormHandle>(&dom),
        "the context is not the handle passed as `form`"
    );

    let (dom, _) = mount(alone);
    assert!(
        exposed::<Option<FormHandle>>(&dom).is_none(),
        "a fieldset without a form hands out a handle"
    );
}

/// Todo 2556: a field still in error keeps its line when its message changes.
#[test]
fn a_line_follows_its_fields_new_message() {
    fn app() -> Element {
        let login = use_store(Login::default);
        expose(login);
        rsx! {
            LiberoProvider {
                Form {
                    value: login,
                    HandleSpy {}
                    TextField {
                        label: "Name",
                        name: crate::path!(Login => name),
                        validate: [
                            not_empty.error("Name needed"),
                            min_length(3).error("Too short"),
                        ],
                    }
                }
            }
        }
    }

    let (mut dom, _) = mount(app);
    dom.in_runtime(|| handle(&dom).validate());
    settle(&mut dom);

    dom.in_runtime(|| login(&dom).write().name = "T".into());
    let changed = settle(&mut dom);
    let summary = summary_of(&changed).expect("the summary vanished with the field in error");
    assert!(summary.contains("Name: Too short"), "{summary}");
    assert!(!summary.contains("Name needed"), "{summary}");

    dom.in_runtime(|| login(&dom).write().name = "Tom".into());
    let fixed = settle(&mut dom);
    assert!(summary_of(&fixed).is_none(), "a fixed line stays");
}

#[test]
fn validate_shows_the_summary_and_reset_clears_it_with_the_value() {
    let (mut dom, before) = mount(login_app);
    assert!(summary_of(&before).is_none());

    let valid = dom.in_runtime(|| handle(&dom).validate());
    let failed = settle(&mut dom);
    assert!(!valid);
    let summary = summary_of(&failed).expect("a failed validate shows no summary");
    assert!(summary.contains("Email: Email needed"), "{summary}");
    assert!(summary.contains("Name: Name needed"), "{summary}");

    dom.in_runtime(|| {
        login(&dom).set(Login {
            email: "tom@libero.dev".into(),
            name: "Tom".into(),
        });
        handle(&dom).reset();
    });
    let reset = settle(&mut dom);
    assert!(summary_of(&reset).is_none(), "the summary survived a reset");
    assert!(
        !reset.contains("needed"),
        "a status survived a reset: {reset}"
    );
    assert_eq!(
        dom.in_runtime(|| login(&dom).peek().clone()),
        Login::default()
    );
}

#[test]
fn a_fixed_line_leaves_the_summary_and_a_new_error_does_not_join_it() {
    let (mut dom, _) = mount(login_app);
    dom.in_runtime(|| handle(&dom).validate());
    settle(&mut dom);

    dom.in_runtime(|| login(&dom).write().email = "tom@libero.dev".into());
    let one_fixed = settle(&mut dom);
    let summary = summary_of(&one_fixed).expect("the summary vanished with a line left");
    assert!(!summary.contains("Email needed"), "{summary}");
    assert!(summary.contains("Name needed"), "{summary}");

    // Broken again: the field shows it, the summary does not take it back.
    dom.in_runtime(|| login(&dom).write().email.clear());
    let broken_again = settle(&mut dom);
    let summary = summary_of(&broken_again).expect("the summary vanished with a line left");
    assert!(!summary.contains("Email needed"), "{summary}");
    assert!(broken_again.contains("Email needed"));

    dom.in_runtime(|| login(&dom).write().name = "Tom".into());
    let all_fixed = settle(&mut dom);
    assert!(
        summary_of(&all_fixed).is_none(),
        "an empty summary still shows"
    );
}

/// Equal rules let the field skip its parent's render; a new message must not.
#[test]
fn a_changed_rule_message_redraws_a_field_that_otherwise_skips() {
    fn app() -> Element {
        let message = use_signal(|| "First");
        expose(message);
        rsx! {
            LiberoProvider {
                Form::<()> {
                    Spy {}
                    TextField { name: "email", value: "", validate: [not_empty.error(message())] }
                }
            }
        }
    }

    let (mut dom, _) = mount(app);
    assert!(submit(&mut dom).contains("First"));

    let mut message = exposed::<Signal<&str>>(&dom);
    dom.in_runtime(|| message.set("Second"));
    let after = settle(&mut dom);
    assert!(after.contains("Second"), "the field kept a stale message");
    assert!(!after.contains("First"));
}

#[test]
fn is_valid_follows_the_fields_without_revealing_them() {
    fn app() -> Element {
        let form = use_form();
        let login = use_store(Login::default);
        expose(login);
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

    dom.in_runtime(|| login(&dom).write().email = "tom@libero.dev".into());
    let after = settle(&mut dom);
    assert!(after.contains(">valid<"), "{after}");
}

#[test]
fn a_field_shown_by_another_fields_value_only_validates_while_shown() {
    fn app() -> Element {
        let login = use_store(Login::default);
        expose(login);
        rsx! {
            LiberoProvider {
                Form {
                    value: login,
                    Spy {}
                    TextField { name: crate::path!(Login => email) }
                    if !login.read().email.is_empty() {
                        TextField { name: crate::path!(Login => name), validate: [not_empty.error("Name needed")] }
                    }
                }
            }
        }
    }
    let has_errors = |dom: &VirtualDom| dom.in_runtime(|| scope(dom).has_errors());

    let (mut dom, _) = mount(app);
    let hidden = submit(&mut dom);
    assert!(!hidden.contains("Name needed"), "{hidden}");
    assert!(!has_errors(&dom), "a hidden field blocks the submit");

    dom.in_runtime(|| login(&dom).write().email = "tom@libero.dev".into());
    settle(&mut dom);
    assert!(has_errors(&dom), "the shown field does not validate");

    dom.in_runtime(|| login(&dom).write().email.clear());
    settle(&mut dom);
    assert!(!has_errors(&dom), "the field kept its error after hiding");
}

/// Todo 1273 (Maintainer): a `Fieldset` in error is one input in error, as a field's explicit
/// status: it blocks a submit and has its own summary line, named by its legend.
#[test]
fn a_fieldset_in_error_blocks_the_submit_and_joins_the_summary() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Form::<()> {
                    Spy {}
                    Fieldset::<()> {
                        label: "Address",
                        status: "Address not found",
                        TextField { name: "zip", value: "" }
                    }
                }
            }
        }
    }

    let (dom, _) = mount(app);
    let scope = scope(&dom);
    let summary = dom.in_runtime(|| {
        assert!(scope.has_errors(), "the fieldset's error does not block");
        scope.summary()
    });
    let messages: Vec<_> = summary.iter().map(|item| item.message.as_str()).collect();
    assert_eq!(messages, ["Address: Address not found"]);
    assert!(summary[0].target.is_some());
}
