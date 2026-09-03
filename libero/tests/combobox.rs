//! The trigger's `aria-activedescendant` names only a row that is in the DOM:
//! never one while the list is loading or empty, and a highlight past the end
//! names the last row, the one drawn as active. And what the loader says comes
//! from the theme.

mod common;

use common::body;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Button, Combobox, ComboboxOption, ComboboxOptionArgs, ComboboxState, use_combobox,
    },
    theme::{ComboboxDefaults, ComboboxLabels, Theme},
};
use std::cell::{Cell, RefCell};

thread_local! {
    static STATE: RefCell<Option<ComboboxState>> = const { RefCell::new(None) };
    static OPTIONS: RefCell<Option<Signal<Vec<&'static str>>>> = const { RefCell::new(None) };
    static LOADING: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
    /// How many times `App` - the trigger's scope - has rendered on this
    /// test's thread.
    static RENDERS: Cell<usize> = const { Cell::new(0) };
}

#[component]
fn App() -> Element {
    RENDERS.set(RENDERS.get() + 1);
    let fruit = use_combobox();
    let options = use_signal(|| vec!["apple", "banana", "grape"]);
    let loading = use_signal(|| false);
    use_hook(|| fruit.open());
    STATE.with(|handle| *handle.borrow_mut() = Some(fruit));
    OPTIONS.with(|handle| *handle.borrow_mut() = Some(options));
    LOADING.with(|handle| *handle.borrow_mut() = Some(loading));

    rsx! {
        LiberoProvider {
            Combobox {
                state: fruit,
                options: options(),
                loading: loading(),
                option: move |o: ComboboxOptionArgs<&'static str>| rsx! {
                    ComboboxOption { onpick: move |_| {}, "{o.value}" }
                },
                Button { attributes: fruit.a11y_attributes(), "pick" }
            }
        }
    }
}

fn get<T: Copy>(key: &'static std::thread::LocalKey<RefCell<Option<T>>>) -> T {
    key.with(|handle| *handle.borrow())
        .expect("the app rendered")
}

/// The trigger's `aria-activedescendant`, if it has one.
///
/// Two passes: the list reports its row count while it renders, after the
/// trigger has already been drawn, so the trigger catches up one pass later.
fn descendant(dom: &mut VirtualDom) -> Option<String> {
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = body(&dioxus_ssr::render(dom));
    let button = &html[html.find("<button").expect("the trigger")..];
    let button = &button[..button.find('>').unwrap()];
    let at = button.find(r#"aria-activedescendant=""#)? + r#"aria-activedescendant=""#.len();
    Some(button[at..at + button[at..].find('"').unwrap()].to_string())
}

fn mount() -> VirtualDom {
    let mut dom = VirtualDom::new(App);
    dom.rebuild_in_place();
    dom
}

#[test]
fn an_open_list_names_the_active_row() {
    let mut dom = mount();
    let id = descendant(&mut dom).expect("row 0 is active on open");
    assert!(id.ends_with("-option-0"), "{id}");
    let html = body(&dioxus_ssr::render(&dom));
    assert!(html.contains(&format!(r#"id="{id}""#)), "{html}");
}

#[test]
fn a_loading_list_names_no_row() {
    let mut dom = mount();
    dom.in_runtime(|| get(&LOADING).set(true));
    assert_eq!(descendant(&mut dom), None);

    // And it comes back once the results land.
    dom.in_runtime(|| get(&LOADING).set(false));
    assert!(descendant(&mut dom).is_some_and(|id| id.ends_with("-option-0")));
}

#[test]
fn an_empty_list_names_no_row() {
    let mut dom = mount();
    dom.in_runtime(|| get(&OPTIONS).set(Vec::new()));
    assert_eq!(descendant(&mut dom), None);
}

/// The list reports its row count by writing the state during render, which
/// re-renders the trigger's scope once per change of count - and then stops.
#[test]
fn reporting_the_row_count_costs_one_render_and_settles() {
    fn renders_after(dom: &mut VirtualDom) -> usize {
        let start = RENDERS.get();
        for _ in 0..5 {
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
        RENDERS.get() - start
    }

    let mut dom = mount();
    // Mount: the open effect, and the count going from 0 to 3.
    assert!(renders_after(&mut dom) <= 2);
    assert_eq!(renders_after(&mut dom), 0, "still rendering after mount");

    // A shorter list: one render for the new options, one for the new count.
    dom.in_runtime(|| get(&OPTIONS).set(vec!["apple"]));
    assert_eq!(renders_after(&mut dom), 2);
    assert_eq!(renders_after(&mut dom), 0, "still rendering after a filter");

    // Same length, different rows: the count is unchanged, so no extra render.
    dom.in_runtime(|| get(&OPTIONS).set(vec!["grape"]));
    assert_eq!(renders_after(&mut dom), 1);
}

/// A shorter list leaves the highlight past its end. The list draws the last
/// row as active, so that is the row the trigger has to name.
#[test]
fn a_highlight_past_the_end_names_the_last_row() {
    let mut dom = mount();
    // Settle first, or the open effect resets the highlight to row 0.
    descendant(&mut dom);
    dom.in_runtime(|| get(&STATE).set_active(Some(2)));
    assert!(descendant(&mut dom).is_some_and(|id| id.ends_with("-option-2")));
    dom.in_runtime(|| get(&OPTIONS).set(vec!["apple"]));
    let id = descendant(&mut dom).expect("a row to point at");
    assert!(id.ends_with("-option-0"), "{id}");
}

static GERMAN: Theme = Theme {
    combobox: ComboboxDefaults {
        labels: ComboboxLabels {
            loading: "Wird geladen",
        },
        ..Theme::DEFAULT.combobox
    },
    ..Theme::DEFAULT
};

/// An open, loading list, so the loader and its text are on screen.
fn loading(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    body(&dioxus_ssr::render(&dom))
}

#[component]
fn Loading(label: Option<String>) -> Element {
    let fruit = use_combobox();
    use_hook(|| fruit.open());
    rsx! {
        Combobox {
            state: fruit,
            options: Vec::<&'static str>::new(),
            loading: true,
            loading_label: label,
            option: move |o: ComboboxOptionArgs<&'static str>| rsx! {
                ComboboxOption { onpick: move |_| {}, "{o.value}" }
            },
            Button { attributes: fruit.a11y_attributes(), "pick" }
        }
    }
}

#[test]
fn the_loading_label_comes_from_the_theme() {
    fn english() -> Element {
        rsx! { LiberoProvider { Loading {} } }
    }
    fn german() -> Element {
        rsx! { LiberoProvider { theme: &GERMAN, Loading {} } }
    }

    let html = loading(english);
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains(">Loading<"), "{html}");

    let html = loading(german);
    assert!(html.contains(">Wird geladen<"), "{html}");
    assert!(!html.contains(">Loading<"), "{html}");
}

/// The prop still wins over the theme, for a label a static string cannot
/// express - one naming what is being searched.
#[test]
fn a_loading_label_prop_beats_the_theme() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { theme: &GERMAN,
                Loading { label: "Searching fruit" }
            }
        }
    }

    let html = loading(app);
    assert!(html.contains(">Searching fruit<"), "{html}");
    assert!(!html.contains(">Wird geladen<"), "{html}");
}
