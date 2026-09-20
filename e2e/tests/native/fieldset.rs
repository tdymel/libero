//! `Fieldset`'s legend natively: Blitz lays it out as the first flex item, so
//! the fieldset's `gap` opened below it where the web has none.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Fieldset, TextField};

fn app() -> Element {
    rsx! {
        Fieldset::<String> { label: "Account",
            div { id: "first", TextField { aria_label: "Name" } }
            div { id: "second", TextField { aria_label: "Email" } }
        }
    }
}

#[test]
fn the_legend_sits_its_own_margin_above_the_first_field() {
    let page = mount(app);
    let (_, legend_top, _, legend_height) = page.rect("legend");
    let (_, first_top, _, first_height) = page.rect("#first");
    let (_, second_top, _, _) = page.rect("#second");
    // 4px, give or take Blitz's pixel rounding; 16.5 with the gap.
    let below = first_top - (legend_top + legend_height);
    assert!((below - 4.0).abs() <= 1.0, "{below}");
    // The gap still spaces the fields themselves.
    assert!(second_top - (first_top + first_height) > 4.0);
}
