//! Responsive form props under Blitz (todo 5652): every copy is a `<style>` here, and media
//! queries follow the window.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{Checkbox, Flex, TextField},
    theme::{ChoiceVariant, Size, responsive},
};

fn app() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md",
            div { id: "rspf-sm", TextField { label: "Small", size: "sm" } }
            div { id: "rspf-lg", TextField { label: "Large", size: "lg" } }
            div { id: "rspf-text",
                TextField { label: "Responsive", size: responsive(Size::Sm).md(Size::Lg) }
            }
            div { id: "rspf-card",
                Checkbox {
                    label: "Card from md",
                    variant: responsive(ChoiceVariant::Plain).md(ChoiceVariant::Card),
                }
            }
        }
    }
}

/// Frame heights of the small, large and responsive fields, then the card's border width.
fn look(page: &Page) -> (f64, f64, f64, String) {
    let frame = |id: &str| page.rect(&format!("#{id} [data-slot='frame']")).3;
    (
        frame("rspf-sm"),
        frame("rspf-lg"),
        frame("rspf-text"),
        page.computed("#rspf-card > div", "border-top-width"),
    )
}

#[test]
fn form_props_follow_the_window_width() {
    let mut page = mount(app);
    page.resize(1280, 800);
    let (small, large, field, card) = look(&page);
    assert_ne!(small, large);
    assert_eq!((field, card.as_str()), (large, "1px"));

    page.resize(600, 800);
    let (_, _, field, card) = look(&page);
    assert_eq!((field, card.as_str()), (small, "0px"));
}
