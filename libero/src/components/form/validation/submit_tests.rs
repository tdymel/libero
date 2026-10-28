//! Errors set by path, the `required` helper, and `.choices(..)` through every pick field.

use std::rc::Rc;

use dioxus::{core::provide_root_context, prelude::*};

use crate::{
    LiberoProvider, choices,
    components::{
        Rule,
        common::{Choice, Choices},
        form::{
            CheckboxGroup, FieldName, FieldPath, Form, FormErrors, FormHandle, FormScope,
            MultiSelect, NativeSelect, RadioGroup, RequiredValue, SegmentedControl, Select, Submit,
            TextField, use_bound, use_form,
        },
        not_empty,
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Country {
    De,
    At,
    Ch,
}

fn country_label(country: &Country) -> String {
    match country {
        Country::De => "Germany",
        Country::At => "Austria",
        Country::Ch => "Switzerland",
    }
    .into()
}

static COUNTRIES: Choices<Country> =
    choices!(Country, country_label, default = At, { De => "de", At => "at", Ch => "ch" });

#[derive(Clone, PartialEq, Default)]
struct Signup {
    email: String,
    select: Option<Country>,
    native: Option<Country>,
    radio: Option<Country>,
    strip: Option<Country>,
    multi: Vec<Country>,
    checks: Vec<Country>,
}

fn email_path() -> FieldPath<Signup, String> {
    crate::path!(Signup => email)
}

fn expose<T: Clone + 'static>(value: T) {
    use_hook(|| provide_root_context(value));
}

fn exposed<T: Clone + 'static>(dom: &VirtualDom) -> T {
    dom.in_scope(ScopeId::ROOT, consume_context::<T>)
}

fn settle(dom: &mut VirtualDom) -> String {
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(dom)
}

#[derive(Clone)]
struct Typist(Rc<dyn Fn(String)>);

/// Types into the email field's binding, as its `oninput` would.
#[component]
fn EmailProbe() -> Element {
    let bound = use_bound(&FieldName::from(email_path()), false);
    let emit = bound.emit(None).expect("a bound field");
    expose(Typist(Rc::new(emit)));
    expose(use_context::<FormScope>());
    rsx! {}
}

fn app() -> Element {
    let signup = use_store(Signup::default);
    let form = use_form();
    expose(signup);
    expose(form);
    rsx! {
        Form { form, value: signup,
            EmailProbe {}
            TextField { label: "Email", name: email_path() }
        }
    }
}

#[component]
fn Wrapper() -> Element {
    app()
}

fn mount() -> VirtualDom {
    let mut dom = VirtualDom::new(|| rsx! { LiberoProvider { Wrapper {} } });
    dom.rebuild_in_place();
    settle(&mut dom);
    dom
}

fn taken() -> FormErrors {
    FormErrors::new().at(email_path(), "This address is taken.")
}

#[test]
fn a_set_error_lands_on_its_field_and_in_the_summary() {
    let mut dom = mount();
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(taken()));
    let html = settle(&mut dom);
    assert!(html.contains("This address is taken."), "{html}");
    assert!(html.contains(r#"aria-invalid="true""#), "{html}");
    let scope = exposed::<FormScope>(&dom);
    let summary = dom.in_runtime(|| scope.summary());
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].message, "Email: This address is taken.");
    assert!(summary[0].target.is_some());
    assert!(
        html.contains(r#"data-slot="summary""#),
        "the summary is not shown"
    );
    assert!(dom.in_runtime(|| scope.has_errors()));
}

#[test]
fn two_errors_on_one_path_both_show() {
    let mut dom = mount();
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(taken().at(email_path(), "Use a work address.")));
    let html = settle(&mut dom);
    assert!(
        html.contains("This address is taken. Use a work address."),
        "{html}"
    );
    let scope = exposed::<FormScope>(&dom);
    let summary = dom.in_runtime(|| scope.summary());
    let messages: Vec<_> = summary.iter().map(|item| item.message.as_str()).collect();
    assert_eq!(
        messages,
        [
            "Email: This address is taken.",
            "Email: Use a work address."
        ]
    );
}

/// Todo 4754: a server error blocks the submit, so the summary lists it beside the field's own.
#[test]
fn a_set_error_joins_the_fields_own_error_in_the_summary() {
    #[component]
    fn Ruled() -> Element {
        let signup = use_store(Signup::default);
        let form = use_form();
        expose(form);
        rsx! {
            Form { form, value: signup,
                EmailProbe {}
                TextField { label: "Email", name: email_path(), validate: [not_empty.error("Required")] }
            }
        }
    }

    let mut dom = VirtualDom::new(|| rsx! { LiberoProvider { Ruled {} } });
    dom.rebuild_in_place();
    settle(&mut dom);
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(taken()));
    settle(&mut dom);
    let scope = exposed::<FormScope>(&dom);
    let summary = dom.in_runtime(|| scope.summary());
    let messages: Vec<_> = summary.iter().map(|item| item.message.as_str()).collect();
    assert_eq!(
        messages,
        ["Email: Required", "Email: This address is taken."]
    );
}

#[test]
fn an_edit_clears_its_fields_error() {
    let mut dom = mount();
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(taken()));
    settle(&mut dom);
    let Typist(emit) = exposed(&dom);
    dom.in_runtime(|| emit("other@example.com".into()));
    let html = settle(&mut dom);
    assert!(!html.contains("This address is taken."), "{html}");
    let scope = exposed::<FormScope>(&dom);
    assert!(!dom.in_runtime(|| scope.has_errors()));
}

#[test]
fn the_next_submit_clears_every_set_error() {
    let mut dom = mount();
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(taken().at("nowhere", "Something else.")));
    settle(&mut dom);
    dom.in_runtime(|| form.submit().expect("the form is mounted"));
    let html = settle(&mut dom);
    assert!(!html.contains("This address is taken."), "{html}");
    assert!(!html.contains("Something else."), "{html}");
}

#[test]
fn clear_errors_drops_them_and_unblocks() {
    let mut dom = mount();
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(taken()));
    settle(&mut dom);
    dom.in_runtime(|| form.clear_errors());
    let html = settle(&mut dom);
    assert!(!html.contains("This address is taken."));
    assert!(dom.in_runtime(|| form.is_valid()));
}

#[test]
fn an_error_on_a_path_no_field_carries_has_an_unlinked_line() {
    let mut dom = mount();
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.set_errors(FormErrors::new().at("terms", "Accept the terms.")));
    settle(&mut dom);
    let scope = exposed::<FormScope>(&dom);
    let summary = dom.in_runtime(|| scope.summary());
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].message, "Accept the terms.");
    assert_eq!(summary[0].target, None);
}

#[test]
fn at_with_words_a_foreign_error() {
    struct Taken;
    let errors = FormErrors::new().at_with(email_path(), Taken, |_: &Taken, _| "Taken.".into());
    let entries = errors.into_entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "email");
    let locale = super::validator::locale();
    assert_eq!(entries[0].1.message(locale), "Taken.");
}

#[test]
fn required_turns_none_into_an_error_on_the_path() {
    let some: Option<u32> = Some(3);
    assert_eq!(some.required("amount").ok(), Some(3));
    let none: Option<u32> = None;
    let entries = none.required("amount").expect_err("missing").into_entries();
    let locale = super::validator::locale();
    assert_eq!(entries[0].0, "amount");
    assert_eq!(entries[0].1.message(locale), locale.form.required);
}

/// What a parsing form saw: parse calls and the commands its handler got.
#[derive(Clone, Default)]
struct Seen {
    parsed: Rc<std::cell::Cell<u32>>,
    commands: Rc<std::cell::RefCell<Vec<String>>>,
}

fn parsing_app() -> Element {
    let signup = use_store(Signup::default);
    let form = use_form();
    let seen = use_hook(Seen::default);
    expose(signup);
    expose(form);
    expose(seen.clone());
    let parsed = seen.parsed.clone();
    let parse = move |signup: &Signup| {
        parsed.set(parsed.get() + 1);
        match signup.email.ends_with("@example.com") {
            true => Err(FormErrors::new().at(email_path(), "Use your own address.")),
            false => Ok(signup.email.clone()),
        }
    };
    let commands = seen.commands.clone();
    rsx! {
        Form { form, value: signup,
            onsubmit: Submit::parse(parse, move |email: String| commands.borrow_mut().push(email)),
            EmailProbe {}
            TextField { label: "Email", required: true, name: email_path() }
        }
    }
}

#[component]
fn ParsingWrapper() -> Element {
    parsing_app()
}

fn submit_with(email: &str) -> (VirtualDom, String) {
    let mut dom = VirtualDom::new(|| rsx! { LiberoProvider { ParsingWrapper {} } });
    dom.rebuild_in_place();
    settle(&mut dom);
    if !email.is_empty() {
        let Typist(emit) = exposed(&dom);
        dom.in_runtime(|| emit(email.into()));
        settle(&mut dom);
    }
    let form = exposed::<FormHandle>(&dom);
    dom.in_runtime(|| form.submit().expect("the form is mounted"));
    let html = settle(&mut dom);
    (dom, html)
}

#[test]
fn field_rules_run_before_parse() {
    let (dom, html) = submit_with("");
    let seen = exposed::<Seen>(&dom);
    assert_eq!(seen.parsed.get(), 0, "parse ran over a blocked form");
    assert!(seen.commands.borrow().is_empty());
    assert!(html.contains(r#"aria-invalid="true""#), "{html}");
}

#[test]
fn a_parse_error_lands_on_its_field_and_stops_the_handler() {
    let (dom, html) = submit_with("me@example.com");
    let seen = exposed::<Seen>(&dom);
    assert_eq!(seen.parsed.get(), 1);
    assert!(seen.commands.borrow().is_empty(), "the handler ran");
    assert!(html.contains("Use your own address."), "{html}");
    assert!(html.contains(r#"data-slot="summary""#), "no summary");
    let scope = exposed::<FormScope>(&dom);
    let summary = dom.in_runtime(|| scope.summary());
    assert_eq!(summary[0].message, "Email: Use your own address.");
}

#[test]
fn a_parsed_command_reaches_the_handler() {
    let (dom, html) = submit_with("me@libero.dev");
    let seen = exposed::<Seen>(&dom);
    assert_eq!(*seen.commands.borrow(), ["me@libero.dev"]);
    assert!(!html.contains(r#"data-slot="summary""#), "{html}");
}

fn select_name() -> FieldName<Option<Choice<Country>>> {
    crate::path!(Signup => select).choices(&COUNTRIES).into()
}

#[test]
fn a_single_pick_round_trips_through_its_choice() {
    let name = select_name();
    let adapter = name.adapter().expect("mapped");
    let mut slot: Option<Country> = Some(Country::Ch);
    let shown = adapter.read(&slot).expect("the slot fits");
    assert_eq!(shown.as_ref().map(Choice::key), Some("ch"));
    assert_eq!(shown.as_deref(), Some(&Country::Ch));

    adapter.write(&mut slot, COUNTRIES.by_key("de"));
    assert_eq!(slot, Some(Country::De));
    adapter.write(&mut slot, None);
    assert_eq!(slot, None);
    assert!(matches!(adapter.read(&slot), Some(None)));
}

#[test]
fn a_multi_pick_round_trips_through_its_choices() {
    let name: FieldName<Vec<Choice<Country>>> =
        crate::path!(Signup => multi).choices(&COUNTRIES).into();
    let adapter = name.adapter().expect("mapped");
    let mut slot = vec![Country::At, Country::Ch];
    let shown = adapter.read(&slot).expect("the slot fits");
    assert_eq!(
        shown.iter().map(Choice::key).collect::<Vec<_>>(),
        ["at", "ch"]
    );
    adapter.write(&mut slot, COUNTRIES.choices()[..1].to_vec());
    assert_eq!(slot, [Country::De]);
    adapter.write(&mut slot, Vec::new());
    assert!(slot.is_empty());
}

#[test]
fn a_segmented_pick_shows_the_fallback_while_nothing_is_stored() {
    let name: FieldName<Choice<Country>> = crate::path!(Signup => strip).choices(&COUNTRIES).into();
    let adapter = name.adapter().expect("mapped");
    let mut slot: Option<Country> = None;
    assert_eq!(adapter.read(&slot).map(|c| c.value), Some(Country::At));
    adapter.write(&mut slot, COUNTRIES.by_key("ch").expect("listed"));
    assert_eq!(slot, Some(Country::Ch));
}

#[test]
fn a_value_no_longer_listed_matches_no_option() {
    static TWO: Choices<Country> = Choices::new(
        &[
            (Country::De, std::borrow::Cow::Borrowed("de")),
            (Country::At, std::borrow::Cow::Borrowed("at")),
        ],
        country_label,
    );
    let name: FieldName<Option<Choice<Country>>> =
        crate::path!(Signup => select).choices(&TWO).into();
    let shown = name
        .adapter()
        .and_then(|adapter| adapter.read(&Some(Country::Ch)))
        .flatten()
        .expect("kept");
    assert_eq!(*shown, Country::Ch);
    assert!(!TWO.choices().contains(&shown));
}

#[test]
fn choices_rebuilt_per_render_compare_equal() {
    assert_eq!(select_name(), select_name());
    let multi = || -> FieldName<Vec<Choice<Country>>> {
        crate::path!(Signup => multi).choices(&COUNTRIES).into()
    };
    assert_eq!(multi(), multi());
}

fn every_pick() -> Element {
    let all = use_store(|| Signup {
        select: Some(Country::Ch),
        native: Some(Country::Ch),
        radio: Some(Country::Ch),
        multi: vec![Country::Ch],
        checks: vec![Country::Ch],
        ..Signup::default()
    });
    expose(all);
    rsx! {
        LiberoProvider {
            Form { value: all,
                Select { label: "Select", options: COUNTRIES.list(), name: crate::path!(Signup => select).choices(&COUNTRIES) }
                NativeSelect { label: "Native", options: COUNTRIES.list(), name: crate::path!(Signup => native).choices(&COUNTRIES) }
                RadioGroup { label: "Radio", options: COUNTRIES.list(), name: crate::path!(Signup => radio).choices(&COUNTRIES) }
                SegmentedControl { label: "Strip", options: COUNTRIES.list(), name: crate::path!(Signup => strip).choices(&COUNTRIES) }
                MultiSelect { label: "Multi", options: COUNTRIES.list(), name: crate::path!(Signup => multi).choices(&COUNTRIES) }
                CheckboxGroup { label: "Checks", options: COUNTRIES.list(), name: crate::path!(Signup => checks).choices(&COUNTRIES) }
            }
        }
    }
}

#[test]
fn every_pick_field_shows_its_stored_choice() {
    let mut dom = VirtualDom::new(every_pick);
    dom.rebuild_in_place();
    let html = settle(&mut dom);
    let tag = |all: &[&str]| {
        html.split('<')
            .any(|tag| all.iter().all(|part| tag.contains(part)))
    };
    for name in ["select", "multi", "checks"] {
        let posted = format!(r#"name="{name}" value="ch""#);
        assert!(html.contains(&posted), "`{name}` does not hold its choice");
    }
    assert!(tag(&["option", r#"value="ch""#, "selected=true"]), "native");
    assert!(
        tag(&["input", r#"name="radio""#, r#"value="ch""#, "checked=true"]),
        "radio"
    );
    assert!(
        tag(&["input", r#"name="strip""#, r#"value="at""#, "checked=true"]),
        "{html}"
    );
}

#[test]
fn a_segmented_control_writes_its_fallback_into_an_empty_store() {
    let mut dom = VirtualDom::new(every_pick);
    dom.rebuild_in_place();
    settle(&mut dom);
    let all = exposed::<Store<Signup>>(&dom);
    assert_eq!(dom.in_runtime(|| all.peek().strip), Some(Country::At));
}

fn required_picks() -> Element {
    let all = use_store(Signup::default);
    let form = use_form();
    expose(form);
    rsx! {
        LiberoProvider {
            Form { form, value: all,
                Select { label: "Select", required: true, options: COUNTRIES.list(), name: crate::path!(Signup => select).choices(&COUNTRIES) }
                NativeSelect { label: "Native", required: true, options: COUNTRIES.list(), name: crate::path!(Signup => native).choices(&COUNTRIES) }
                RadioGroup { label: "Radio", required: true, options: COUNTRIES.list(), name: crate::path!(Signup => radio).choices(&COUNTRIES) }
                MultiSelect { label: "Multi", required: true, options: COUNTRIES.list(), name: crate::path!(Signup => multi).choices(&COUNTRIES) }
                CheckboxGroup { label: "Checks", required: true, options: COUNTRIES.list(), name: crate::path!(Signup => checks).choices(&COUNTRIES) }
            }
        }
    }
}

#[test]
fn every_required_pick_field_blocks_a_submit_while_empty() {
    let mut dom = VirtualDom::new(required_picks);
    dom.rebuild_in_place();
    settle(&mut dom);
    let form = exposed::<FormHandle>(&dom);
    assert!(!dom.in_runtime(|| form.validate()));
    let summary = dom.in_runtime(|| form.scope.summary());
    let labels: Vec<&str> = summary
        .iter()
        .filter_map(|item| item.message.split(':').next())
        .collect();
    assert_eq!(labels, ["Select", "Native", "Radio", "Multi", "Checks"]);
}
