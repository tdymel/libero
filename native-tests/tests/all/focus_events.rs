//! Focus moves Blitz makes without a focus event (Tab, libero's `focus()`),
//! served to the hooks that need them by `platform::silent_focus` (todo 468 N6).

use dioxus::prelude::*;
use libero::{
    components::{
        Autocomplete, Button, Carousel, ColorCode, ColorField, FloatingWindowOptions, HoverCard,
        Menu, MenuItem, Options, Rule, Select, TextField, Tooltip, not_empty, use_menu,
    },
    hooks::{use_element, use_floating_window},
    platform::ElementApi,
};
use native_tests::{Key, Page, mount};

const TRIGGER: &str = "[aria-haspopup]";

fn menu_app() -> Element {
    let menu = use_menu();
    rsx! {
        Menu {
            state: menu,
            items: vec![
                MenuItem::new("Copy").onselect(|_| {}).into(),
                MenuItem::new("Paste").onselect(|_| {}).into(),
            ],
            Button { attributes: menu.a11y_attributes(), "Actions" }
        }
        Button { id: "after", "After" }
    }
}

fn expanded(page: &Page) -> bool {
    page.attr(TRIGGER, "aria-expanded")
        .is_some_and(|v| v == "true")
}

#[test]
fn tab_out_of_an_open_menu_closes_it() {
    let mut page = mount(menu_app);
    page.focus(TRIGGER);
    page.press(Key::Enter);
    assert!(expanded(&page), "Enter did not open it:\n{}", page.tree());

    page.tab();
    assert!(
        !expanded(&page),
        "Tab left it open, focus on {}:\n{}",
        page.focus_owner(),
        page.tree()
    );
}

fn focusing_menu_app() -> Element {
    let menu = use_menu();
    let search = use_element();
    rsx! {
        Menu {
            state: menu,
            close_on_select: false,
            items: vec![
                // From a task: Blitz focuses a clicked element after its handlers.
                MenuItem::new("Find").onselect(move |_| {
                    spawn(async move {
                        let _ = search.focus();
                    });
                }).into(),
            ],
            Button { attributes: menu.a11y_attributes(), "Actions" }
        }
        input { id: "search", onmounted: search.mount() }
    }
}

/// `use_dismiss` closes on focus leaving; libero's own `focus()` fires no
/// `focusout` natively.
#[test]
fn focus_moved_out_of_an_open_menu_by_code_closes_it() {
    let mut page = mount(focusing_menu_app);
    page.focus(TRIGGER);
    page.press(Key::Enter);
    assert!(expanded(&page), "Enter did not open it:\n{}", page.tree());

    page.press(Key::Enter);
    assert!(
        page.is_focused("#search"),
        "focus is on {}",
        page.focus_owner()
    );
    assert!(
        !expanded(&page),
        "focus left and it stayed open:\n{}",
        page.tree()
    );
}

fn focusing_button_app() -> Element {
    let search = use_element();
    rsx! {
        button {
            id: "go",
            onclick: move |_| {
                let _ = search.focus();
            },
            "Search"
        }
        input { id: "search", onmounted: search.mount() }
    }
}

/// Blitz focuses a clicked element after its handlers (the web: before), so
/// a `focus()` from a click handler has to be made again after that move.
#[test]
fn focus_from_a_click_handler_wins_over_the_click() {
    let mut page = mount(focusing_button_app);
    page.click("#go");
    assert!(
        page.is_focused("#search"),
        "focus is on {}",
        page.focus_owner()
    );

    page.focus("#go");
    page.press(Key::Enter);
    assert!(
        page.is_focused("#search"),
        "focus is on {}",
        page.focus_owner()
    );
}

fn field_app() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        TextField { label: "Email", value: "", validate: [not_empty.error("Required")] }
        Button { id: "after", "After" }
    }
}

/// `use_field` marks a field touched on `focusout`; its rule shows from then on.
#[test]
fn tabbing_through_a_field_reveals_its_rule() {
    let mut page = mount(field_app);
    page.click("#before");
    page.tab();
    assert!(
        page.is_focused("input"),
        "focus is on {}",
        page.focus_owner()
    );
    page.tab();
    assert_eq!(
        page.attr("input", "aria-invalid").as_deref(),
        Some("true"),
        "Tab out did not touch it:\n{}",
        page.tree()
    );
}

fn card_app() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        HoverCard {
            aria_label: "Ada Lovelace",
            content: rsx! { Button { id: "inside", "Profile" } },
            Button { id: "trigger", "Ada Lovelace" }
        }
        Button { id: "after", "After" }
    }
}

#[test]
fn tabbing_onto_a_hover_card_trigger_opens_it_and_tabbing_on_closes_it() {
    let mut page = mount(card_app);
    page.click("#before");
    page.tab();
    assert!(
        page.is_focused("#trigger"),
        "focus is on {}",
        page.focus_owner()
    );
    assert!(
        page.exists("[role=dialog]"),
        "Tab onto the trigger did not open it"
    );

    page.tab();
    assert!(
        page.is_focused("#inside"),
        "focus is on {}",
        page.focus_owner()
    );
    page.tab();
    assert!(
        !page.exists("[role=dialog]"),
        "Tab out left it open, focus on {}",
        page.focus_owner()
    );
    assert!(
        page.is_focused("#after"),
        "focus is on {}",
        page.focus_owner()
    );
}

#[test]
fn escape_from_inside_a_keyboard_opened_card_returns_to_the_trigger_and_stays_closed() {
    let mut page = mount(card_app);
    page.click("#before");
    page.tab();
    page.tab();
    assert!(
        page.is_focused("#inside"),
        "focus is on {}",
        page.focus_owner()
    );

    page.press(Key::Escape);
    assert!(
        page.is_focused("#trigger"),
        "focus is on {}",
        page.focus_owner()
    );
    assert!(
        !page.exists("[role=dialog]"),
        "the returned focus reopened it"
    );
}

fn tooltip_app() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        Tooltip { label: rsx! { "Saves the draft" }, open_delay: 0, close_delay: 0,
            Button { id: "trigger", "Save" }
        }
        Button { id: "after", "After" }
    }
}

#[test]
fn a_tooltip_shows_while_tab_rests_on_its_trigger() {
    let mut page = mount(tooltip_app);
    page.click("#before");
    page.tab();
    assert!(
        page.is_focused("#trigger"),
        "focus is on {}",
        page.focus_owner()
    );
    assert!(
        page.exists("[role=tooltip]"),
        "Tab onto the trigger showed nothing"
    );

    page.tab();
    assert!(
        !page.exists("[role=tooltip]"),
        "Tab away left it up, focus on {}",
        page.focus_owner()
    );
}

fn carousel_app() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        Carousel {
            aria_label: "Photos",
            autoplay: true,
            autoplay_delay: 60_000,
            slides: vec![rsx! { div { "one" } }, rsx! { div { "two" } }],
        }
    }
}

/// WCAG 2.2.2: focus entering stops the rotation, which the status region
/// says by turning `polite`; leaving does not resume it (todo 548).
#[test]
fn focus_tabbed_into_a_carousel_pauses_it() {
    const STATUS: &str = "[role=status]";
    let mut page = mount(carousel_app);
    page.click("#before");
    assert_eq!(page.attr(STATUS, "aria-live").as_deref(), Some("off"));

    page.tab();
    assert_eq!(
        page.attr(STATUS, "aria-live").as_deref(),
        Some("polite"),
        "still rotating with focus on {}",
        page.focus_owner()
    );
    page.shift_tab();
    assert!(
        page.is_focused("#before"),
        "focus is on {}",
        page.focus_owner()
    );
    assert_eq!(page.attr(STATUS, "aria-live").as_deref(), Some("polite"));
    assert_eq!(
        page.attr("[aria-pressed]", "aria-pressed").as_deref(),
        Some("true")
    );
}

fn windows_app() -> Element {
    let first = use_floating_window(
        FloatingWindowOptions {
            aria_label: Some("First".into()),
            ..Default::default()
        },
        |_| rsx! { button { id: "in-first", "First" } },
    );
    let second = use_floating_window(
        FloatingWindowOptions {
            aria_label: Some("Second".into()),
            ..Default::default()
        },
        |_| rsx! { button { id: "in-second", "Second" } },
    );
    rsx! {
        button {
            id: "open",
            onclick: move |_| {
                first.open();
                second.open();
            },
            "Open"
        }
    }
}

/// The z-index of the positioned box a window's dialog sits in.
fn stacking(page: &Page, label: &str) -> i64 {
    let mut id = page.node(&format!("[aria-label={label}]"));
    loop {
        let doc = page.doc.inner.borrow();
        let value = doc.resolved_style_value(id, "z-index");
        if let Ok(z) = value.parse() {
            return z;
        }
        id = doc
            .get_node(id)
            .and_then(|node| node.parent)
            .expect("no z-index above");
    }
}

#[test]
fn focus_tabbed_into_a_lower_window_raises_it() {
    let mut page = mount(windows_app);
    page.click("#open");
    assert!(stacking(&page, "Second") > stacking(&page, "First"));

    // Through Second's four stops (handle, Window menu, Close, its button),
    // `#open`, then First's handle, Window menu and Close (todo 704).
    let mut menus = 0;
    for _ in 0..10 {
        if page.is_focused("#in-first") {
            break;
        }
        page.tab();
        menus += usize::from(page.is_focused("[data-window-menu]"));
    }
    assert!(
        page.is_focused("#in-first"),
        "Tab never reached it:\n{}",
        page.tree()
    );
    assert_eq!(menus, 2, "Tab skipped a window's menu button");
    assert!(
        stacking(&page, "First") > stacking(&page, "Second"),
        "focus inside did not raise it"
    );
}

fn color_app() -> Element {
    rsx! {
        Button { id: "before", "Before" }
        ColorField { label: "Accent", value: "#1c7ed6".parse::<ColorCode>().unwrap() }
        Button { id: "after", "After" }
    }
}

/// Focus alone opens it on the web; natively the silent move does.
#[test]
fn tab_onto_a_colour_field_opens_it_and_tab_out_closes_it() {
    let mut page = mount(color_app);
    page.click("#before");
    page.tab();
    assert!(page.exists("[role=dialog]"), "Tab onto it opened nothing");

    page.shift_tab();
    assert!(
        page.is_focused("#before"),
        "focus is on {}",
        page.focus_owner()
    );
    assert!(!page.exists("[role=dialog]"), "Tab out left it open");
}

fn autocomplete_app() -> Element {
    rsx! {
        Autocomplete {
            label: "City",
            value: "",
            options: vec!["Amsterdam".to_string(), "Berlin".to_string()],
            oninput: move |_| {},
        }
        Button { id: "after", "After" }
    }
}

#[test]
fn tab_out_of_an_autocomplete_closes_its_list() {
    let mut page = mount(autocomplete_app);
    page.focus("input");
    page.press(Key::ArrowDown);
    assert!(page.exists("[role=listbox]"), "{}", page.tree());

    page.tab();
    assert!(
        !page.exists("[role=listbox]"),
        "Tab left the list open, focus on {}",
        page.focus_owner()
    );
}

#[derive(Clone, Copy, PartialEq, Debug, Options)]
enum Fruit {
    Apple,
    Banana,
}

fn select_app() -> Element {
    let mut value = use_signal(|| Some(Fruit::Apple));
    rsx! {
        Select { label: "Fruit", value: value(), onchange: move |next| value.set(next) }
        Button { id: "after", "After" }
    }
}

#[test]
fn tab_out_of_a_select_closes_its_list() {
    let mut page = mount(select_app);
    page.click("#after");
    page.shift_tab();
    assert!(
        page.is_focused("[role=combobox]"),
        "focus is on {}",
        page.focus_owner()
    );
    page.press(Key::ArrowDown);
    assert!(page.exists("[role=listbox]"), "{}", page.tree());

    page.tab();
    assert!(
        !page.exists("[role=listbox]"),
        "Tab left the list open, focus on {}",
        page.focus_owner()
    );
}
