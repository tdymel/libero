use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Text,
    theme::{Size, TextDefaults, Theme},
};

#[test]
fn text_carries_its_size_as_a_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { size: Size::Lg, "body copy" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "p")["data-state"], "size-lg");
    assert!(body(&html).contains("body copy"));
}

static SMALL_TEXT: Theme = Theme {
    text: TextDefaults {
        size: Size::Sm,
        ..Theme::DEFAULT.text
    },
    ..Theme::DEFAULT
};

#[test]
fn an_unsized_text_takes_the_themes_default_size() {
    fn app() -> Element {
        rsx! { LiberoProvider { theme: &SMALL_TEXT, Text { "body copy" } } }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "p")["data-state"], "size-sm");
}
