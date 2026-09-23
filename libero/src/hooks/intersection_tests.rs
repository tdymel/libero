//! Off the web nothing can observe: the documented default is "never intersecting".

use std::cell::RefCell;

use dioxus::prelude::*;

use crate::hooks::{IntersectionEntry, IntersectionOptions, use_in_viewport, use_intersection};

thread_local! {
    static SEEN: RefCell<Option<(Option<IntersectionEntry>, bool)>> = const { RefCell::new(None) };
}

fn app() -> Element {
    let seen = use_intersection(IntersectionOptions {
        once: true,
        root_margin: "10px".to_string(),
        thresholds: vec![0.0, 0.5],
        ..Default::default()
    });
    let (on_mounted, visible) = use_in_viewport();
    SEEN.with(|slot| *slot.borrow_mut() = Some((*seen.entry.peek(), visible())));
    rsx! {
        div {
            onmounted: move |event| {
                seen.on_mounted.call(event.clone());
                on_mounted.call(event);
            },
        }
    }
}

#[test]
fn nothing_intersects_without_an_observer() {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    assert_eq!(SEEN.with(|slot| *slot.borrow()), Some((None, false)));
}

#[test]
fn the_options_default_to_the_whole_viewport() {
    let options = IntersectionOptions::default();
    assert!(options.root.is_none());
    assert_eq!(options.root_margin, "0px");
    assert_eq!(options.thresholds, vec![0.0]);
    assert!(!options.once);
}
