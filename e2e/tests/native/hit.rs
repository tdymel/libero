//! Blitz hit testing: an inline box in a padded block takes no hit (an unpadded block or a flex
//! container avoids it); and the client rect of a translated box.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::ActionIcon;

fn app() -> Element {
    rsx! {
        div { padding_left: "200px", button { id: "padded-left", "A" } }
        div { padding_top: "20px", button { id: "padded-top", "B" } }
        div { margin_left: "200px", button { id: "margin", "C" } }
        div { padding_left: "200px", div { button { id: "wrapped", "D" } } }
        div { padding_left: "200px", display: "flex", button { id: "flex", "E" } }
        div { transform: "translate(40px, 40px)", display: "flex", button { id: "moved", "F" } }
    }
}

/// Hit testing follows the translate; Blitz's client rect does not, so the
/// harness and libero map the box through it (todo 734).
#[test]
fn a_translated_box_reports_where_it_is_drawn() {
    let page = mount(app);
    let (x, _, _, _) = page.rect("#moved");
    assert_eq!(x, 40.0, "#moved reports x {x}");
    assert!(page.hits("#moved"));
}

#[test]
fn inline_content_without_padding_around_it_takes_its_hits() {
    let page = mount(app);
    for id in ["#margin", "#wrapped", "#flex"] {
        assert!(page.hits(id), "{id} at {:?} takes no hit", page.rect(id));
    }
}

/// Todos 505, 566: a 20px `ActionIcon`'s `::before` takes presses 11.5px out
/// from its centre, and none past its 24px box.
#[test]
fn a_small_action_icon_takes_presses_in_its_24px_box() {
    fn icon_app() -> Element {
        let mut count = use_signal(|| 0);
        rsx! {
            div { display: "flex", padding: "20px",
                ActionIcon { id: "icon", aria_label: "Go", size: "sm", onclick: move |_| count += 1,
                    svg { view_box: "0 0 24 24", circle { cx: "12", cy: "12", r: "8" } }
                }
            }
            span { id: "count", "{count}" }
        }
    }
    let mut page = mount(icon_app);
    let (x, y, width, height) = page.rect("#icon");
    assert_eq!((width, height), (20.0, 20.0));
    let (cx, cy) = ((x + width / 2.0) as f32, (y + height / 2.0) as f32);
    for (dx, dy) in [(11.5, 0.0), (0.0, -11.5), (-11.5, 11.5), (13.0, 0.0)] {
        page.click_at(cx + dx, cy + dy);
    }
    assert_eq!(page.text("#count"), "3", "the 24px box or the miss past it");
}

#[test]
#[ignore = "needs Blitz: inline content in a padded block takes no hit"]
fn inline_content_in_a_padded_block_takes_its_hits() {
    let page = mount(app);
    for id in ["#padded-left", "#padded-top"] {
        assert!(page.hits(id), "{id} at {:?} takes no hit", page.rect(id));
    }
}
