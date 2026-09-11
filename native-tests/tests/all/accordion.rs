//! `Accordion`: the arrows move focus and never toggle, Enter and Space toggle,
//! a closed panel hides, and focus returns out of a panel that closes.

use std::time::Duration;

use dioxus::prelude::*;
use libero::components::{Accordion, AccordionOpen, Button, OptionList, Options, Text};
use native_tests::{Key, Page, mount};

#[derive(Clone, PartialEq, Options)]
enum Step {
    Shipping,
    Payment,
    Review,
}

fn app() -> Element {
    let mut open = use_signal(|| AccordionOpen::One(None::<Step>));
    rsx! {
        Accordion {
            id: "checkout",
            open: open(),
            onchange: move |next| open.set(next),
            options: OptionList::from_options().disabling(|s| *s == Step::Payment),
            panel: move |step: Step| match step {
                Step::Shipping => rsx! {
                    Text { "Shipping is calculated at checkout." }
                    Button {
                        id: "continue",
                        onclick: move |_| open.set(AccordionOpen::One(Some(Step::Review))),
                        "Continue"
                    }
                },
                Step::Payment => rsx! { Text { "Card details." } },
                Step::Review => rsx! { Text { "Check your order." } },
            },
        }
    }
}

fn trigger(index: usize) -> String {
    format!("#checkout-trigger-{index}")
}

fn expanded(page: &Page, index: usize) -> bool {
    page.attr(&trigger(index), "aria-expanded").as_deref() == Some("true")
}

fn region_hidden(page: &Page, index: usize) -> bool {
    page.computed(&format!("#checkout-region-{index}"), "visibility") == "hidden"
}

/// Ends the open or close: the CSS transition, then libero's own timer.
fn finish(page: &mut Page) {
    page.advance(1.0);
    page.wait(Duration::from_millis(400));
}

#[test]
fn the_arrows_move_focus_skip_the_disabled_and_never_toggle() {
    let mut page = mount(app);
    page.focus(&trigger(0));
    for (key, to) in [
        (Key::ArrowDown, 2),
        (Key::ArrowDown, 0),
        (Key::ArrowUp, 2),
        (Key::Home, 0),
        (Key::End, 2),
    ] {
        page.press(key.clone());
        assert!(
            page.is_focused(&trigger(to)),
            "{key:?} went to {}, not {to}",
            page.focus_owner()
        );
    }
    assert!(!expanded(&page, 2), "an arrow toggled Review");
}

#[test]
fn enter_opens_space_closes_and_a_closed_panel_hides() {
    let mut page = mount(app);
    for index in 0..3 {
        assert!(region_hidden(&page, index), "closed panel {index} shows");
    }
    page.focus(&trigger(1));
    page.press(Key::Enter);
    assert!(!expanded(&page, 1), "Enter opened the disabled Payment");

    page.focus(&trigger(0));
    page.press(Key::Enter);
    finish(&mut page);
    assert!(expanded(&page, 0), "Enter did not open Shipping");
    assert!(!region_hidden(&page, 0), "the open panel is hidden");
    assert!(page.exists("#continue"), "{}", page.tree());

    page.press(Key::Character(" ".into()));
    finish(&mut page);
    assert!(!expanded(&page, 0), "Space did not close Shipping");
    assert!(!page.exists("#continue"), "the closed panel stayed mounted");
    assert!(region_hidden(&page, 0), "the closed root shows");
}

#[test]
fn a_click_toggles_a_section() {
    let mut page = mount(app);
    page.click(&trigger(2));
    finish(&mut page);
    assert!(expanded(&page, 2), "the click did not open Review");
    page.click(&trigger(2));
    finish(&mut page);
    assert!(!expanded(&page, 2), "the second click did not close Review");
}

#[test]
fn focus_returns_to_the_trigger_of_a_panel_that_closes_around_it() {
    let mut page = mount(app);
    page.focus(&trigger(0));
    page.press(Key::Enter);
    finish(&mut page);
    page.focus("#continue");
    page.press(Key::Enter);
    finish(&mut page);
    assert!(expanded(&page, 2), "Continue did not open Review");
    assert!(
        page.is_focused(&trigger(0)),
        "focus is on {}",
        page.focus_owner()
    );
}
