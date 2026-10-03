use crate::common::{body, clear_buttons, element_at, render, tag_with, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{MultiSelect, Options, Select, SelectionArgs},
};

/// `Select` and `MultiSelect` closed. Clicks and keys are the `dispatched`
/// module's `select_keyboard` and `select_memo`; the open list is the Combobox's.
mod select_listbox {
    use super::*;

    #[derive(Clone, Copy, PartialEq, Debug, Options)]
    enum Fruit {
        Apple,
        Banana,
        Cherry,
    }

    #[test]
    fn the_trigger_is_a_combobox_named_by_the_label_and_shows_the_selection() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select { label: "Fruit", value: Fruit::Banana, onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains(r#"role="combobox""#), "{html}");
        assert!(html.contains(r#"aria-haspopup="listbox""#), "{html}");
        assert!(html.contains(r#"aria-expanded="false""#), "{html}");
        assert!(html.contains(r#"tabindex="0""#), "{html}");
        assert!(
            html.contains("Banana"),
            "the selection is not drawn:\n{html}"
        );
        assert!(
            !html.contains("Apple"),
            "a closed list rendered its rows:\n{html}"
        );

        let label_id = html
            .split(r#"<label data-slot="label" id=""#)
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .expect("the label carries an id");
        assert!(
            html.contains(&format!(r#"aria-labelledby="{label_id}""#)),
            "the trigger is not named by its label:\n{html}"
        );
    }

    /// A pick on a closed select skips `SelectCore` and redraws only what shows
    /// the value, so the trigger and the posted value must still move.
    #[test]
    fn a_pick_on_a_closed_select_redraws_its_value_and_its_post() {
        fn app() -> Element {
            let fruit = use_context_provider(|| Signal::new(Fruit::Apple));
            let onchange = use_callback(|_: Option<Fruit>| {});
            rsx! {
                LiberoProvider {
                    Select { name: "fruit", value: fruit(), onchange }
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let value = |dom: &VirtualDom| {
            let html = body(&dioxus_ssr::render(dom));
            let shown = html.contains(r#"data-slot="value">Cherry<"#);
            let posted = html.contains(r#"value="Cherry""#);
            (shown, posted)
        };
        assert_eq!(value(&dom), (false, false));

        let mut fruit = dom.in_scope(ScopeId::APP, consume_context::<Signal<Fruit>>);
        dom.in_runtime(|| fruit.set(Fruit::Cherry));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert_eq!(value(&dom), (true, true), "the pick left a stale value");
    }

    #[test]
    fn nothing_selected_shows_the_placeholder() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: None::<Fruit>,
                        placeholder: "Pick a fruit",
                        onchange: move |_: Option<Fruit>| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("data-placeholder"), "{html}");
        assert!(html.contains("Pick a fruit"), "{html}");
    }

    #[test]
    fn clearable_offers_the_x_only_while_something_is_selected() {
        fn picked() -> Element {
            rsx! {
                LiberoProvider {
                    Select { value: Fruit::Apple, clearable: true, onchange: move |_| {} }
                }
            }
        }
        fn empty() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: None::<Fruit>,
                        clearable: true,
                        onchange: move |_: Option<Fruit>| {},
                    }
                }
            }
        }

        assert_eq!(clear_buttons(&body(&render(picked))).len(), 1);
        assert_eq!(clear_buttons(&body(&render(empty))).len(), 0);
    }

    #[test]
    fn a_disabled_select_leaves_the_tab_order() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select { value: Fruit::Apple, disabled: true, onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains(r#"aria-disabled="true""#), "{html}");
        assert!(!html.contains(r#"tabindex="0""#), "{html}");
    }

    #[test]
    fn a_multi_select_draws_each_value_in_the_order_it_was_picked() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { value: vec![Fruit::Cherry, Fruit::Apple], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        let cherry = html.find("Cherry").expect("Cherry is drawn");
        let apple = html.find("Apple").expect("Apple is drawn");
        assert!(cherry < apple, "the chips lost the pick order:\n{html}");
        assert!(
            !html.contains("Banana"),
            "an unpicked value is drawn:\n{html}"
        );
        assert!(
            html.contains("multiple"),
            "the trigger lost its `multiple` state:\n{html}"
        );
    }

    #[test]
    fn every_chip_carries_a_labelled_remove_button_and_an_id_to_point_at() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { value: vec![Fruit::Cherry, Fruit::Apple], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        // Labelled, and not tab stops: the cursor reaches them through the trigger.
        for name in ["Remove Cherry", "Remove Apple"] {
            let remove = tag_with(&html, &format!(r#"aria-label="{name}""#));
            assert_eq!(
                remove.get("tabindex").map(String::as_str),
                Some("-1"),
                "{remove:?}"
            );
        }
        // The ids `aria-activedescendant` points at once the cursor moves.
        let chips = tags_with(&html, r#"data-slot="chip""#);
        assert_eq!(chips.len(), 2, "a chip lost its wrapper:\n{html}");
        let ids: Vec<_> = chips.iter().filter_map(|chip| chip.get("id")).collect();
        assert_eq!(ids.len(), 2, "a chip has no id:\n{html}");
        assert_ne!(ids[0], ids[1]);
        assert!(
            !html.contains("aria-activedescendant"),
            "a closed select with no cursor still names a descendant:\n{html}"
        );
    }

    /// A disabled field's chips keep their x, disabled, and the field leaves
    /// the tab order.
    #[test]
    fn a_disabled_multi_select_disables_every_chips_remove_button() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect {
                        value: vec![Fruit::Cherry, Fruit::Apple],
                        disabled: true,
                        onchange: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        for name in ["Remove Cherry", "Remove Apple"] {
            let remove = tag_with(&html, &format!(r#"aria-label="{name}""#));
            assert_eq!(
                remove.get("disabled").map(String::as_str),
                Some("true"),
                "{remove:?}"
            );
        }
        let trigger = tag_with(&html, r#"role="combobox""#);
        assert_eq!(trigger["aria-disabled"], "true", "{trigger:?}");
        assert!(!trigger.contains_key("tabindex"), "{trigger:?}");
    }

    /// Todo 70 (b): inside the combobox, each chip's x was part of its value,
    /// which read "Cherry Remove Cherry Apple Remove Apple". The chips now sit
    /// before the trigger, and the trigger says the selection as text.
    #[test]
    fn the_chips_sit_beside_the_combobox_and_it_says_the_selection() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { value: vec![Fruit::Cherry, Fruit::Apple], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        let combobox = html.find(r#"role="combobox""#).expect("no combobox");
        let inside = &html[combobox..];
        assert!(
            !inside.contains("Remove"),
            "a remove button is inside or after the combobox:\n{html}"
        );
        assert!(
            inside.contains("Cherry, Apple"),
            "the combobox does not say the selection:\n{html}"
        );
        // Two regions, both empty: the list's loading status and the chips'
        // own. Mounted empty, because a list that was already there is not
        // news.
        assert_eq!(
            html.matches(r#"role="status"></span>"#).count(),
            2,
            "the chips' live region is missing, or spoke at mount:\n{html}"
        );
    }

    #[test]
    fn the_chips_follow_the_field_one_step_down_the_size_scale() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect { size: "lg", value: vec![Fruit::Cherry], onchange: move |_| {} }
                }
            }
        }
        let html = body(&render(app));

        // The size rides the chip inside the wrapper.
        let wrapper = element_at(&html, html.find(r#"data-slot="chip""#).expect("a chip"));
        let chip = tag_with(wrapper, "data-state=");
        assert!(
            chip["data-state"]
                .split_whitespace()
                .any(|token| token == "size-md"),
            "an `lg` field did not draw an `md` chip: {chip:?}"
        );
    }

    /// The chip's inner design is the caller's, remove control included - the
    /// component adds nothing of its own around what `selection` returns.
    #[test]
    fn a_custom_selection_draws_only_what_the_caller_drew() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect {
                        value: vec![Fruit::Cherry],
                        onchange: move |_| {},
                        selection: move |args: SelectionArgs<Fruit>| rsx! {
                            span { onclick: move |_| args.remove.call(()), "{args.value.label()}!" }
                        },
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(html.contains("Cherry!"), "{html}");
        assert!(
            !html.contains("aria-label=\"Remove Cherry\""),
            "the default x survived a custom selection:\n{html}"
        );
    }

    /// Todo 1888: a caller's chip hears what the default one does, so it can
    /// drop or disable its remove control.
    #[test]
    fn a_custom_selection_hears_disabled_and_readonly() {
        fn app() -> Element {
            let chip = move |args: SelectionArgs<Fruit>| {
                rsx! {
                    span { "{args.value.label()}:{args.disabled}:{args.readonly}" }
                }
            };
            rsx! {
                LiberoProvider {
                    MultiSelect { value: vec![Fruit::Cherry], onchange: move |_| {}, selection: chip }
                    MultiSelect { value: vec![Fruit::Cherry], onchange: move |_| {}, disabled: true, selection: chip }
                    MultiSelect { value: vec![Fruit::Cherry], onchange: move |_| {}, readonly: true, selection: chip }
                }
            }
        }
        let html = body(&render(app));

        for drawn in [
            "Cherry:false:false",
            "Cherry:true:false",
            "Cherry:false:true",
        ] {
            assert!(html.contains(drawn), "no {drawn:?} in\n{html}");
        }
    }

    /// `searchable` must not leak a dropdown into a closed select. The core
    /// keeps an *open* list alive through a query that matches nothing - that
    /// is what the header slot is for - and a closed one draws neither.
    #[test]
    fn a_closed_searchable_select_draws_no_search_box() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Select {
                        value: Fruit::Apple,
                        searchable: true,
                        search_placeholder: "Find a fruit",
                        onchange: move |_| {},
                    }
                }
            }
        }
        let html = body(&render(app));

        assert!(
            !html.contains("Find a fruit"),
            "a closed select drew its search box:\n{html}"
        );
        assert!(!html.contains(r#"role="listbox""#), "{html}");
        // Closed, the trigger is still the combobox - the role only moves to
        // the search box while one exists.
        assert!(html.contains(r#"role="combobox""#), "{html}");
    }
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::{attributes_of, body};
    use crate::dispatch::*;
    use dioxus::core::ElementId;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Form, NativeSelect, Options},
    };

    /// A label that is not the variant's name, so a test can tell which of the
    /// two a control posted.
    #[derive(Clone, Copy, PartialEq, Debug, Options)]
    pub enum Plan {
        #[option(label = "Free plan")]
        Free,
        #[option(label = "Pro plan")]
        Pro,
    }

    #[derive(Clone, PartialEq, Default, libero::components::Fields)]
    pub struct Signup {
        pub plan: Option<Plan>,
    }

    /// Mounts `app`, sends `change` carrying `posted` to the select, and returns
    /// what the handler heard and the markup afterwards.
    fn change_select(app: fn() -> Element, posted: &str) -> (Vec<String>, String) {
        HEARD.with_borrow_mut(Vec::clear);
        let Page { mut dom, rec: find } = Page::build(app);
        let select = find.change.expect("registered no change listener");
        dom.runtime()
            .handle_event("change", Event::new(input_event(posted), true), select);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        (HEARD.with_borrow(Clone::clone), dioxus_ssr::render(&dom))
    }

    /// Todo 20: the `<select>` reports the chosen option's `value`, which is
    /// `Options::value` now - so that is what `onchange` looks up. An index or
    /// the visible label is no option's value any more, and picks nothing.
    #[test]
    fn a_native_select_reads_the_option_value_back() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    NativeSelect {
                        value: Plan::Free,
                        onchange: move |next: Plan| heard(next),
                    }
                }
            }
        }

        assert_eq!(change_select(app, "Pro").0, ["Pro"]);
        assert_eq!(change_select(app, "1").0, Vec::<String>::new());
        assert_eq!(change_select(app, "Pro plan").0, Vec::<String>::new());
    }

    /// The same round trip without `onchange`: the path `name` writes the
    /// `Form`'s value, and the select renders it back as selected.
    #[test]
    fn a_native_select_with_a_path_name_writes_its_forms_value() {
        fn app() -> Element {
            let signup = use_store(Signup::default);
            rsx! {
                LiberoProvider {
                    Form { value: signup,
                        NativeSelect { name: Signup::FIELDS.plan(), placeholder: "Pick" }
                    }
                    "plan: {signup.read().plan:?}"
                }
            }
        }

        let (_, html) = change_select(app, "Pro");
        assert!(html.contains("plan: Some(Pro)"), "{html}");
        assert!(html.contains("<option value=\"Pro\" selected"), "{html}");
        let select = attributes_of(&body(&html), "select");
        assert_eq!(select.get("name").map(String::as_str), Some("plan"));
    }

    /// `Select`'s typeahead, and the arrows passing over a disabled row. Both are
    /// keyboard-only, so they need real dispatched events rather than SSR.
    mod select_keyboard {
        use super::*;
        use libero::components::{OptionItem, OptionList, Select};

        thread_local! {
            static PICKED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
        }

        fn cities() -> OptionList<String> {
            OptionList::new([
                "Berlin".to_string().into(),
                "Bonn".to_string().into(),
                // Skipped by the arrows and by typeahead alike, which is why it
                // sits between two rows that a "B" would otherwise walk through.
                OptionItem::new("Bochum".to_string()).disabled(true),
                "Cologne".to_string().into(),
            ])
        }

        /// Really controlled: `Select` renders `value` and asks for a new one, so
        /// a test that never moves it would have typeahead searching from the same
        /// place every time - and would never see the cycle at all.
        fn app() -> Element {
            let mut city = use_signal(|| None::<String>);
            rsx! {
                LiberoProvider {
                    Select::<String> {
                        label: "City",
                        options: cities(),
                        value: city(),
                        onchange: move |next: Option<String>| {
                            PICKED.with_borrow_mut(|picked| picked.push(next.clone().unwrap_or_default()));
                            city.set(next);
                        },
                    }
                }
            }
        }

        /// A dom with the event converter installed and the picks reset, plus the
        /// trigger's element id - the innermost element carrying a `keydown`,
        /// since `ComboboxCore`'s wrapper registers one first.
        fn mount() -> (VirtualDom, ElementId) {
            PICKED.with_borrow_mut(Vec::clear);
            let Page { dom, rec: find } = Page::build(app);
            let trigger = *find.keydown.last().expect("the trigger listens for keys");
            (dom, trigger)
        }

        fn type_keys(dom: &mut VirtualDom, trigger: ElementId, keys: &str) {
            for ch in keys.chars() {
                press(dom, trigger, Key::Character(ch.to_string()));
            }
        }

        /// Two passes: the list reports its row count while it renders, after the
        /// trigger has been drawn, so the trigger catches up one pass later.
        fn press(dom: &mut VirtualDom, trigger: ElementId, key: Key) {
            dom.runtime()
                .handle_event("keydown", Event::new(key_event(key), true), trigger);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }

        fn picked() -> Vec<String> {
            PICKED.with_borrow(Clone::clone)
        }

        /// The buffer: three characters inside the window narrow to one row, and
        /// a closed trigger changes the value in place the way a native `<select>`
        /// does - no list is opened at all.
        #[test]
        fn a_buffered_query_picks_in_place_on_a_closed_trigger() {
            let (mut dom, trigger) = mount();
            type_keys(&mut dom, trigger, "ber");

            // "b" lands on Berlin, and "be" then "ber" stay on it while it still
            // matches - the narrowing half of the buffer.
            assert_eq!(picked(), ["Berlin", "Berlin", "Berlin"]);
            let html = body(&dioxus_ssr::render(&dom));
            assert!(
                !html.contains(r#"role="listbox""#),
                "typing opened the list:\n{html}"
            );
        }

        /// One character, pressed again, cycles - `typeahead_match` treats a
        /// repeat as a single-character search that starts *after* the current
        /// row. Bochum is disabled, so "B" walks Berlin, Bonn and back.
        #[test]
        fn a_repeated_character_cycles_and_skips_a_disabled_row() {
            let (mut dom, trigger) = mount();
            type_keys(&mut dom, trigger, "b");
            assert_eq!(picked(), ["Berlin"]);

            // Each press searches from the row the last one selected, so the
            // repeat walks on rather than landing on Berlin again.
            type_keys(&mut dom, trigger, "b");
            assert_eq!(picked(), ["Berlin", "Bonn"]);
            type_keys(&mut dom, trigger, "b");
            assert_eq!(
                picked(),
                ["Berlin", "Bonn", "Berlin"],
                "Bochum is disabled, so the cycle wraps past it"
            );
        }

        /// Space still opens the list, as it always has - typeahead takes a space
        /// only mid-query, where it is part of "new york".
        #[test]
        fn space_opens_the_list_rather_than_typing() {
            let (mut dom, trigger) = mount();
            press(&mut dom, trigger, Key::Character(" ".into()));

            assert_eq!(picked(), Vec::<String>::new());
            let html = body(&dioxus_ssr::render(&dom));
            assert!(html.contains(r#"role="listbox""#), "{html}");
        }

        /// An open list moves its highlight instead of picking, and the arrows
        /// pass over the disabled row: Berlin, Bonn, then Cologne.
        #[test]
        fn the_arrows_skip_a_disabled_row() {
            let (mut dom, trigger) = mount();
            press(&mut dom, trigger, Key::Character(" ".into()));

            let rows = |dom: &VirtualDom| {
                let html = body(&dioxus_ssr::render(dom));
                let at = html
                    .find(r#"aria-activedescendant=""#)
                    .map(|at| at + r#"aria-activedescendant=""#.len());
                at.map(|at| html[at..].split('"').next().unwrap().to_string())
            };

            // Opening already arms the first row, as a native `<select>` does.
            assert!(rows(&dom).is_some_and(|id| id.ends_with("-option-0")));
            press(&mut dom, trigger, Key::ArrowDown);
            assert!(rows(&dom).is_some_and(|id| id.ends_with("-option-1")));
            // Row 2 is Bochum, which is disabled.
            press(&mut dom, trigger, Key::ArrowDown);
            let id = rows(&dom).expect("a row to point at");
            assert!(
                id.ends_with("-option-3"),
                "the arrows stopped on Bochum: {id}"
            );
            // And back up over it.
            press(&mut dom, trigger, Key::ArrowUp);
            let id = rows(&dom).expect("a row to point at");
            assert!(id.ends_with("-option-1"), "{id}");

            assert_eq!(picked(), Vec::<String>::new(), "an arrow picked something");
        }
    }

    /// A closed `Select`'s `SelectCore` skips a parent re-render (todo 29). What it
    /// drew on an older render must still run the newest handler and show the newest
    /// caller-drawn selection.
    mod select_memo {
        use super::*;
        use libero::components::Select;

        thread_local! {
            static PICKS: std::cell::RefCell<Vec<(u32, String)>> = const { std::cell::RefCell::new(Vec::new()) };
        }

        /// A select on `value`, drawing its own selection when `own_selection`.
        /// Each render's handler and selection carry the generation the test set.
        fn city((value, own_selection): (&'static str, bool)) -> Element {
            let generation = use_context_provider(|| Signal::new(1u32))();
            let selection = own_selection
                .then(|| Callback::new(move |city: String| rsx! { "{city} #{generation}" }));
            rsx! {
                LiberoProvider {
                    Select::<String> {
                        options: vec!["Berlin".to_string(), "Bonn".to_string(), "Hamburg".to_string()],
                        value: value.to_string(),
                        selection,
                        onchange: move |city: Option<String>| {
                            PICKS.with_borrow_mut(|picks| picks.push((generation, city.unwrap_or_default())))
                        },
                    }
                }
            }
        }

        fn mount(value: &'static str, own_selection: bool) -> (VirtualDom, ElementId) {
            dioxus::html::set_event_converter(Box::new(TestConverter));
            PICKS.with_borrow_mut(Vec::clear);
            let mut dom = VirtualDom::new_with_props(city, (value, own_selection));
            let mut find = FindClickListener::default();
            dom.rebuild(&mut find);
            dom.render_immediate(&mut find);
            let trigger = *find.keydown.last().expect("the trigger takes keys");
            (dom, trigger)
        }

        fn rerender(dom: &mut VirtualDom, generation: u32) {
            let mut current = dom.in_scope(ScopeId::APP, consume_context::<Signal<u32>>);
            dom.in_runtime(|| current.set(generation));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }

        fn press(dom: &mut VirtualDom, trigger: ElementId, key: Key) {
            dom.runtime()
                .handle_event("keydown", Event::new(key_event(key), true), trigger);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }

        #[test]
        fn a_closed_select_picks_through_the_newest_onchange() {
            let (mut dom, trigger) = mount("Berlin", false);
            rerender(&mut dom, 2);
            rerender(&mut dom, 3);
            // Typeahead on a closed select picks in place: "b" from Berlin is Bonn.
            press(&mut dom, trigger, Key::Character("b".into()));
            assert_eq!(PICKS.with_borrow(Clone::clone), [(3, "Bonn".to_string())]);
        }

        #[test]
        fn a_callers_selection_redraws_on_its_own_state() {
            let (mut dom, _) = mount("Berlin", true);
            assert!(body(&dioxus_ssr::render(&dom)).contains("Berlin #1"));
            rerender(&mut dom, 2);
            let html = body(&dioxus_ssr::render(&dom));
            assert!(
                html.contains("Berlin #2"),
                "a stale caller selection:\n{html}"
            );
        }

        /// No rows are drawn while closed, so the highlight a list opens on is
        /// counted over the options, not over the rows.
        #[test]
        fn opening_after_skipped_renders_highlights_the_selected_row() {
            let (mut dom, trigger) = mount("Hamburg", false);
            rerender(&mut dom, 2);
            press(&mut dom, trigger, Key::ArrowDown);
            let html = body(&dioxus_ssr::render(&dom));
            let label = html
                .find(r#"data-slot="label">Hamburg"#)
                .unwrap_or_else(|| panic!("no Hamburg row:\n{html}"));
            let row = &html[html[..label].rfind("<div").expect("the row")..label];
            assert!(
                row.contains("active"),
                "Hamburg is not the highlight:\n{html}"
            );
        }
    }

    /// An open list hands unchanged rows out again (todo 2022); a renamed or disabled row
    /// must still redraw, and so must the highlight it loses.
    mod select_row_cache {
        use super::*;
        use libero::components::{OptionItem, OptionList, Select};

        fn city(_: ()) -> Element {
            let generation = use_context_provider(|| Signal::new(1u32))();
            let options = OptionList::new([
                "Berlin".to_string().into(),
                OptionItem::new(format!("Bonn {generation}")).disabled(generation > 1),
                "Cologne".to_string().into(),
            ]);
            rsx! {
                LiberoProvider {
                    Select::<String> { options, value: "Berlin".to_string(), onchange: move |_| {} }
                }
            }
        }

        fn row<'a>(html: &'a str, label: &str) -> &'a str {
            let at = html
                .find(&format!(r#"data-slot="label">{label}<"#))
                .unwrap_or_else(|| panic!("no {label} row:\n{html}"));
            &html[html[..at].rfind("<div").expect("the row")..at]
        }

        #[test]
        fn a_renamed_or_disabled_row_redraws_in_an_open_list() {
            dioxus::html::set_event_converter(Box::new(TestConverter));
            let mut dom = VirtualDom::new_with_props(city, ());
            let mut find = FindClickListener::default();
            dom.rebuild(&mut find);
            dom.render_immediate(&mut find);
            let trigger = *find.keydown.last().expect("the trigger takes keys");
            press_twice(&mut dom, trigger, Key::ArrowDown);
            press_twice(&mut dom, trigger, Key::ArrowDown);
            let html = body(&dioxus_ssr::render(&dom));
            assert!(row(&html, "Bonn 1").contains("active"), "{html}");

            let mut generation = dom.in_scope(ScopeId::APP, consume_context::<Signal<u32>>);
            dom.in_runtime(|| generation.set(2));
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            let html = body(&dioxus_ssr::render(&dom));
            assert!(!html.contains("Bonn 1"), "a stale label:\n{html}");
            let bonn = row(&html, "Bonn 2");
            assert!(bonn.contains(r#"aria-disabled="true""#), "{html}");
            assert!(
                !bonn.contains("active"),
                "a disabled row kept the highlight:\n{html}"
            );
            assert!(!row(&html, "Berlin").contains(r#"aria-disabled"#), "{html}");
        }

        fn press_twice(dom: &mut VirtualDom, trigger: ElementId, key: Key) {
            dom.runtime()
                .handle_event("keydown", Event::new(key_event(key), true), trigger);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
    }
}
