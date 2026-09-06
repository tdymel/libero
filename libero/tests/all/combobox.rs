//! The trigger's `aria-controls` names the listbox only while it is mounted,
//! and its `aria-activedescendant` names only a row that is in the DOM:
//! never one while the list is loading or empty, and a highlight past the end
//! names the last row, the one drawn as active. What the list says while it
//! loads comes from the theme, and it is said by a status region that is always
//! mounted and sits outside the `aria-busy` dropdown.

use crate::common::body;

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

/// The trigger's `aria-controls`, if it has one. Two passes, as for
/// [`descendant`].
fn controls(dom: &mut VirtualDom) -> (Option<String>, String) {
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = body(&dioxus_ssr::render(dom));
    let button = &html[html.find("<button").expect("the trigger")..];
    let button = &button[..button.find('>').unwrap()];
    let id = button.find(r#"aria-controls=""#).map(|at| {
        let at = at + r#"aria-controls=""#.len();
        button[at..at + button[at..].find('"').unwrap()].to_string()
    });
    (id, html)
}

/// A closed, empty or loading list draws no listbox, so there is nothing for
/// `aria-controls` to name (todo 360).
#[test]
fn aria_controls_names_the_listbox_only_while_it_is_mounted() {
    let mut dom = mount();
    let (id, html) = controls(&mut dom);
    let id = id.expect("an open list with rows is named");
    assert!(html.contains(&format!(r#"id="{id}""#)), "{html}");

    dom.in_runtime(|| get(&LOADING).set(true));
    assert_eq!(controls(&mut dom).0, None, "while loading");
    dom.in_runtime(|| get(&LOADING).set(false));
    assert!(controls(&mut dom).0.is_some(), "once the results land");

    dom.in_runtime(|| get(&OPTIONS).set(Vec::new()));
    assert_eq!(controls(&mut dom).0, None, "while empty");
    dom.in_runtime(|| get(&OPTIONS).set(vec!["apple"]));
    assert!(controls(&mut dom).0.is_some(), "once rows are back");

    dom.in_runtime(|| get(&STATE).close());
    let (id, html) = controls(&mut dom);
    assert_eq!(id, None, "while closed");
    assert!(!html.contains("-listbox\""), "{html}");
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
    assert_eq!(status(&html), "Loading", "{html}");
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

/// The one `role="status"` element's text. Panics if there is not exactly one.
fn status(html: &str) -> String {
    assert_eq!(html.matches(r#"role="status""#).count(), 1, "{html}");
    let open = html.find(r#"role="status""#).unwrap();
    let text = &html[open + html[open..].find('>').unwrap() + 1..];
    text[..text.find('<').unwrap()].to_string()
}

/// The region is there before the loading starts, says the label while it
/// runs, and empties once the results land - without ever unmounting, which a
/// screen reader needs to announce a change at all.
#[test]
fn the_status_region_stays_mounted_and_its_text_changes() {
    let mut dom = mount();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(status(&body(&dioxus_ssr::render(&dom))), "");

    dom.in_runtime(|| get(&LOADING).set(true));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(status(&body(&dioxus_ssr::render(&dom))), "Loading");

    dom.in_runtime(|| get(&LOADING).set(false));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(status(&body(&dioxus_ssr::render(&dom))), "");
}

/// A closed list is not loading anything the user asked for, so it says nothing.
#[test]
fn a_closed_list_says_nothing_while_loading() {
    let mut dom = mount();
    dom.in_runtime(|| {
        get(&LOADING).set(true);
        get(&STATE).close();
    });
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(status(&body(&dioxus_ssr::render(&dom))), "");
}

/// The region is the trigger's sibling, not the dropdown's content: the
/// dropdown is `aria-busy` while loading, and the loader inside it is silent.
#[test]
fn the_status_region_sits_outside_the_busy_dropdown() {
    fn app() -> Element {
        rsx! { LiberoProvider { Loading {} } }
    }
    let html = loading(app);
    let after_trigger = &html[html.find("</button>").expect("the trigger") + "</button>".len()..];
    assert!(
        after_trigger.starts_with("<span")
            && after_trigger[..after_trigger.find('>').unwrap()].contains(r#"role="status""#),
        "{html}"
    );

    let busy = html.find(r#"aria-busy="true""#).expect("a busy dropdown");
    let dropdown = &html[busy..];
    assert!(!dropdown.contains(r#"role="status""#), "{html}");
    assert!(
        dropdown.contains(r#"aria-hidden="true""#),
        "the loader is silent: {html}"
    );
}

/// The regression that made the arrow keys look dead: the rows were behind a
/// memoized subtree, so moving the highlight - or filtering the options -
/// changed state nothing redrew.
mod combobox_highlight {
    use crate::common::{attributes_of, body};
    use dioxus::prelude::*;
    use libero::components::use_combobox;
    use libero::{
        LiberoProvider,
        components::{Button, Combobox, ComboboxOption, ComboboxOptionArgs, ComboboxState},
    };
    use std::cell::RefCell;

    thread_local! {
        /// The rendered app's state, so the test can move the highlight.
        static STATE: RefCell<Option<ComboboxState>> = const { RefCell::new(None) };
        /// Its `options`, so the test can filter them the way typing does.
        static OPTIONS: RefCell<Option<Signal<Vec<&'static str>>>> = const { RefCell::new(None) };
    }

    #[component]
    fn App() -> Element {
        let fruit = use_combobox();
        let options = use_signal(|| vec!["apple", "banana", "grape"]);
        use_hook(|| fruit.open());
        STATE.with(|handle| *handle.borrow_mut() = Some(fruit));
        OPTIONS.with(|handle| *handle.borrow_mut() = Some(options));

        rsx! {
            LiberoProvider {
                Combobox {
                    state: fruit,
                    options: options(),
                    option: move |o: ComboboxOptionArgs<&'static str>| rsx! {
                        ComboboxOption { onpick: move |_| {}, "{o.value}" }
                    },
                    Button { attributes: fruit.a11y_attributes(), "pick" }
                }
            }
        }
    }

    fn state() -> ComboboxState {
        STATE
            .with(|handle| *handle.borrow())
            .expect("the app rendered")
    }

    /// One row, opening tag through closing tag, found by the `id`
    /// `ComboboxOption` takes from the `Combobox`.
    ///
    /// Matched together with `role="option"`, the attribute that follows it:
    /// the trigger's own `aria-activedescendant` holds the very same id, and
    /// it comes first in the document.
    fn row_of(html: &str, index: usize) -> String {
        let id = format!(r#"-option-{index}" role="option""#);
        let at = html
            .find(&id)
            .unwrap_or_else(|| panic!("no row {index} in:\n{html}"));
        let open = html[..at].rfind('<').expect("an unterminated tag");
        let close = at + html[at..].find("</div>").expect("an unclosed row");
        html[open..close].to_string()
    }

    fn render_pass(dom: &mut VirtualDom) -> String {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        body(&dioxus_ssr::render(dom))
    }

    #[test]
    fn moving_the_active_row_redraws_the_rows_it_touches() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();

        // Mount takes one extra pass to settle: the list reports its row count
        // to the state, which re-renders the trigger's scope.
        render_pass(&mut dom);
        let html = render_pass(&mut dom);
        assert!(row_of(&html, 0).contains("active"));
        assert!(!row_of(&html, 2).contains("active"));

        dom.in_runtime(|| state().set_active(Some(2)));

        let html = render_pass(&mut dom);
        assert!(!row_of(&html, 0).contains("active"), "row 0 stayed active");
        assert!(row_of(&html, 2).contains("active"), "row 2 never lit up");
    }

    /// The same trap one level up: a stable row `Callback` compared equal, so
    /// filtering the options left the old ones on screen.
    #[test]
    fn filtering_the_options_redraws_the_rows() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        assert!(render_pass(&mut dom).contains("banana"));

        let options = OPTIONS
            .with(|handle| *handle.borrow())
            .expect("the app rendered");
        // What typing "ap" leaves: a shorter list whose second row is a
        // different option at the same index.
        dom.in_runtime(|| options.clone().set(vec!["apple", "grape"]));
        assert!(
            row_of(&render_pass(&mut dom), 1).contains("grape"),
            "row 1 stayed stale"
        );

        // The harder case: same length, different options, same highlight - so
        // every prop a memoizing subtree could compare is unchanged.
        dom.in_runtime(|| options.clone().set(vec!["apricot", "plum"]));

        let html = render_pass(&mut dom);
        assert!(row_of(&html, 0).contains("apricot"), "row 0 stayed stale");
        assert!(row_of(&html, 1).contains("plum"), "row 1 stayed stale");
    }

    /// The dropdown's chrome is `Paper`'s, and both halves have to agree: the
    /// class reads the themed surface, and the root carries the `bordered`
    /// token that class's border fold answers.
    #[test]
    fn the_dropdown_is_a_bordered_paper_surface() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);

        let markup = body(&html);
        let listbox = markup
            .find(r#"role="listbox""#)
            .expect("the dropdown rendered");
        let root = markup[..listbox]
            .rfind("position:fixed")
            .and_then(|style| markup[..style].rfind("<div"))
            .expect("the portaled dropdown root");
        let root = attributes_of(&markup[root..], "div");

        assert!(
            root["data-state"]
                .split(' ')
                .any(|token| token == "bordered"),
            "{root:?}"
        );
        let class = root["class"].split(' ').next().expect("a framework class");
        assert!(
            html.contains(&format!(
                ".{class}{{background:var(--lsx-paper-background);--lsx-focus-contrast:var(--lsx-paper-contrast);"
            )),
            "the dropdown's base no longer starts from `paper_sx()`"
        );
        assert!(
            html.contains(&format!(
                r#".{class}[data-state~="bordered"]{{border:1px solid var(--lsx-paper-border-color);}}"#
            )),
            "nothing answers the `bordered` token"
        );
    }

    /// The port to `use_popover`: the dropdown leaves the wrapper entirely, so
    /// no `overflow: hidden` ancestor can clip it.
    #[test]
    fn the_dropdown_is_portaled_out_of_the_wrapper() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        let html = render_pass(&mut dom);

        let trigger = html.find("<button").expect("the trigger rendered");
        let wrapper_closes = trigger + html[trigger..].find("</div>").expect("an open wrapper");
        let listbox = html
            .find(r#"role="listbox""#)
            .expect("the dropdown rendered");

        assert!(
            listbox > wrapper_closes,
            "the dropdown is still nested inside the trigger's wrapper"
        );
        assert!(
            row_of(&html, 0).contains("apple"),
            "the portaled rows lost their content"
        );
    }

    /// `ComboboxOption` reads its id and its highlight from a context, and a
    /// portaled subtree mounts under `PortalOutlet` rather than under
    /// `ComboboxCore` - so the context has to be handed across as a prop. The
    /// lookup is a `try_consume_context`, so getting this wrong fails silently.
    #[test]
    fn the_rows_keep_their_context_across_the_portal() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        dom.in_runtime(|| state().set_active(Some(2)));

        let html = render_pass(&mut dom);
        let row = row_of(&html, 2);

        assert!(row.contains("active"), "the row lost the highlight context");
        assert!(
            row.contains(&format!(r#"id="{}-option-2""#, state().id())),
            "the row lost the id context"
        );
    }

    /// Fixed to the viewport, not absolute to a wrapper that is no longer
    /// positioned - and hidden until the first measurement lands, which under
    /// SSR never does.
    #[test]
    fn the_dropdown_is_hidden_until_it_has_been_measured() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        let html = render_pass(&mut dom);

        assert!(
            html.contains(
                r#"style="position:fixed;left:0px;top:0px;width:auto;min-width:auto;max-width:calc(100vw - 16px);visibility:hidden;""#
            ),
            "the dropdown is not laid out fixed and hidden before its first \
             measurement, which under SSR never lands:\n{html}"
        );
    }

    /// A long row must not push the dropdown off a phone's screen: the box is
    /// capped at the viewport less the collision padding (8px, the theme's) at
    /// both edges, so the row's ellipsis takes over (todo 357). In CSS, so it
    /// already holds on the pass that is measured.
    #[test]
    fn the_dropdown_is_capped_at_the_viewport_less_its_padding() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        let html = render_pass(&mut dom);

        assert!(
            html.contains("max-width:calc(100vw - 16px);"),
            "the dropdown can grow past the viewport:\n{html}"
        );
    }

    /// The whole reason the state is a handle: the trigger has to be able to
    /// name the row the arrows are on.
    #[test]
    fn the_trigger_points_at_the_active_row() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        dom.in_runtime(|| state().set_active(Some(1)));

        let html = render_pass(&mut dom);
        let button = attributes_of(&html, "button");

        assert_eq!(button["role"], "combobox");
        assert_eq!(button["aria-expanded"], "true");
        assert!(html.contains(&format!(r#"id="{}""#, button["aria-activedescendant"])));
        assert!(
            row_of(&html, 1).contains(&button["aria-activedescendant"]),
            "the trigger names a row other than the active one"
        );
    }
}
