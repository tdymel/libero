//! Events dispatched the way a renderer does: the payload is a
//! `PlatformEventData`, not the concrete event type. A listener built without
//! that conversion type-checks and passes an SSR test, but panics on the first
//! real click - see `BoxBuilder::event`.

use crate::common::body;
use crate::dispatch::*;
use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Box, Table, column},
};

/// Every listener name registered during a rebuild, in order.
#[derive(Default)]
struct Listeners(Vec<String>);

impl WriteMutations for Listeners {
    fn add_event_listener(&mut self, name: &str) {
        self.0.push(name.to_string());
    }
    fn push_id(&mut self, _id: ElementId) {}
    fn set_id(&mut self, _id: ElementId) {}
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {}
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
    fn create_text(&mut self, _value: &str) {}
    fn clone(&mut self) {}
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_attribute(&mut self, _n: &str, _ns: Option<&str>, _v: &AttributeValue) {}
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

/// What lets a field declare no `onfocus`, `onblur` or `onkeydown` and still
/// accept one: `base_props!` extends `GlobalAttributes`, which carries
/// listeners and not only attributes. `Button` declares `onclick` for its own
/// ripple reasons, so nothing else pins this down.
#[test]
fn global_attributes_carry_event_listeners_through_the_spread() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Box { oninput: move |_| {}, onfocus: move |_| {}, "x" }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    let mut listeners = Listeners::default();
    dom.rebuild(&mut listeners);

    for name in ["input", "focus"] {
        assert!(
            listeners.0.iter().any(|listener| listener == name),
            "no {name} listener: {:?}",
            listeners.0
        );
    }
}

/// Todo 428: the sort stays on its column when the columns move.
#[test]
fn a_table_sort_follows_its_column_through_a_reorder() {
    #[derive(Clone, PartialEq)]
    struct Person {
        name: &'static str,
        age: u32,
    }

    fn app() -> Element {
        let layout = use_context_provider(|| Signal::new(vec!["Name", "Age"]));
        let columns = layout()
            .into_iter()
            .map(|header| match header {
                "Name" => column("Name")
                    .value(|p: &Person| p.name.to_string())
                    .sortable(),
                _ => column("Age").value(|p: &Person| p.age).sortable(),
            })
            .collect::<Vec<_>>();

        rsx! {
            LiberoProvider {
                Table {
                    data: vec![
                        Person { name: "Grace", age: 45 },
                        Person { name: "Linus", age: 28 },
                        Person { name: "Ada", age: 36 },
                    ],
                    columns,
                }
            }
        }
    }

    fn row_order(html: &str) -> Vec<&'static str> {
        let mut names = ["Grace", "Linus", "Ada"];
        names.sort_by_key(|name| html.find(&format!(">{name}<")).expect("a missing row"));
        names.to_vec()
    }

    dioxus::html::set_event_converter(Box::new(TestConverter));
    let mut dom = VirtualDom::new(app);
    let mut find = FindClickListener::default();
    dom.rebuild(&mut find);
    let age = find.element("click", "text", "Age");

    dom.runtime()
        .handle_event("click", Event::new(click_event(), true), age);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(
        row_order(&dioxus_ssr::render(&dom)),
        ["Linus", "Ada", "Grace"]
    );

    let mut layout = dom.in_scope(ScopeId::APP, consume_context::<Signal<Vec<&str>>>);
    dom.in_runtime(|| layout.set(vec!["Age", "Name"]));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(row_order(&html), ["Linus", "Ada", "Grace"]);
    assert!(
        html.contains(r#"aria-sort="ascending"><button type="button" data-sort-button=true>Age"#),
        "{html}"
    );

    // Without its column the sort is gone, not moved onto "Name".
    dom.in_runtime(|| layout.set(vec!["Name"]));
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(row_order(&html), ["Grace", "Linus", "Ada"]);
    assert!(
        html.contains(r#"<button type="button" data-sort-button=true>Name"#),
        "{html}"
    );
    assert!(!body(&html).contains("aria-sort"), "{html}");
}
