//! Whether a component renders again when its parent does, counted off the
//! `render` span dioxus opens per scope run. The deterministic check of the
//! memoization comments in `render_cost.rs`, which only times the rows.

use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use dioxus::dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
use dioxus::logger::tracing::{
    self, Event, Metadata, Subscriber,
    field::{Field, Visit},
    span,
};
use dioxus::prelude::*;
use libero::{LiberoProvider, components::*};

/// The scope name of every `render` span opened while it is the default.
#[derive(Clone, Default)]
struct Renders(Arc<Mutex<Vec<String>>>);

#[derive(Default)]
struct ScopeName(String);

impl Visit for ScopeName {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "scope" {
            self.0 = format!("{value:?}");
        }
    }
}

impl Subscriber for Renders {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.name() == "render"
    }

    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        let mut scope = ScopeName::default();
        span.record(&mut scope);
        self.0.lock().expect("not poisoned").push(scope.0);
        span::Id::from_u64(1)
    }

    fn record(&self, _: &span::Id, _: &span::Record<'_>) {}
    fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}
    fn event(&self, _: &Event<'_>) {}
    fn enter(&self, _: &span::Id) {}
    fn exit(&self, _: &span::Id) {}
}

/// How often `name` renders on the first render, then on one re-render of the
/// app with nothing changed. A scope is named by its path, `libero::...::Divider`.
fn renders(app: fn() -> Element, name: &str) -> (usize, usize) {
    let count = |renders: Renders| {
        let names = renders.0.lock().expect("not poisoned");
        names
            .iter()
            .filter(|scope| scope.rsplit("::").next() == Some(name))
            .count()
    };
    let mut dom = VirtualDom::new(app);
    let first = Renders::default();
    tracing::subscriber::with_default(first.clone(), || dom.rebuild_in_place());
    let again = Renders::default();
    tracing::subscriber::with_default(again.clone(), || {
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate(&mut NoOpMutations);
    });
    (count(first), count(again))
}

/// `render_cost.rs`'s `TextField+validate` row: a rule that captures nothing
/// compares equal, a capturing one does not.
#[test]
fn a_text_field_with_capture_free_rules_skips_its_parents_rerender() {
    fn app() -> Element {
        let oninput = use_callback(|_: String| {});
        rsx! {
            LiberoProvider { TextField { value: "", oninput, validate: [not_empty.error("r")] } }
        }
    }
    fn capturing() -> Element {
        let oninput = use_callback(|_: String| {});
        let min = 2;
        let rule = move |value: &String| value.len() >= min;
        rsx! {
            LiberoProvider { TextField { value: "", oninput, validate: [rule.error("r")] } }
        }
    }
    assert_eq!(renders(app, "TextField"), (1, 0));
    assert_eq!(renders(capturing, "TextField"), (1, 1));
}

/// The header's claim: a component without `children` compares equal.
#[test]
fn a_divider_skips_its_parents_rerender() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Divider {} }
        }
    }
    assert_eq!(renders(app, "Divider"), (1, 0));
}

/// `children: Element` never compares equal.
#[test]
fn a_skeleton_renders_with_its_parent() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Skeleton { "x" } }
        }
    }
    assert_eq!(renders(app, "Skeleton"), (1, 1));
}

/// `slides: Vec<Element>` never compares equal.
#[test]
fn a_carousel_renders_with_its_parent() {
    fn app() -> Element {
        let onindexchange = use_callback(|_: usize| {});
        rsx! {
            LiberoProvider {
                Carousel { aria_label: "c", slides: vec![rsx! { "a" }, rsx! { "b" }], onindexchange }
            }
        }
    }
    assert_eq!(renders(app, "Carousel"), (1, 1));
}

/// The notifications rows' host: what redraws it is a store write, not its parent.
#[test]
fn a_notifications_host_skips_its_parents_rerender() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Notifications { limit: 3usize } }
        }
    }
    assert_eq!(renders(app, "Notifications"), (1, 0));
}
