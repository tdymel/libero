//! The trigger's `aria-controls` names the listbox only while it is mounted,
//! and its `aria-activedescendant` names only a row that is in the DOM:
//! never one while the list is loading or empty, and a highlight past the end
//! names the last row, the one drawn as active. What the list says while it
//! loads comes from the localization, and it is said by a status region that is always
//! mounted and sits outside the `aria-busy` dropdown; so is the result count.

use crate::common::{attributes_of, body, element_at};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        Button, Combobox, ComboboxOption, ComboboxOptionArgs, ComboboxState, OptionList,
        use_combobox,
    },
    localization::{CommonLabels, Localization},
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
                // `None` is the pending list - the shape a `Resource` that
                // has not answered yet converts into.
                options: (!loading()).then(|| OptionList::from(options())),
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

/// The trigger's attribute `name`, if it has one, and the markup it was read from.
///
/// Two passes: the list reports its row count while it renders, after the
/// trigger has already been drawn, so the trigger catches up one pass later.
fn trigger_attribute(dom: &mut VirtualDom, name: &str) -> (Option<String>, String) {
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = body(&dioxus_ssr::render(dom));
    (attributes_of(&html, "button").get(name).cloned(), html)
}

/// The trigger's `aria-activedescendant`, if it has one.
fn descendant(dom: &mut VirtualDom) -> Option<String> {
    trigger_attribute(dom, "aria-activedescendant").0
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

/// The trigger's `aria-controls`, if it has one, and the markup.
fn controls(dom: &mut VirtualDom) -> (Option<String>, String) {
    trigger_attribute(dom, "aria-controls")
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

static GERMAN: Localization = Localization {
    common: CommonLabels {
        loading: "Wird geladen",
        ..CommonLabels::ENGLISH
    },
    ..Localization::ENGLISH
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
            options: None::<OptionList<&'static str>>,
            loading_label: label,
            option: move |o: ComboboxOptionArgs<&'static str>| rsx! {
                ComboboxOption { onpick: move |_| {}, "{o.value}" }
            },
            Button { attributes: fruit.a11y_attributes(), "pick" }
        }
    }
}

#[test]
fn the_loading_label_comes_from_the_localization() {
    fn english() -> Element {
        rsx! { LiberoProvider { Loading {} } }
    }
    fn german() -> Element {
        rsx! { LiberoProvider { localization: &GERMAN, Loading {} } }
    }

    let html = loading(english);
    assert_eq!(status(&html), "Loading", "{html}");
    assert!(html.contains(">Loading<"), "{html}");

    let html = loading(german);
    assert!(html.contains(">Wird geladen<"), "{html}");
    assert!(!html.contains(">Loading<"), "{html}");
}

/// The prop still wins over the localization, for a label a static string
/// cannot express - one naming what is being searched.
#[test]
fn a_loading_label_prop_beats_the_localization() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { localization: &GERMAN,
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

/// The region counts the results, says the label while loading, and counts
/// again once they land (todo 1574) - without ever unmounting, which a screen
/// reader needs to announce a change at all.
#[test]
fn the_status_region_stays_mounted_and_its_text_changes() {
    let mut dom = mount();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(status(&body(&dioxus_ssr::render(&dom))), "3 results");

    dom.in_runtime(|| get(&LOADING).set(true));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(status(&body(&dioxus_ssr::render(&dom))), "Loading");

    dom.in_runtime(|| get(&LOADING).set(false));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(status(&body(&dioxus_ssr::render(&dom))), "3 results");
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

/// An open list, its `empty_label` and `labelled_by` set.
#[component]
fn Labelled(options: Vec<&'static str>, empty_label: Option<String>) -> Element {
    let fruit = use_combobox();
    use_hook(|| fruit.open());
    rsx! {
        Combobox {
            state: fruit,
            options,
            empty_label,
            labelled_by: "fruit-label",
            option: move |o: ComboboxOptionArgs<&'static str>| rsx! {
                ComboboxOption { onpick: move |_| {}, "{o.value}" }
            },
            Button { attributes: fruit.a11y_attributes(), "pick" }
        }
    }
}

/// Todo 1269: an empty list is said, not only drawn, as on `Select` and `Autocomplete`.
#[test]
fn an_empty_list_says_its_empty_label() {
    fn default_label() -> Element {
        rsx! { LiberoProvider { Labelled { options: Vec::new() } } }
    }
    fn own_label() -> Element {
        rsx! { LiberoProvider { Labelled { options: Vec::new(), empty_label: "No fruit" } } }
    }

    let html = loading(default_label);
    assert_eq!(status(&html), "No results", "{html}");
    assert!(html.contains(">No results<"), "drawn too: {html}");

    let html = loading(own_label);
    assert_eq!(status(&html), "No fruit", "{html}");
}

/// Todo 1270: the bare `Combobox` names its listbox as `Select` and `Autocomplete` do.
#[test]
fn labelled_by_names_the_listbox() {
    fn app() -> Element {
        rsx! { LiberoProvider { Labelled { options: vec!["apple"] } } }
    }
    let html = loading(app);
    let listbox = html.find(r#"role="listbox""#).expect("a listbox");
    assert!(
        element_at(&html, listbox).contains(r#"aria-labelledby="fruit-label""#),
        "{html}"
    );
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
    let dropdown = element_at(&html, busy);
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
    use super::{App, OPTIONS, STATE, get};
    use crate::common::{attributes_of, body, rule_of, style_of, tags_with};
    use dioxus::prelude::*;
    use libero::components::ComboboxState;
    use std::collections::BTreeMap;

    fn state() -> ComboboxState {
        get(&STATE)
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

        let options = get(&OPTIONS);
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

        let root = dropdown_root(&html);

        assert!(
            root["data-state"]
                .split(' ')
                .any(|token| token == "bordered"),
            "{root:?}"
        );
        let class = root["class"].split(' ').next().expect("a framework class");
        let base = rule_of(&html, &format!(".{class}"));
        assert_eq!(
            base["background"], "var(--lsx-paper-background)",
            "the dropdown's base no longer starts from `paper_sx()`: {base:?}"
        );
        assert_eq!(
            base["--lsx-focus-contrast"], "var(--lsx-paper-contrast)",
            "{base:?}"
        );
        let bordered = rule_of(&html, &format!(r#".{class}[data-state~="bordered"]"#));
        assert_eq!(
            bordered["border"], "1px solid var(--lsx-paper-border-color)",
            "nothing answers the `bordered` token"
        );
    }

    /// The portaled dropdown's root: the tag whose inline style publishes the
    /// popover's available height.
    fn dropdown_root(html: &str) -> BTreeMap<String, String> {
        tags_with(&body(html), "style=")
            .into_iter()
            .find(|tag| style_of(tag).contains_key("--lsx-popover-available-height"))
            .unwrap_or_else(|| panic!("no portaled dropdown root in:\n{html}"))
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

        let style = style_of(&dropdown_root(&html));
        assert_eq!(style["position"], "fixed", "{style:?}");
        assert_eq!(
            style["visibility"], "hidden",
            "the dropdown is shown before its first measurement, which under \
             SSR never lands: {style:?}"
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

        let style = style_of(&dropdown_root(&html));
        assert_eq!(
            style["max-width"], "calc(100vw - 16px)",
            "the dropdown can grow past the viewport: {style:?}"
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

/// Groups, per-option `disabled` and the pending list, which all arrive
/// through the one `options` prop.
///
/// The list is open from the first render, so SSR sees the rows.
mod option_list {
    use super::*;
    use libero::components::{OptionItem, OptionList};

    /// An open list over `options`, rendered once.
    fn html(options: fn() -> Option<OptionList<&'static str>>) -> String {
        #[component]
        fn App(options: Option<OptionList<&'static str>>) -> Element {
            let fruit = use_combobox();
            use_hook(|| fruit.open());
            rsx! {
                LiberoProvider {
                    Combobox {
                        state: fruit,
                        options,
                        option: move |o: ComboboxOptionArgs<&'static str>| rsx! {
                            ComboboxOption { onpick: move |_| {}, "{o.value}" }
                        },
                        empty: rsx! { "No fruit" },
                        Button { attributes: fruit.a11y_attributes(), "pick" }
                    }
                }
            }
        }

        let options = options();
        let mut dom = VirtualDom::new_with_props(App, AppProps { options });
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        body(&dioxus_ssr::render(&dom))
    }

    /// Each named run is its own `role="group"`, named by a heading that is
    /// `role="presentation"` - the group's `aria-labelledby` already speaks it,
    /// so as an element of its own it would be read twice.
    #[test]
    fn a_named_run_is_a_group_that_its_heading_names() {
        let html = html(|| {
            Some(
                OptionList::grouped()
                    .group("Pome", ["apple"])
                    .group("Berry", ["grape", "fig"]),
            )
        });

        assert_eq!(html.matches(r#"role="group""#).count(), 2, "{html}");
        assert_eq!(html.matches(r#"role="presentation""#).count(), 2, "{html}");

        // Every `aria-labelledby` inside the list has to name exactly one
        // element, or the group is left unnamed. The ids are keyed on the
        // group's first row, so they differ.
        let named: Vec<String> = html
            .split(r#"aria-labelledby=""#)
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap().to_string())
            .filter(|id| id.contains("-group-"))
            .collect();
        assert_eq!(named.len(), 2, "{html}");
        assert_ne!(named[0], named[1], "both groups share one heading id");
        for id in &named {
            assert_eq!(
                html.matches(&format!(r#"id="{id}""#)).count(),
                1,
                "{id} names no element, or more than one:\n{html}"
            );
        }
        assert!(html.contains(">Pome<"), "{html}");
        assert!(html.contains(">Berry<"), "{html}");
    }

    /// A label that comes back after another group is drawn twice rather than
    /// merged, so the caller's order survives. The two headings carry
    /// different ids - nothing requires a `role="group"` to be uniquely named,
    /// but two elements may never share an id.
    #[test]
    fn a_repeated_group_label_is_drawn_twice_in_the_callers_order() {
        let html = html(|| {
            Some(
                OptionList::grouped()
                    .group("Pome", ["apple"])
                    .group("Berry", ["grape"])
                    .group("Pome", ["pear"]),
            )
        });

        assert_eq!(html.matches(r#"role="group""#).count(), 3, "{html}");
        // The rows are in the order they were listed, not regrouped.
        let order: Vec<&str> = ["apple", "grape", "pear"]
            .into_iter()
            .filter(|row| html.contains(row))
            .collect();
        assert_eq!(order, ["apple", "grape", "pear"], "{html}");
        let apple = html.find("apple").expect("apple");
        let grape = html.find("grape").expect("grape");
        let pear = html.find("pear").expect("pear");
        assert!(apple < grape && grape < pear, "reordered:\n{html}");

        // Two headings say "Pome", and no id is shared.
        assert_eq!(html.matches(">Pome<").count(), 2, "{html}");
        let ids: Vec<String> = html
            .split(r#"aria-labelledby=""#)
            .skip(1)
            .map(|rest| rest.split('"').next().unwrap().to_string())
            .filter(|id| id.contains("-group-"))
            .collect();
        assert_eq!(ids.len(), 3, "{html}");
        for id in &ids {
            assert_eq!(html.matches(&format!(r#"id="{id}""#)).count(), 1, "{id}");
        }
    }

    /// An ungrouped list keeps the markup it has always had: no wrapper at
    /// all, rows straight inside the listbox.
    #[test]
    fn an_ungrouped_list_wraps_nothing() {
        let html = html(|| Some(OptionList::from(vec!["apple", "grape"])));
        assert!(!html.contains(r#"role="group""#), "{html}");
        assert_eq!(html.matches(r#"role="option""#).count(), 2, "{html}");
    }

    /// A disabled row is drawn and reachable by a reader - `aria-disabled`,
    /// not the `disabled` attribute, which means nothing on a `div`.
    #[test]
    fn a_disabled_option_says_so_and_stays_in_the_list() {
        let html = html(|| {
            Some(OptionList::new([
                "apple".into(),
                OptionItem::new("grape").disabled(true),
            ]))
        });

        assert_eq!(html.matches(r#"role="option""#).count(), 2, "{html}");
        assert_eq!(html.matches(r#"aria-disabled="true""#).count(), 1, "{html}");
        assert!(html.contains("grape"), "{html}");
    }

    /// The pending list holds the empty state back. An async list is empty
    /// between the request and its answer, so without this every keystroke
    /// would flash "no results" before the data lands.
    #[test]
    fn a_pending_list_shows_the_loader_and_not_the_empty_state() {
        let pending = html(|| None);
        assert!(!pending.contains("No fruit"), "{pending}");
        assert!(pending.contains(r#"aria-busy="true""#), "{pending}");

        // Ready and empty is a real answer - a fetch that failed looks exactly
        // like this - so the empty state is what it shows.
        let empty = html(|| Some(OptionList::default()));
        assert!(empty.contains("No fruit"), "{empty}");
        assert!(!empty.contains(r#"aria-busy="true""#), "{empty}");
    }
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::body;
    use crate::dispatch::*;
    use dioxus::core::{ElementId, WriteMutations};

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Button, ColorField, MultiSelect, PhoneField, SelectionArgs, TagsField},
    };

    /// The pointer and the ARIA go through the same `disabled`/`readonly` gate the
    /// keys do (todos 401, 411, 414): a refused list is not drawn, so no row takes
    /// a click, and the trigger claims no listbox that is not in the DOM.
    mod combobox_refusal {
        use super::*;
        use libero::components::{
            Autocomplete, Cascader, CascaderOption, Combobox, ComboboxOption, ComboboxOptionArgs,
            DateField, Select, use_combobox,
        };

        thread_local! {
            static LOCKED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
            static PICKED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
            static TAGS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
        }

        fn locked() -> bool {
            LOCKED.with(|cell| cell.get())
        }

        /// A row's click listener registers before its `role="option"`; the
        /// recorder matches the two by element id.
        fn option_click(find: &FindClickListener) -> Option<ElementId> {
            find.matching("click", "role", "option").first().copied()
        }

        fn mount(app: fn() -> Element) -> (VirtualDom, FindClickListener) {
            LOCKED.with(|cell| cell.set(false));
            PICKED.with(|cell| cell.set(false));
            TAGS.with(|cell| cell.borrow_mut().clear());
            let Page { dom, rec } = Page::mount(app);
            (dom, rec)
        }

        /// Two passes: the list writes into the caller's state while it renders,
        /// so the trigger catches up one pass later.
        fn settle(dom: &mut VirtualDom, to: &mut impl WriteMutations) {
            dom.render_immediate(to);
            dom.render_immediate(to);
        }

        /// Flips the lock with the list already open, the way a caller reacting
        /// to something else would.
        fn lock(dom: &mut VirtualDom, to: &mut impl WriteMutations) {
            LOCKED.with(|cell| cell.set(true));
            dom.mark_dirty(dioxus::core::ScopeId::APP);
            settle(dom, to);
        }

        fn assert_no_listbox_claimed(dom: &VirtualDom, what: &str) {
            let html = body(&dioxus_ssr::render(dom));
            assert!(
                !html.contains(r#"role="listbox""#),
                "{what}: the list is drawn:\n{html}"
            );
            assert!(
                !html.contains(r#"aria-expanded="true""#),
                "{what}: aria-expanded:\n{html}"
            );
            assert!(
                !html.contains("aria-controls"),
                "{what}: aria-controls:\n{html}"
            );
            assert!(
                !html.contains("aria-activedescendant"),
                "{what}: aria-activedescendant:\n{html}"
            );
        }

        fn assert_listbox(dom: &VirtualDom, what: &str) {
            let html = body(&dioxus_ssr::render(dom));
            assert!(
                html.contains(r#"role="listbox""#),
                "{what}: the list never opened:\n{html}"
            );
        }

        fn fruit() -> Element {
            let fruit = use_combobox();
            use_hook(|| fruit.open());
            rsx! {
                LiberoProvider {
                    Combobox {
                        state: fruit,
                        disabled: locked(),
                        options: vec!["apple"],
                        option: move |o: ComboboxOptionArgs<&'static str>| rsx! {
                            ComboboxOption {
                                onpick: move |_| PICKED.with(|p| p.set(true)),
                                "{o.value}"
                            }
                        },
                        Button { attributes: fruit.a11y_attributes(), "pick" }
                    }
                }
            }
        }

        /// Todo 411: `disabled` gated only the keys, so an open row still took a
        /// mouse pick.
        #[test]
        fn a_combobox_disabled_while_open_takes_no_mouse_pick() {
            LOCKED.with(|cell| cell.set(false));
            PICKED.with(|cell| cell.set(false));
            let Page {
                mut dom,
                rec: mut find,
            } = Page::build(fruit);
            settle(&mut dom, &mut find);
            assert_listbox(&dom, "enabled");
            let row = option_click(&find).expect("the row registered no click listener");

            lock(&mut dom, &mut find);
            assert_no_listbox_claimed(&dom, "disabled");
            dom.runtime()
                .handle_event("click", Event::new(click_event(), true), row);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            assert!(
                !PICKED.with(|p| p.get()),
                "disabled: true still let a mouse click pick a row"
            );
        }

        /// Mounted disabled with the state already open: nothing is drawn at all.
        #[test]
        fn a_combobox_mounted_disabled_and_open_draws_no_row() {
            LOCKED.with(|cell| cell.set(true));
            let Page {
                mut dom,
                rec: mut find,
            } = Page::build(fruit);
            settle(&mut dom, &mut find);
            assert_eq!(
                option_click(&find),
                None,
                "a disabled combobox drew a clickable row"
            );
            assert_no_listbox_claimed(&dom, "disabled from the start");
        }

        fn topics() -> Element {
            rsx! {
                LiberoProvider {
                    TagsField {
                        label: "Topics",
                        suggestions: vec!["rust".to_string()],
                        readonly: locked(),
                        onchange: move |next: Vec<String>| TAGS.with(|cell| *cell.borrow_mut() = next),
                    }
                }
            }
        }

        /// Todo 414: the list opened on `!disabled` alone and `onpick` had no
        /// guard, so a read-only field still committed a mouse-picked tag.
        #[test]
        fn a_tags_field_made_readonly_while_open_takes_no_mouse_pick() {
            let (mut dom, find) = mount(topics);
            let mut rows = FindClickListener::default();
            dom.runtime().handle_event(
                "input",
                Event::new(input_event("rust"), true),
                input_listener(&find),
            );
            settle(&mut dom, &mut rows);
            assert_listbox(&dom, "editable");
            let row = option_click(&rows).expect("the row registered no click listener");

            lock(&mut dom, &mut rows);
            assert_no_listbox_claimed(&dom, "readonly");
            dom.runtime()
                .handle_event("click", Event::new(click_event(), true), row);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            assert_eq!(
                TAGS.with(|cell| cell.borrow().clone()),
                Vec::<String>::new(),
                "a readonly TagsField committed a tag through a mouse pick"
            );
        }

        fn city(readonly: bool) -> Element {
            rsx! {
                LiberoProvider {
                    Select::<String> {
                        label: "City",
                        options: vec!["Berlin".to_string(), "Bonn".to_string()],
                        readonly,
                        disabled: locked(),
                    }
                }
            }
        }

        fn press(dom: &mut VirtualDom, target: ElementId, key: Key) {
            dom.runtime()
                .handle_event("keydown", Event::new(key_event(key), true), target);
            settle(dom, &mut dioxus::core::NoOpMutations);
        }

        /// Todo 401: one ArrowDown on a read-only `Select` opened the state, and
        /// the trigger read the raw state into `aria-expanded` and `aria-controls`.
        #[test]
        fn arrow_down_on_a_readonly_select_claims_no_listbox() {
            let (mut dom, find) = mount(|| city(true));
            press(&mut dom, last_keydown(&find), Key::ArrowDown);
            assert_no_listbox_claimed(&dom, "readonly after ArrowDown");
        }

        /// Todo 401: disabling an open `Select` hid the list, while nothing closed
        /// the state the trigger's ARIA read.
        #[test]
        fn a_select_disabled_while_open_claims_no_listbox() {
            let (mut dom, find) = mount(|| city(false));
            press(&mut dom, last_keydown(&find), Key::ArrowDown);
            assert_listbox(&dom, "enabled after ArrowDown");

            lock(&mut dom, &mut dioxus::core::NoOpMutations);
            assert_no_listbox_claimed(&dom, "disabled while open");
        }

        fn fruit_field() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        label: "Fruit",
                        options: vec!["apple".to_string()],
                        readonly: locked(),
                    }
                }
            }
        }

        #[test]
        fn an_autocomplete_made_readonly_while_open_claims_no_listbox() {
            let (mut dom, find) = mount(fruit_field);
            dom.runtime().handle_event(
                "input",
                Event::new(input_event("a"), true),
                input_listener(&find),
            );
            settle(&mut dom, &mut dioxus::core::NoOpMutations);
            assert_listbox(&dom, "editable");

            lock(&mut dom, &mut dioxus::core::NoOpMutations);
            assert_no_listbox_claimed(&dom, "readonly while open");
        }

        fn phone(readonly: bool) -> Element {
            rsx! {
                LiberoProvider {
                    PhoneField { label: "Phone", readonly, disabled: locked() }
                }
            }
        }

        /// The country button reads `aria-expanded` itself, so it needs its own
        /// gate: a disabled field hides the list without closing the state.
        #[test]
        fn a_phone_field_disabled_while_open_claims_no_listbox() {
            let (mut dom, find) = mount(|| phone(false));
            let button = find.first_click.expect("the country button takes clicks");
            dom.runtime()
                .handle_event("click", Event::new(click_event(), true), button);
            settle(&mut dom, &mut dioxus::core::NoOpMutations);
            assert_listbox(&dom, "enabled after a click");

            lock(&mut dom, &mut dioxus::core::NoOpMutations);
            assert_no_listbox_claimed(&dom, "disabled while open");
        }

        #[test]
        fn arrow_down_on_a_readonly_phone_field_opens_no_list() {
            let (mut dom, find) = mount(|| phone(true));
            let button = find.first_click.expect("the country button takes clicks");
            press(&mut dom, button, Key::ArrowDown);
            assert_no_listbox_claimed(&dom, "readonly after ArrowDown");
        }

        /// A name for the failure message, and the app that locks that way.
        type Case = (&'static str, fn() -> Element);

        fn category(readonly: bool) -> Element {
            rsx! {
                LiberoProvider {
                    Cascader {
                        label: "Category",
                        data: vec![CascaderOption::new("tea", "Tea")],
                        readonly: readonly && locked(),
                        disabled: !readonly && locked(),
                        onchange: move |_: Option<String>| {},
                    }
                }
            }
        }

        /// Todo 439 (b): `Cascader` reads its own state, so it needs the gate too.
        #[test]
        fn a_cascader_locked_while_open_claims_no_listbox() {
            let cases: [Case; 2] = [
                ("disabled", || category(false)),
                ("readonly", || category(true)),
            ];
            for (how, app) in cases {
                let (mut dom, find) = mount(app);
                press(&mut dom, last_keydown(&find), Key::ArrowDown);
                assert_listbox(&dom, "enabled after ArrowDown");

                lock(&mut dom, &mut dioxus::core::NoOpMutations);
                assert_no_listbox_claimed(&dom, &format!("{how} while open"));
            }
        }

        fn due(readonly: bool) -> Element {
            rsx! {
                LiberoProvider {
                    DateField {
                        label: "Due",
                        readonly: readonly && locked(),
                        disabled: !readonly && locked(),
                        onchange: move |_| {},
                    }
                }
            }
        }

        fn accent(readonly: bool) -> Element {
            rsx! {
                LiberoProvider {
                    ColorField {
                        label: "Accent",
                        readonly: readonly && locked(),
                        disabled: !readonly && locked(),
                        oninput: move |_| {},
                    }
                }
            }
        }

        /// The picker fields open a dialog, not a listbox, and claim it the same
        /// way: `aria-expanded` and `aria-controls` on the text input.
        fn assert_dialog(dom: &VirtualDom, open: bool, what: &str) {
            let html = body(&dioxus_ssr::render(dom));
            assert_eq!(html.contains(r#"role="dialog""#), open, "{what}:\n{html}");
            assert_eq!(
                html.contains(r#"aria-expanded="true""#),
                open,
                "{what}: aria-expanded:\n{html}"
            );
            assert_eq!(
                html.contains("aria-controls"),
                open,
                "{what}: aria-controls:\n{html}"
            );
        }

        /// Todo 439 (b): the date and colour fields gate their dialog on
        /// `disabled` and `readonly`, and the input's ARIA reads that same gate.
        #[test]
        fn a_picker_field_locked_while_open_claims_no_dialog() {
            let cases: [Case; 4] = [
                ("DateField disabled", || due(false)),
                ("DateField readonly", || due(true)),
                ("ColorField disabled", || accent(false)),
                ("ColorField readonly", || accent(true)),
            ];
            for (how, app) in cases {
                let (mut dom, find) = mount(app);
                dom.runtime().handle_event(
                    "click",
                    Event::new(click_event(), true),
                    last_click(&find),
                );
                settle(&mut dom, &mut dioxus::core::NoOpMutations);
                assert_dialog(&dom, true, &format!("{how}: after a click"));

                lock(&mut dom, &mut dioxus::core::NoOpMutations);
                assert_dialog(&dom, false, &format!("{how}: while open"));
            }
        }
    }

    /// Todo 439 (a): the pointer refuses what the keys refuse. Each case runs once
    /// editable, so the refusal is believed next to the same click being answered.
    mod pointer_guards {
        use super::*;
        use libero::components::Autocomplete;

        /// Mounts `app` read-only or not and clicks the element labelled `label`,
        /// if it was drawn at all. Returns what the handler heard.
        fn click_labelled(
            app: fn() -> Element,
            readonly: bool,
            label: &'static str,
        ) -> Vec<String> {
            READ_ONLY.set(readonly);
            HEARD.with_borrow_mut(Vec::clear);
            let mut page = Page::mount(app);
            if let Some(&target) = page.rec.matching("click", "aria-label", label).first() {
                page.click(target);
            }
            HEARD.with_borrow(Clone::clone)
        }

        fn own_tags() -> Element {
            rsx! {
                LiberoProvider {
                    TagsField {
                        label: "Topics",
                        value: vec!["rust".to_string()],
                        readonly: READ_ONLY.get(),
                        onchange: move |next: Vec<String>| heard(next),
                        tag: move |args: SelectionArgs<String>| {
                            let label = format!("Drop {}", args.value);
                            rsx! {
                                button { "aria-label": label, onclick: move |_| args.remove.call(()), "x" }
                            }
                        },
                    }
                }
            }
        }

        /// A caller's `tag` got an unguarded `remove`, while Backspace refuses a
        /// read-only field.
        #[test]
        fn a_readonly_tags_field_refuses_a_custom_tags_remove() {
            let heard = click_labelled(own_tags, false, "Drop rust");
            assert_eq!(heard, ["[]"], "the custom x is the control");
            let heard = click_labelled(own_tags, true, "Drop rust");
            assert_eq!(heard, Vec::<String>::new());
        }

        fn own_chips() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect::<String> {
                        label: "Cities",
                        options: vec!["Berlin".to_string(), "Bonn".to_string()],
                        value: vec!["Berlin".to_string()],
                        readonly: READ_ONLY.get(),
                        onchange: move |next: Vec<String>| heard(next),
                        selection: move |args: SelectionArgs<String>| {
                            let label = format!("Drop {}", args.value);
                            rsx! {
                                button { "aria-label": label, onclick: move |_| args.remove.call(()), "x" }
                            }
                        },
                    }
                }
            }
        }

        /// The same for `MultiSelect`'s `selection`: the trigger's Backspace
        /// refuses a read-only select.
        #[test]
        fn a_readonly_multi_select_refuses_a_custom_chips_remove() {
            let heard = click_labelled(own_chips, false, "Drop Berlin");
            assert_eq!(heard, ["[]"], "the custom x is the control");
            let heard = click_labelled(own_chips, true, "Drop Berlin");
            assert_eq!(heard, Vec::<String>::new());
        }

        fn clearable_fruit() -> Element {
            rsx! {
                LiberoProvider {
                    Autocomplete {
                        label: "Fruit",
                        options: vec!["apple".to_string()],
                        value: "apple".to_string(),
                        clearable: true,
                        readonly: READ_ONLY.get(),
                        oninput: move |next: String| heard(next),
                    }
                }
            }
        }

        /// Every other clearable field hides its x while read-only; `Autocomplete`
        /// drew it, and a click emptied the text the native `readonly` protects.
        #[test]
        fn a_readonly_autocomplete_has_no_clear_to_click() {
            let heard = click_labelled(clearable_fruit, false, "Clear");
            assert_eq!(heard, ["\"\""], "the x is the control");
            let heard = click_labelled(clearable_fruit, true, "Clear");
            assert_eq!(heard, Vec::<String>::new());
        }
    }
}
