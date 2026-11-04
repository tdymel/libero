//! Placement per breakpoint (todo 5692): Tour step `side`/`align` and FloatingWindowOptions
//! `placement`. Rust reads them, so the server renders the base and switches after mount.

use crate::common::render_with;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{FloatingWindowOptions, Placement, TourOptions, TourStep, use_tour},
    hooks::{Align, Side, use_floating_window},
    theme::{Responsive, responsive},
};

#[component]
fn Window(placement: Responsive<Placement>) -> Element {
    let window = use_floating_window(
        FloatingWindowOptions {
            title: Some("Inspector".into()),
            placement: placement.into(),
            ..Default::default()
        },
        |_| rsx! { "window body" },
    );
    use_hook(|| window.open());
    rsx! {}
}

#[component]
fn Tour(side: Responsive<Side>, align: Responsive<Align>) -> Element {
    let tour = use_tour(TourOptions {
        steps: vec![TourStep::new("save").title("Save").side(side).align(align)],
        ..Default::default()
    });
    use_hook(|| tour.start());
    rsx! {}
}

fn window_html(placement: Responsive<Placement>) -> String {
    render_with(
        |props: WindowProps| rsx! { LiberoProvider { Window { placement: props.placement } } },
        WindowProps { placement },
    )
}

fn tour_html(side: Responsive<Side>, align: Responsive<Align>) -> String {
    let mut dom = VirtualDom::new_with_props(
        |props: TourProps| {
            rsx! { LiberoProvider { Tour { side: props.side, align: props.align } } }
        },
        TourProps { side, align },
    );
    dom.rebuild_in_place();
    // The start lands after the slot was published empty.
    for _ in 0..4 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

#[test]
fn a_responsive_window_placement_renders_its_base_on_the_server() {
    let plain = window_html(Placement::BottomEnd.into());
    assert!(plain.contains("Inspector"), "{plain}");
    assert_ne!(window_html(Placement::TopStart.into()), plain);
    let per_breakpoint = responsive(Placement::BottomEnd).md(Placement::TopStart);
    assert_eq!(window_html(per_breakpoint), plain);
}

#[test]
fn a_responsive_tour_side_and_align_render_their_base_on_the_server() {
    let plain = tour_html(Side::Top.into(), Align::Start.into());
    assert!(plain.contains("Save"), "{plain}");
    let per_breakpoint = tour_html(
        responsive(Side::Top).md(Side::End),
        responsive(Align::Start).lg(Align::End),
    );
    assert_eq!(per_breakpoint, plain);
}

#[test]
fn a_plain_step_side_converts_into_its_responsive_field() {
    let step = TourStep::new("a").side(Side::Top).align(Align::End);
    assert_eq!(step.side, Responsive::from(Side::Top));
    assert_eq!(step.align, Responsive::from(Align::End));
    assert_eq!(TourStep::new("b").side, Responsive::from(Side::Bottom));
}
