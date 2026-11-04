//! Responsive FloatingWindow `placement` under Blitz (todo 5692): top start below `md`,
//! bottom end from it.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{FloatingWindowOptions, Placement},
    hooks::use_floating_window,
    theme::responsive,
};

fn app() -> Element {
    let window = use_floating_window(
        FloatingWindowOptions {
            title: Some("Inspector".into()),
            placement: responsive(Placement::TopStart)
                .md(Placement::BottomEnd)
                .into(),
            ..Default::default()
        },
        |_| rsx! { "Layers" },
    );
    use_hook(move || window.open());
    rsx! {}
}

/// The window's centre, `None` before it is laid out.
fn centre(page: &Page) -> Option<(f64, f64)> {
    page.rects("[role=dialog]")
        .first()
        .filter(|rect| rect.2 > 0.0)
        .map(|rect| (rect.0 + rect.2 / 2.0, rect.1 + rect.3 / 2.0))
}

#[test]
fn a_responsive_window_placement_follows_the_window() {
    let mut page = mount(app);
    page.resize(1280, 800);
    let end = page.wait_for(|page| centre(page).is_some_and(|(x, y)| x > 640.0 && y > 400.0));
    assert!(end, "{:?} at 1280", centre(&page));
    page.resize(600, 800);
    let start = page.wait_for(|page| centre(page).is_some_and(|(x, y)| x < 300.0 && y < 400.0));
    assert!(start, "{:?} at 600", centre(&page));
}
