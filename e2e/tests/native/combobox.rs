//! `Autocomplete`'s narrowed labels as Blitz lays them out (627). The family's
//! keys and clicks are e2e's shared scenarios (`combobox::`).

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::{Autocomplete, Cascader, CascaderOption};

const TRIGGER: &str = "[role=combobox]";

// As Spotlight's (627): a long list, and "G" keeps the first two rows in place.
fn narrowing() -> Element {
    let mut value = use_signal(String::new);
    let options = use_hook(|| {
        let mut all: Vec<String> = ["Getting Started", "Theming", "Select", "MultiSelect"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        all.extend((0..60).map(|n| format!("Item {n}")));
        all
    });
    rsx! {
        Autocomplete {
            label: "Page",
            options,
            value: value(),
            oninput: move |next| value.set(next),
        }
    }
}

// The trigger near a phone window's foot, on a page that scrolls on (1546).
fn low_cascader() -> Element {
    let mut place = use_signal(|| None::<String>);
    let data = vec![
        CascaderOption::new("europe", "Europe").children(vec![
            CascaderOption::new("france", "France")
                .children(vec![CascaderOption::new("paris", "Paris")]),
        ]),
        CascaderOption::new("oceania", "Oceania").children(vec![
            CascaderOption::new("australia", "Australia")
                .children(vec![CascaderOption::new("sydney", "Sydney")]),
        ]),
    ];
    rsx! {
        div { style: "padding: 0 8px; max-width: 320px",
            div { height: "85vh" }
            Cascader {
                label: "Place",
                data,
                value: place(),
                onchange: move |next: Option<String>| place.set(next),
            }
            div { height: "100vh" }
        }
    }
}

/// Todo 1659: the narrow sheet covers the low trigger, and the open scrolls
/// the window (`scroll_viewport_by`) until the trigger clears it.
#[test]
fn a_phone_scrolls_a_low_cascader_above_its_sheet() {
    const SHEET: &str = "[data-slot=dropdown]";
    let mut page = mount(low_cascader);
    page.resize(320, 568);
    page.focus(TRIGGER);
    page.press(Key::ArrowDown);
    let opened = page.wait_for(|page| page.exists("[role=listbox]"));
    assert!(opened, "no sheet: {}", page.tree());
    let clear = |page: &e2e::native::Page| {
        let (_, top, _, height) = page.rect(TRIGGER);
        top + height <= page.rect(SHEET).1 + 0.5
    };
    let cleared = page.wait_for(clear);
    let sheet = page.rect(SHEET);
    assert!(
        cleared,
        "trigger {:?} under sheet {sheet:?}, window scrolled {:?}",
        page.rect(TRIGGER),
        page.viewport_scroll()
    );
    assert!(
        (sheet.1 + sheet.3 - 568.0).abs() <= 1.0,
        "the sheet {sheet:?} is not at the window's foot"
    );
}

#[test]
fn a_narrowed_autocomplete_keeps_each_label_on_one_line() {
    let mut page = mount(narrowing);
    page.focus(TRIGGER);
    page.press(Key::ArrowDown);
    assert!(
        page.query_all("[role=option]").len() > 60,
        "{}",
        page.tree()
    );
    page.press_before_layout(Key::Character("G".into()));
    assert_eq!(page.query_all("[role=option]").len(), 2, "{}", page.tree());
    assert_eq!(page.wrapped_text("[role=option]"), Vec::<String>::new());
}
