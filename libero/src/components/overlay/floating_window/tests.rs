use dioxus::prelude::*;

use super::{FloatingWindow, FloatingWindowOptions, FloatingWindowPart, geometry::WindowBounds};
use crate::{
    LiberoProvider,
    components::{
        common::Part,
        overlay::{Dialog, Modal},
    },
};

/// The slot names are public: a rename here is a breaking change.
#[test]
fn the_part_table_is_stable() {
    let table: Vec<_> = FloatingWindowPart::ALL
        .iter()
        .map(|part| (part.slot(), part.selector()))
        .collect();

    assert_eq!(
        table,
        [
            ("title-bar", "& > [data-slot='title-bar']"),
            (
                "handle",
                "& > [data-slot='title-bar'] > [data-slot='handle']"
            ),
            (
                "title",
                "& > [data-slot='title-bar'] > [data-slot='handle'] > [data-slot='title']"
            ),
            ("menu", "& > [data-slot='title-bar'] [data-slot='menu']"),
            ("close", "& > [data-slot='title-bar'] > [data-slot='close']"),
            ("steps", "& > [data-slot='steps']"),
            ("body", "& > [data-slot='body']"),
            ("resize", "& > [data-slot='resize']"),
        ]
    );
}

/// Todo 527: rendered inline in a modal, not portaled, the window still
/// cuts its content off from the modal's close.
#[test]
fn a_window_inside_a_modal_is_not_modal() {
    fn app() -> Element {
        let onclose = use_callback(|()| {});
        rsx! {
            LiberoProvider {
                Modal {
                    FloatingWindow {
                        options: FloatingWindowOptions {
                            title: Some("Window".into()),
                            ..Default::default()
                        },
                        onclose,
                        onmount: |_| {},
                        opener: |()| None,
                        Dialog { title: "Inner", "inner content" }
                    }
                }
            }
        }
    }
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("inner content"), "{html}");
    assert!(!html.contains("aria-modal=\"true\""), "{html}");
}

const BOUNDS: WindowBounds = WindowBounds {
    min: (240.0, 120.0),
    max: (480.0, 360.0),
};

/// Home and End ask for `0x0` and `u16::MAX`; the separator has to say
/// what the CSS draws, not what was asked for.
#[test]
fn a_request_is_clamped_as_the_css_clamps_it() {
    assert_eq!(BOUNDS.fit((0.0, 0.0)), (240.0, 120.0));
    assert_eq!(BOUNDS.fit((65535.0, 65535.0)), (480.0, 360.0));
    assert_eq!(BOUNDS.fit((300.0, 200.0)), (300.0, 200.0));
}

/// CSS lets `min-width` win over `max-width`.
#[test]
fn a_min_above_the_max_wins() {
    let crossed = WindowBounds {
        min: (500.0, 0.0),
        max: (480.0, 360.0),
    };
    assert_eq!(crossed.fit((65535.0, 10.0)).0, 500.0);
}
