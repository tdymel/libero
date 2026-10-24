//! Todo 4093: every pick and list field bound to a domain type. Empty, a `required` one
//! blocks the submit; holding a value, its `.rules` speak through the field.

use std::convert::Infallible;

use chrono::NaiveDate;
use dioxus::{core::provide_root_context, prelude::*};

use crate::{
    LiberoProvider, choices,
    components::{
        common::Choices,
        form::{
            Autocomplete, CheckboxGroup, DateField, Form, FormHandle, NativeSelect, PinField,
            RadioGroup, Rule, TagsField, use_form,
        },
    },
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Country {
    De,
    Ch,
}

fn country_label(country: &Country) -> String {
    match country {
        Country::De => "Germany",
        Country::Ch => "Switzerland",
    }
    .into()
}

static COUNTRIES: Choices<Country> = choices!(Country, country_label, { De => "de", Ch => "ch" });

#[derive(Clone, Debug, PartialEq)]
struct Code(String);

fn code_text(code: &Code) -> String {
    code.0.clone()
}

fn code(text: String) -> Result<Code, Infallible> {
    Ok(Code(text))
}

fn long(code: &Code) -> bool {
    code.0.len() > 3
}

#[derive(Clone, Debug, PartialEq)]
struct Tags(Vec<String>);

fn tag_list(tags: &Tags) -> Vec<String> {
    tags.0.clone()
}

fn tags(list: Vec<String>) -> Result<Tags, Infallible> {
    Ok(Tags(list))
}

fn several(tags: &Tags) -> bool {
    tags.0.len() > 1
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Day(NaiveDate);

fn day_date(day: &Day) -> NaiveDate {
    day.0
}

fn day(date: NaiveDate) -> Result<Day, Infallible> {
    Ok(Day(date))
}

fn recent(day: &Day) -> bool {
    day.0 >= NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
}

fn domestic(country: &Country) -> bool {
    *country != Country::Ch
}

/// `Vec`, not a slice: a rule over the stored `Vec<Country>`.
#[allow(clippy::ptr_arg)]
fn domestic_only(countries: &Vec<Country>) -> bool {
    !countries.contains(&Country::Ch)
}

/// Per field: one left empty, one holding a value its `.rules` refuse.
#[derive(Clone, PartialEq, Default)]
struct Sweep {
    empty_code: Option<Code>,
    code: Option<Code>,
    empty_tags: Option<Tags>,
    tags: Option<Tags>,
    empty_day: Option<Day>,
    day: Option<Day>,
    empty_country: Option<Country>,
    country: Option<Country>,
    empty_countries: Vec<Country>,
    countries: Vec<Country>,
}

fn filled() -> Sweep {
    Sweep {
        code: Some(Code("12".into())),
        tags: Some(Tags(vec!["one".into()])),
        day: Some(Day(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap())),
        country: Some(Country::Ch),
        countries: vec![Country::Ch],
        ..Sweep::default()
    }
}

/// The form's summary after a submit attempt.
fn summary(app: fn() -> Element) -> Vec<String> {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let form = dom.in_scope(ScopeId::ROOT, consume_context::<FormHandle>);
    assert!(!dom.in_runtime(|| form.validate()), "the form passed");
    dom.in_runtime(|| form.scope.summary())
        .into_iter()
        .map(|item| item.message)
        .collect()
}

/// The empty field asks for a value; the filled one says its rule.
fn assert_blocks(summary: &[String], rule: &str) {
    assert_eq!(summary.len(), 2, "{summary:?}");
    assert!(summary[0].starts_with("Empty: "), "{summary:?}");
    assert_eq!(summary[1], format!("Filled: {rule}"));
}

/// A form over [`filled`], around the fields under test.
#[component]
fn Host(children: Element) -> Element {
    let all = use_store(filled);
    let form = use_form();
    use_hook(|| provide_root_context(form));
    rsx! {
        LiberoProvider {
            Form { form, value: all, {children} }
        }
    }
}

#[test]
fn a_mapped_autocomplete_keeps_its_status() {
    fn app() -> Element {
        rsx! {
            Host {
                Autocomplete::<String> { label: "Empty", required: true, options: Vec::<String>::new(),
                    name: crate::path!(Sweep => empty_code).map(code_text, code) }
                Autocomplete::<String> { label: "Filled", required: true, options: Vec::<String>::new(),
                    name: crate::path!(Sweep => code).map(code_text, code).rules([long.error("Too short.")]) }
            }
        }
    }
    assert_blocks(&summary(app), "Too short.");
}

#[test]
fn a_mapped_tags_field_keeps_its_status() {
    fn app() -> Element {
        rsx! {
            Host {
                TagsField { label: "Empty", required: true,
                    name: crate::path!(Sweep => empty_tags).map(tag_list, tags) }
                TagsField { label: "Filled", required: true,
                    name: crate::path!(Sweep => tags).map(tag_list, tags).rules([several.error("Add another.")]) }
            }
        }
    }
    assert_blocks(&summary(app), "Add another.");
}

#[test]
fn a_mapped_pin_field_keeps_its_status() {
    fn app() -> Element {
        rsx! {
            Host {
                PinField { label: "Empty", required: true,
                    name: crate::path!(Sweep => empty_code).map(code_text, code) }
                PinField { label: "Filled", required: true,
                    name: crate::path!(Sweep => code).map(code_text, code).rules([long.error("Too short.")]) }
            }
        }
    }
    assert_blocks(&summary(app), "Too short.");
}

#[test]
fn a_mapped_date_field_keeps_its_status() {
    fn app() -> Element {
        rsx! {
            Host {
                DateField { label: "Empty", required: true,
                    name: crate::path!(Sweep => empty_day).map(day_date, day) }
                DateField { label: "Filled", required: true,
                    name: crate::path!(Sweep => day).map(day_date, day).rules([recent.error("Too early.")]) }
            }
        }
    }
    assert_blocks(&summary(app), "Too early.");
}

#[test]
fn a_mapped_radio_group_keeps_its_status() {
    fn app() -> Element {
        rsx! {
            Host {
                RadioGroup { label: "Empty", required: true, options: COUNTRIES.list(),
                    name: crate::path!(Sweep => empty_country).choices(&COUNTRIES) }
                RadioGroup { label: "Filled", required: true, options: COUNTRIES.list(),
                    name: crate::path!(Sweep => country).choices(&COUNTRIES).rules([domestic.error("Domestic only.")]) }
            }
        }
    }
    assert_blocks(&summary(app), "Domestic only.");
}

#[test]
fn a_mapped_checkbox_group_keeps_its_status() {
    fn app() -> Element {
        rsx! {
            Host {
                CheckboxGroup { label: "Empty", required: true, options: COUNTRIES.list(),
                    name: crate::path!(Sweep => empty_countries).choices(&COUNTRIES) }
                CheckboxGroup { label: "Filled", required: true, options: COUNTRIES.list(),
                    name: crate::path!(Sweep => countries).choices(&COUNTRIES).rules([domestic_only.error("Domestic only.")]) }
            }
        }
    }
    assert_blocks(&summary(app), "Domestic only.");
}

#[test]
fn a_mapped_native_select_keeps_its_status() {
    fn app() -> Element {
        rsx! {
            Host {
                NativeSelect { label: "Empty", required: true, options: COUNTRIES.list(),
                    name: crate::path!(Sweep => empty_country).choices(&COUNTRIES) }
                NativeSelect { label: "Filled", required: true, options: COUNTRIES.list(),
                    name: crate::path!(Sweep => country).choices(&COUNTRIES).rules([domestic.error("Domestic only.")]) }
            }
        }
    }
    assert_blocks(&summary(app), "Domestic only.");
}
