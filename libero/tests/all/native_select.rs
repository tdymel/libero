//! `NativeSelect`: the platform `<select>`, its options and its placeholder.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{NativeSelect, Options},
};

#[derive(Clone, PartialEq, Options)]
enum Pick {
    First,
    Second,
}

#[test]
fn select_renders_its_options_and_marks_the_current_one() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect { value: Pick::Second, onchange: move |_| {} }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("<select"));
    assert_eq!(body.matches("<option").count(), 2);
    // The selection is on the `<option>`, not a `value` on the `<select>`:
    // that is what SSR can express, and it needs no second render to appear.
    // Each option posts `Options::value` - the variant's name - not its index.
    assert!(body.contains("<option value=\"First\">"));
    assert!(body.contains("<option value=\"Second\" selected"));
    assert!(!body.contains("<select value="));
}

/// `value: None` is a real state - the field has not been filled in yet - so
/// it selects an entry no one can pick rather than silently taking the first.
#[test]
fn a_select_without_a_value_shows_its_placeholder() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect {
                    value: None::<Pick>,
                    placeholder: "Choose one",
                    // Annotated: with `value: None` there is nothing else for
                    // `T` to be inferred from.
                    onchange: move |_: Pick| {},
                }
            }
        }
    }

    let body = body(&render(app));

    assert!(body.contains("Choose one"));
    assert_eq!(body.matches("<option").count(), 3);
    // Disabled and hidden, so it cannot be picked back once a value is set.
    let placeholder = attributes_of(&body, "option");
    assert!(placeholder.contains_key("disabled"), "{placeholder:?}");
    assert!(placeholder.contains_key("hidden"), "{placeholder:?}");
    assert!(placeholder.contains_key("selected"), "{placeholder:?}");
    assert!(!body.contains("<option value=\"First\" selected"));
}

#[test]
fn a_disabled_select_renders_the_attribute() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                NativeSelect {
                    value: Pick::First,
                    disabled: true,
                    onchange: move |_| {},
                }
            }
        }
    }

    let select = attributes_of(&body(&render(app)), "select");

    assert!(select.contains_key("disabled"), "{select:?}");
}
