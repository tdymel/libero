use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Title,
    theme::{Size, Theme, TitleDefaults},
};

#[test]
fn title_renders_as_its_heading_level() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Title { component: "h2", "Heading" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("<h2"));
    assert!(attributes_of(&html, "h2").contains_key("data-state"));
}

static SMALL_TITLES: Theme = Theme {
    title: TitleDefaults {
        size: Size::Sm,
        ..Theme::DEFAULT.title
    },
    ..Theme::DEFAULT
};

/// The theme's step is the look only: a bare `Title` keeps its h1, so a theme
/// cannot move it in the document outline.
#[test]
fn an_unsized_title_takes_the_themes_look_but_keeps_its_h1() {
    fn app() -> Element {
        rsx! { LiberoProvider { theme: &SMALL_TITLES, Title { "Heading" } } }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "h1")["data-state"], "size-sm");
}
