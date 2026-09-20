//! Blitz's stylo matches none of the accessibility media features, so libero
//! settles them to `all` or `not all` in its own sheets (todo 954).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::Box;
use libero::hooks::use_accessibility;
use libero::sx::sx;

const SETTLED: &str = "#all{width:20px;} #none{width:20px;}
    @media all{#all{width:30px;}} @media not all{#none{width:30px;}}";

#[test]
fn stylo_matches_all_and_never_not_all() {
    let page = mount(|| {
        rsx! {
            style { dangerous_inner_html: SETTLED }
            div { id: "all" }
            div { id: "none" }
        }
    });
    assert_eq!(page.computed("#all", "width"), "30px");
    assert_eq!(page.computed("#none", "width"), "20px");
}

fn app() -> Element {
    let accessibility = use_accessibility();
    let force = move |reduced: Option<bool>| {
        let accessibility = accessibility.clone();
        move |_| accessibility.set_reduced_motion(reduced)
    };
    rsx! {
        button { id: "still", onclick: force(Some(true)) }
        button { id: "moving", onclick: force(Some(false)) }
        Box {
            id: "motion-box",
            sx: sx()
                .width("10px")
                .media("(prefers-reduced-motion: reduce)", sx().width("20px"))
                .media("(prefers-reduced-motion: no-preference)", sx().height("10px"))
                .media("(min-width: 1px) and (prefers-reduced-motion: reduce)", sx().min_height("5px")),
        }
    }
}

#[test]
fn a_forced_reduced_motion_answers_the_media_feature() {
    let mut page = mount(app);
    page.click("#moving");
    assert_eq!(page.computed("#motion-box", "width"), "10px");
    assert_eq!(page.computed("#motion-box", "height"), "10px");
    assert_eq!(page.computed("#motion-box", "min-height"), "auto");

    page.click("#still");
    assert_eq!(page.computed("#motion-box", "width"), "20px");
    assert_eq!(page.computed("#motion-box", "height"), "auto");
    assert_eq!(page.computed("#motion-box", "min-height"), "5px");
}
