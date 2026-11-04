//! Placement per breakpoint (todo 5692): a tour card under its target below `md`, at its end
//! from it, and a floating window at the top start below `md`, at the bottom end from it.

use dioxus::prelude::*;
use libero::components::{Button, FloatingWindowOptions, Placement, TourOptions, TourStep, use_tour};
use libero::hooks::{Side, use_element, use_floating_window};
use libero::theme::responsive;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/responsive-placement/tour", || rsx! { TourPage {} }),
    ("/responsive-placement/window", || rsx! { WindowPage {} }),
];

#[component]
fn TourPage() -> Element {
    let target = use_element();
    let tour = use_tour(TourOptions {
        steps: vec![
            TourStep::new("save")
                .target(target)
                .title("Save")
                .side(responsive(Side::Bottom).md(Side::End)),
        ],
        ..Default::default()
    });
    rsx! {
        div { style: "padding: 40px",
            Button { id: "rsp-start", onclick: move |_| tour.start(), "Start" }
            Button { id: "rsp-target", onmounted: target.mount(), attributes: target.attributes(), "Save" }
        }
    }
}

#[component]
fn WindowPage() -> Element {
    let window = use_floating_window(
        FloatingWindowOptions {
            title: Some("Inspector".into()),
            placement: responsive(Placement::TopStart).md(Placement::BottomEnd).into(),
            ..Default::default()
        },
        |_| rsx! { "Layers" },
    );
    use_effect(move || window.open());
    rsx! {}
}
