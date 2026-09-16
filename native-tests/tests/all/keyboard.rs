//! `KeyboardApi` natively: a press heard as it bubbles out of `LiberoProvider`.

use std::{cell::Cell, rc::Rc, time::Duration};

use dioxus::prelude::*;
use libero::{
    components::{Button, HoverCard},
    hooks::{ModalScope, use_modal},
    platform::{KeySubscription, keyboard},
};
use native_tests::{Key, Modifiers, mount};

thread_local! {
    static HEARD: Cell<u32> = const { Cell::new(0) };
    static HEARD_UNFILTERED: Cell<u32> = const { Cell::new(0) };
}

fn hotkeys() -> Element {
    use_hook(|| {
        let api = keyboard().expect("no native keyboard");
        let filtered = api.on_key(Box::new(|chord| {
            let hit = chord.key == Key::Character("k".into()) && chord.modifiers.ctrl();
            if hit {
                HEARD.set(HEARD.get() + 1);
            }
            hit
        }));
        let unfiltered = api.on_key_unfiltered(Box::new(|chord| {
            if chord.key == Key::Character("k".into()) {
                HEARD_UNFILTERED.set(HEARD_UNFILTERED.get() + 1);
            }
            false
        }));
        Rc::new((filtered, unfiltered)) as Rc<(Box<dyn KeySubscription>, Box<dyn KeySubscription>)>
    });
    rsx! {
        Button { id: "page", "Page" }
        input { id: "text", r#type: "text" }
        input { id: "check", r#type: "checkbox" }
        p { id: "prose", "Nothing to focus" }
    }
}

fn ctrl_k(page: &mut native_tests::Page) {
    page.press_with(Key::Character("k".into()), Modifiers::CONTROL);
}

#[test]
fn a_hotkey_is_heard_from_a_focused_button() {
    let mut page = mount(hotkeys);
    page.focus("#page");
    ctrl_k(&mut page);
    assert_eq!(HEARD.get(), 1);
    assert_eq!(HEARD_UNFILTERED.get(), 1);
}

/// Todo 702: with a second document live on the thread, a press reaches only
/// its own document's subscribers.
#[test]
fn a_press_reaches_only_its_own_documents_hotkeys() {
    let mut older = mount(hotkeys);
    let _newer = mount(hotkeys);
    older.focus("#page");
    ctrl_k(&mut older);
    assert_eq!(HEARD.get(), 1);
    assert_eq!(HEARD_UNFILTERED.get(), 1);
}

#[test]
fn a_text_field_hides_a_press_from_the_filtered_path_only() {
    let mut page = mount(hotkeys);
    page.focus("#text");
    ctrl_k(&mut page);
    assert_eq!(HEARD.get(), 0, "the hotkey fired from a text field");
    assert_eq!(HEARD_UNFILTERED.get(), 1);

    page.focus("#check");
    ctrl_k(&mut page);
    assert_eq!(HEARD.get(), 1, "a checkbox is not text entry");
}

#[test]
fn a_hotkey_is_heard_after_a_click_on_nothing_focusable() {
    let mut page = mount(hotkeys);
    page.click("#prose");
    ctrl_k(&mut page);
    assert_eq!(
        HEARD.get(),
        1,
        "focus sat on {} and the press was lost",
        page.focus_owner()
    );
}

#[test]
fn a_dropped_subscription_hears_nothing() {
    fn app() -> Element {
        let mut on = use_signal(|| true);
        rsx! {
            Button { id: "off", onclick: move |_| on.set(false), "Off" }
            if on() {
                Hotkey {}
            }
        }
    }
    #[component]
    fn Hotkey() -> Element {
        let subscription = use_hook(|| {
            Rc::new(keyboard().unwrap().on_key(Box::new(|_| {
                HEARD.set(HEARD.get() + 1);
                false
            })))
        });
        let _ = subscription;
        rsx! {}
    }

    let mut page = mount(app);
    page.focus("#off");
    page.press(Key::Character("a".into()));
    assert_eq!(HEARD.get(), 1);
    page.click("#off");
    page.focus("#off");
    page.press(Key::Character("a".into()));
    assert_eq!(HEARD.get(), 1, "a dropped subscription still heard a press");
}

const CARD: &str = "[role=dialog][aria-label='Ada Lovelace']";

fn card_in_modal() -> Element {
    let modal = use_modal(|_: ModalScope<()>| {
        rsx! {
            // `Modal` leaves hits to its content, as `Dialog` takes them.
            div { id: "modal-body", pointer_events: "auto",
                Button { id: "inside", "Inside" }
                HoverCard {
                    aria_label: "Ada Lovelace",
                    open_delay: 10,
                    close_delay: 10,
                    content: rsx! { "Profile" },
                    Button { id: "trigger", "Ada" }
                }
            }
        }
    });
    rsx! {
        Button {
            id: "open",
            onclick: move |_| {
                modal.open();
            },
            "Open"
        }
    }
}

/// One Escape closes one layer: the pointer-opened card, then the `Modal`.
#[test]
fn escape_closes_a_card_in_a_modal_before_the_modal() {
    let mut page = mount(card_in_modal);
    page.click("#open");
    assert!(page.exists("#modal-body"), "the modal did not open");
    page.focus("#inside");
    page.hover("#trigger");
    page.wait(Duration::from_millis(100));
    assert!(
        page.exists(CARD),
        "hovering did not open it:\n{}",
        page.tree()
    );

    page.press(Key::Escape);
    assert!(
        !page.exists(CARD),
        "Escape did not close the card:\n{}",
        page.tree()
    );
    assert!(page.exists("#modal-body"), "Escape closed the modal too");

    page.focus("#inside");
    page.press(Key::Escape);
    assert!(
        !page.exists("#modal-body"),
        "the second Escape did not close the modal"
    );
}

fn modal_over_card() -> Element {
    let modal = use_modal(|_: ModalScope<()>| {
        rsx! {
            div { id: "modal-body",
                Button { id: "inside", "Inside" }
            }
        }
    });
    rsx! {
        Button {
            id: "open",
            onclick: move |_| {
                modal.open();
            },
            "Open"
        }
        HoverCard {
            aria_label: "Ada Lovelace",
            open_delay: 10,
            close_delay: 10,
            content: rsx! { "Profile" },
            Button { id: "trigger", "Ada" }
        }
    }
}

/// A `Modal` opened over a pointer-opened card is the top layer: Escape closes
/// it alone, and the next Escape the card.
#[test]
fn escape_closes_a_modal_over_a_card_before_the_card() {
    let mut page = mount(modal_over_card);
    page.hover("#trigger");
    page.wait(Duration::from_millis(100));
    assert!(
        page.exists(CARD),
        "hovering did not open it:\n{}",
        page.tree()
    );
    page.focus("#open");
    page.press(Key::Enter);
    assert!(page.exists("#modal-body"), "Enter did not open the modal");

    page.press(Key::Escape);
    assert!(
        !page.exists("#modal-body"),
        "Escape did not close the modal"
    );
    assert!(
        page.exists(CARD),
        "Escape closed the card under the modal too"
    );

    page.focus("#open");
    page.press(Key::Escape);
    assert!(
        !page.exists(CARD),
        "the second Escape did not close the card"
    );
}
