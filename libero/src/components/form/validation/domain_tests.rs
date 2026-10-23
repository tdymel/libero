//! A field mapped to a domain type: the parse states, the typed text, and the
//! adapter's equality. SSR fires no input, so a probe types through the binding.

use std::rc::Rc;

use dioxus::{
    core::{Runtime, current_scope_id, provide_root_context},
    prelude::*,
};

use crate::{
    LiberoProvider,
    components::form::{
        DateField, FieldError, FieldName, FieldPath, Form, FormHandle, FormScope, Rule, TextField,
        Typed, checked, use_bound, use_form,
    },
    localization::Localization,
    use_localization_handle,
};

#[derive(Clone, Debug, PartialEq)]
struct Iban(String);

#[derive(Debug)]
enum IbanError {
    Format,
    Foreign,
}

impl Iban {
    fn new(text: String) -> Result<Iban, IbanError> {
        let text: String = text.split_whitespace().collect::<String>().to_uppercase();
        let letters = text.chars().take(2).all(|c| c.is_ascii_alphabetic());
        match text.len() == 22 && letters {
            true => Ok(Iban(text)),
            false => Err(IbanError::Format),
        }
    }

    /// Grouped by four, as printed.
    fn text(&self) -> String {
        let chars: Vec<char> = self.0.chars().collect();
        chars
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn domestic(iban: &Iban) -> Result<(), IbanError> {
    match iban.0.starts_with("GB") {
        true => Err(IbanError::Foreign),
        false => Ok(()),
    }
}

fn iban_message(error: &IbanError, locale: &Localization) -> String {
    match (error, locale.lang) {
        (IbanError::Format, "de") => "Keine IBAN.".into(),
        (IbanError::Format, _) => "Not an IBAN.".into(),
        (IbanError::Foreign, _) => "Use a German account.".into(),
    }
}

#[derive(Clone, PartialEq, Default)]
struct Payee {
    iban: Option<Iban>,
    since: Option<chrono::NaiveDate>,
}

/// Reads the IBAN as there: the form runs it only once the field holds one.
fn not_austrian(payee: &Payee) -> bool {
    !payee
        .iban
        .as_ref()
        .expect("a guarded rule saw no IBAN")
        .0
        .starts_with("AT")
}

fn iban_path() -> FieldPath<Payee, Option<Iban>> {
    crate::path!(Payee => iban)
}

fn iban_name() -> FieldName<String> {
    iban_path()
        .map(Iban::text, Iban::new)
        .message(iban_message)
        .rules([checked(domestic)])
        .into()
}

fn expose<T: Clone + 'static>(value: T) {
    use_hook(|| provide_root_context(value));
}

fn exposed<T: Clone + 'static>(dom: &VirtualDom) -> T {
    dom.in_scope(ScopeId::ROOT, consume_context::<T>)
}

#[derive(Clone)]
struct Typist(Rc<dyn Fn(String)>);

#[derive(Clone, Copy)]
struct ProbeScope(ScopeId);

/// Types into the field's binding, and shows the text and status it gets.
#[component]
fn Probe() -> Element {
    let bound = use_bound(&iban_name(), false);
    let emit = bound.emit(None).expect("a bound field");
    expose(Typist(Rc::new(emit)));
    expose(ProbeScope(current_scope_id()));
    expose(use_context::<FormScope>());
    let shown = bound.value().unwrap_or_default();
    let status = bound
        .mapped
        .as_ref()
        .and_then(|mapped| mapped.status.clone())
        .and_then(|status| status.message().map(str::to_string))
        .unwrap_or_default();
    let shown = format!("[{shown}]");
    let status = format!("[{status}]");
    rsx! {
        span { id: "shown", {shown} }
        span { id: "status", {status} }
    }
}

fn app() -> Element {
    let payee = use_store(Payee::default);
    let probe = use_signal(|| true);
    let form = use_form();
    expose(payee);
    expose(probe);
    expose(form);
    expose(use_localization_handle());
    rsx! {
        Form {
            form,
            value: payee,
            validate: [not_austrian.error("No Austrian accounts.").on([iban_path()])],
            if probe() {
                Probe {}
                DateField { label: "Since", name: crate::path!(Payee => since) }
            }
            TextField { label: "IBAN", required: true, name: iban_name() }
        }
    }
}

fn mount() -> VirtualDom {
    let mut dom = VirtualDom::new(|| rsx! { LiberoProvider { Wrapper {} } });
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom
}

#[component]
fn Wrapper() -> Element {
    app()
}

fn settle(dom: &mut VirtualDom) -> String {
    dom.process_events();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dioxus_ssr::render(dom)
}

fn type_text(dom: &mut VirtualDom, text: &str) -> String {
    let Typist(emit) = exposed(dom);
    dom.in_runtime(|| emit(text.to_string()));
    settle(dom)
}

fn stored(dom: &VirtualDom) -> Option<Iban> {
    let payee = exposed::<Store<Payee>>(dom);
    dom.in_runtime(|| payee.peek().iban.clone())
}

fn scope(dom: &VirtualDom) -> FormScope {
    exposed(dom)
}

/// What the probe shows: its text and its status.
fn probe(html: &str) -> (String, String) {
    let span = |id: &str| {
        let open = format!(r#"id="{id}">["#);
        let start = html.find(&open).expect("probe rendered") + open.len();
        let end = start + html[start..].find("]</span>").expect("closed");
        html[start..end].to_string()
    };
    (span("shown"), span("status"))
}

/// A blur, as the field's `focusout` does it.
fn blur(dom: &mut VirtualDom) -> String {
    let mut scope = scope(dom);
    let ProbeScope(probe) = exposed(dom);
    dom.in_runtime(|| {
        scope.touch("iban");
        if scope.settle_draft("iban") {
            Runtime::current().needs_update(probe);
        }
    });
    settle(dom)
}

#[test]
fn empty_text_is_empty_and_stores_nothing() {
    let mut dom = mount();
    let html = type_text(&mut dom, "   ");
    assert_eq!(stored(&dom), None);
    assert_eq!(probe(&html), ("   ".into(), String::new()));
    assert!(
        dom.in_runtime(|| scope(&dom).has_errors()),
        "a required empty field passes"
    );
}

#[test]
fn invalid_text_resets_the_store_keeps_the_text_and_blocks_the_submit() {
    let mut dom = mount();
    type_text(&mut dom, "DE89 3704 0044 0532 0130 00");
    assert!(stored(&dom).is_some());

    let html = type_text(&mut dom, "DE89 12");
    assert_eq!(stored(&dom), None, "the invalid text left a value behind");
    assert_eq!(probe(&html), ("DE89 12".into(), "Not an IBAN.".into()));
    assert!(
        html.contains(r#"value="DE89 12""#),
        "the field lost the typed text"
    );
    assert!(dom.in_runtime(|| scope(&dom).has_errors()));

    let mut form = scope(&dom);
    dom.in_runtime(|| form.submit());
    let html = settle(&mut dom);
    assert!(html.contains("Not an IBAN."));
    let summary = dom.in_runtime(|| form.summary());
    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].message, "IBAN: Not an IBAN.");
}

#[test]
fn a_valid_value_that_fails_a_rule_is_stored_and_shows_the_rule() {
    let mut dom = mount();
    let html = type_text(&mut dom, "GB29 NWBK 6016 1331 9268 19");
    assert_eq!(stored(&dom), Some(Iban("GB29NWBK60161331926819".into())));
    assert_eq!(probe(&html).1, "Use a German account.");
    assert!(dom.in_runtime(|| scope(&dom).has_errors()));
}

#[test]
fn a_valid_value_is_stored_and_shows_nothing() {
    let mut dom = mount();
    let html = type_text(&mut dom, "DE89 3704 0044 0532 0130 00");
    assert_eq!(stored(&dom), Some(Iban("DE89370400440532013000".into())));
    assert_eq!(probe(&html).1, "");
    assert!(!dom.in_runtime(|| scope(&dom).has_errors()));
}

#[test]
fn typed_text_is_not_reformatted_until_a_blur() {
    let mut dom = mount();
    let html = type_text(&mut dom, "de89370400440532013000");
    assert!(stored(&dom).is_some());
    assert_eq!(
        probe(&html).0,
        "de89370400440532013000",
        "reformatted mid-typing"
    );

    let html = blur(&mut dom);
    assert_eq!(probe(&html).0, "DE89 3704 0044 0532 0130 00");
}

#[test]
fn invalid_text_stays_after_a_blur_and_an_unmount() {
    let mut dom = mount();
    type_text(&mut dom, "DE89 12");
    let html = blur(&mut dom);
    assert_eq!(probe(&html).0, "DE89 12");

    let mut shown = exposed::<Signal<bool>>(&dom);
    dom.in_runtime(|| shown.set(false));
    settle(&mut dom);
    dom.in_runtime(|| shown.set(true));
    let html = settle(&mut dom);
    assert_eq!(probe(&html), ("DE89 12".into(), "Not an IBAN.".into()));
}

#[test]
fn a_store_change_from_elsewhere_or_a_reset_drops_the_typed_text() {
    let mut dom = mount();
    type_text(&mut dom, "DE89 12");
    let mut payee = exposed::<Store<Payee>>(&dom);
    dom.in_runtime(|| payee.write().iban = Some(Iban("DE89370400440532013000".into())));
    let html = settle(&mut dom);
    assert_eq!(
        probe(&html),
        ("DE89 3704 0044 0532 0130 00".into(), String::new())
    );

    type_text(&mut dom, "DE89 12");
    let mut form = scope(&dom);
    dom.in_runtime(|| form.reset());
    assert!(
        dom.in_runtime(|| form.draft("iban")).is_none(),
        "a reset kept the draft"
    );
}

#[test]
fn a_language_switch_rewords_the_parse_error() {
    let mut dom = mount();
    type_text(&mut dom, "DE89 12");
    let handle = exposed::<crate::LocalizationHandle>(&dom);
    dom.in_runtime(|| handle.set(&Localization::GERMAN));
    let html = settle(&mut dom);
    assert_eq!(probe(&html).1, "Keine IBAN.");
}

#[test]
fn a_mapped_name_rebuilt_per_render_is_equal() {
    assert!(iban_name() == iban_name());

    let other: FieldName<String> = iban_path()
        .map(Iban::text, Iban::new)
        .message(|_: &IbanError, _: &Localization| "Other".to_string())
        .into();
    assert!(iban_name() != other, "another message fn compared equal");

    let min = 22;
    let capturing = || -> FieldName<String> {
        iban_path()
            .map(Iban::text, move |text: String| match text.len() >= min {
                true => Iban::new(text),
                false => Err(IbanError::Format),
            })
            .message(iban_message)
            .into()
    };
    assert!(
        capturing() != capturing(),
        "a capturing parse compared equal"
    );

    let plain: FieldName<String> = FieldPath::<Payee, String>::new("iban").into();
    assert!(iban_name() != plain);
}

#[test]
fn a_composite_rule_runs_only_on_a_parsed_value() {
    let mut dom = mount();
    let mut form = scope(&dom);
    dom.in_runtime(|| form.submit());
    let html = settle(&mut dom);
    assert!(!html.contains("No Austrian accounts."));

    let html = type_text(&mut dom, "AT61 1904 30");
    assert!(html.contains("Not an IBAN."));
    assert!(
        !html.contains("No Austrian accounts."),
        "a second error on invalid text"
    );

    let html = type_text(&mut dom, "AT61 1904 3002 3457 3201 01");
    assert!(html.contains("No Austrian accounts."));
}

#[test]
fn the_error_summary_rewords_on_a_language_switch() {
    let mut dom = mount();
    type_text(&mut dom, "DE89 12");
    let form = exposed::<FormHandle>(&dom);
    assert!(!dom.in_runtime(|| form.validate()));
    let html = settle(&mut dom);
    assert!(html.contains("IBAN: Not an IBAN."), "{html}");

    let handle = exposed::<crate::LocalizationHandle>(&dom);
    dom.in_runtime(|| handle.set(&Localization::GERMAN));
    let html = settle(&mut dom);
    assert!(
        html.contains("IBAN: Keine IBAN."),
        "the summary kept the old words"
    );
}

#[test]
fn a_date_fields_refused_text_survives_an_unmount_and_blocks_the_submit() {
    let mut dom = mount();
    let mut shown = exposed::<Signal<bool>>(&dom);
    dom.in_runtime(|| shown.set(false));
    settle(&mut dom);
    let mut form = scope(&dom);
    dom.in_runtime(|| {
        let error: Rc<dyn FieldError> =
            Rc::new(|locale: &Localization| locale.date.invalid_date.to_string());
        form.keep_typed(
            "since",
            Typed {
                text: "32.13.".into(),
                error,
            },
        );
    });
    dom.in_runtime(|| shown.set(true));
    settle(&mut dom);
    let mut form = scope(&dom);
    dom.in_runtime(|| form.submit());
    let html = settle(&mut dom);
    assert!(
        html.contains(r#"value="32.13.""#),
        "the refused text is gone"
    );
    assert!(html.contains(Localization::ENGLISH.date.invalid_date));
}

/// A word with no blanks: the domain type every text-like field below maps to.
#[derive(Clone, Debug, PartialEq)]
struct Word(String);

struct Blank;

impl FieldError for Blank {
    fn message(&self, _: &Localization) -> String {
        "No blanks.".into()
    }
}

impl std::str::FromStr for Word {
    type Err = Blank;

    fn from_str(text: &str) -> Result<Word, Blank> {
        match text.contains(' ') {
            true => Err(Blank),
            false => Ok(Word(text.to_string())),
        }
    }
}

impl std::fmt::Display for Word {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Count(f64);

#[derive(Clone, Debug, PartialEq)]
struct Words(Vec<String>);

fn words(list: Vec<String>) -> Result<Words, Blank> {
    Ok(Words(list))
}

#[derive(Clone, PartialEq, Default)]
struct Everything {
    text: Option<Word>,
    area: Option<Word>,
    secret: Option<Word>,
    auto: Option<Word>,
    number: Option<Count>,
    select: Option<Word>,
    native: Option<Word>,
    radio: Option<Word>,
    multi: Option<Words>,
    checks: Option<Words>,
    tags: Option<Words>,
}

fn word(text: &str) -> Option<Word> {
    Some(Word(text.into()))
}

fn listed() -> Option<Words> {
    Some(Words(vec!["Bonn".into()]))
}

fn every_field() -> Element {
    let all = use_store(|| Everything {
        text: word("text-a"),
        area: word("area-b"),
        secret: word("secret-c"),
        auto: word("auto-d"),
        number: Some(Count(41.5)),
        select: word("Bonn"),
        native: word("Bonn"),
        radio: word("Bonn"),
        multi: listed(),
        checks: listed(),
        tags: listed(),
    });
    let cities = || vec!["Berlin".to_string(), "Bonn".to_string()];
    rsx! {
        LiberoProvider {
            Form { value: all,
                TextField { label: "Text", name: crate::path!(Everything => text).parsed() }
                crate::components::form::Textarea { label: "Area", name: crate::path!(Everything => area).parsed() }
                crate::components::form::PasswordField { label: "Secret", name: crate::path!(Everything => secret).parsed() }
                crate::components::form::Autocomplete::<String> { label: "Auto", options: cities(), name: crate::path!(Everything => auto).parsed() }
                crate::components::form::NumberField::<f64> {
                    label: "Number",
                    name: crate::path!(Everything => number).map(|count: &Count| count.0, |n: f64| Ok::<_, Blank>(Count(n))),
                }
                crate::components::form::Select::<String> { label: "Select", options: cities(), name: crate::path!(Everything => select).parsed() }
                crate::components::form::NativeSelect::<String> { label: "Native", options: cities(), name: crate::path!(Everything => native).parsed() }
                crate::components::form::RadioGroup::<String> { label: "Radio", options: cities(), name: crate::path!(Everything => radio).parsed() }
                crate::components::form::MultiSelect::<String> {
                    label: "Multi",
                    options: cities(),
                    name: crate::path!(Everything => multi).map(|list: &Words| list.0.clone(), words),
                }
                crate::components::form::CheckboxGroup::<String> {
                    label: "Checks",
                    options: cities(),
                    name: crate::path!(Everything => checks).map(|list: &Words| list.0.clone(), words),
                }
                crate::components::form::TagsField {
                    label: "Tags",
                    name: crate::path!(Everything => tags).map(|list: &Words| list.0.clone(), words),
                }
            }
        }
    }
}

#[test]
fn every_wire_typed_field_shows_a_mapped_value() {
    let mut dom = VirtualDom::new(every_field);
    dom.rebuild_in_place();
    let html = settle(&mut dom);
    for text in ["text-a", "area-b", "secret-c", "auto-d", "41.5"] {
        assert!(html.contains(text), "`{text}` is not shown");
    }
    for name in ["select", "multi", "checks", "tags"] {
        let posted = format!(r#"name="{name}" value="Bonn""#);
        assert!(
            html.contains(&posted),
            "`{name}` does not hold its mapped value"
        );
    }
    let tag = |all: &[&str]| {
        html.split('<')
            .any(|tag| all.iter().all(|part| tag.contains(part)))
    };
    assert!(tag(&["option", r#"value="Bonn""#, "selected=true"]));
    assert!(tag(&[
        "input",
        r#"name="radio""#,
        r#"value="Bonn""#,
        "checked=true"
    ]));
}
