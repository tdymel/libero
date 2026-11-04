//! Responsive Dialog `full_screen` under Blitz (todo 5657): edge to edge below `md`, centred from it.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::Dialog,
    hooks::{ModalScope, use_modal},
    theme::responsive,
};

fn app() -> Element {
    let modal = use_modal(|_: ModalScope<()>| {
        rsx! {
            Dialog { title: "Filters", full_screen: responsive(true).md(false), "Pick a colour." }
        }
    });
    use_hook(move || modal.open());
    rsx! {}
}

fn width(page: &Page) -> f64 {
    page.rects("[role=dialog]")
        .first()
        .map_or(0.0, |rect| rect.2)
}

#[test]
fn a_responsive_full_screen_dialog_follows_the_window() {
    let mut page = mount(app);
    page.resize(1280, 800);
    let centred = page.wait_for(|page| (1.0..700.0).contains(&width(page)));
    assert!(centred, "{} wide at 1280", width(&page));
    page.resize(600, 800);
    let full = page.wait_for(|page| width(page) >= 599.0);
    assert!(full, "{} wide at 600", width(&page));
}
